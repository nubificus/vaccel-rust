// SPDX-License-Identifier: Apache-2.0

pub mod ops;
pub mod resource;
#[cfg(feature = "async")]
pub mod rpc_async;
#[cfg(not(feature = "async"))]
pub mod rpc_sync;
pub mod session;

use dashmap::DashMap;
use std::{default::Default, net::ToSocketAddrs, pin::Pin, sync::Arc};
use thiserror::Error;
#[cfg(feature = "async")]
use ttrpc::asynchronous::Server;
#[cfg(not(feature = "async"))]
use ttrpc::sync::Server;
use vaccel::{profiling::ProfRegions, Resource, Session, VaccelId};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::create_rpc_agent;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::create_rpc_agent;
use vaccel_rpc_proto::{
    error::VaccelError,
    profiling::{ProfilingRequest, ProfilingResponse},
};

#[derive(Clone)]
pub struct VaccelRpcAgent {
    pub sessions: Arc<DashMap<VaccelId, Box<Session>>>,
    pub resources: Arc<DashMap<VaccelId, Pin<Box<Resource>>>>,
    pub timers: Arc<DashMap<VaccelId, ProfRegions>>,
}

unsafe impl Sync for VaccelRpcAgent {}
unsafe impl Send for VaccelRpcAgent {}

impl VaccelRpcAgent {
    pub(crate) fn new() -> VaccelRpcAgent {
        VaccelRpcAgent {
            sessions: Arc::new(DashMap::new()),
            resources: Arc::new(DashMap::new()),
            timers: Arc::new(DashMap::new()),
        }
    }

    pub(crate) fn do_get_timers(&self, req: ProfilingRequest) -> ttrpc::Result<ProfilingResponse> {
        let timers = self
            .timers
            .entry(req.session_id.into())
            .or_insert_with(|| ProfRegions::new("vaccel-agent"));

        Ok(ProfilingResponse {
            result: Some(timers.clone().into()).into(),
            ..Default::default()
        })
    }
}

#[derive(Error, Debug)]
pub enum Error {
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(String),

    /// Socket error
    #[error("ttprc error: {0}")]
    TtrpcError(ttrpc::Error),

    /// Undefined error
    #[error("Undefined error")]
    Undefined,
}

impl From<ttrpc::Error> for Error {
    fn from(err: ttrpc::Error) -> Self {
        Error::TtrpcError(err)
    }
}

pub(crate) fn ttrpc_error(code: ttrpc::Code, msg: String) -> ttrpc::Error {
    ttrpc::Error::RpcStatus(ttrpc::error::get_status(code, msg))
}

pub(crate) fn vaccel_error(err: vaccel::Error) -> VaccelError {
    let mut grpc_error = VaccelError::new();

    match err {
        vaccel::Error::Runtime(e) => grpc_error.set_vaccel_error(e as i64),
        vaccel::Error::InvalidArgument => grpc_error.set_agent_error(1i64),
        vaccel::Error::Uninitialized => grpc_error.set_agent_error(2i64),
        #[cfg(target_pointer_width = "64")]
        vaccel::Error::TensorFlow(_) => grpc_error.set_agent_error(3i64),
        vaccel::Error::TensorFlowLite(_) => grpc_error.set_agent_error(4i64),
        vaccel::Error::Torch(_) => grpc_error.set_agent_error(5i64),
        vaccel::Error::Others(_) => grpc_error.set_agent_error(6i64),
    }

    grpc_error
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn resolve_uri(uri: &str) -> Result<String> {
    let parts: Vec<&str> = uri.split("://").collect();
    if parts.len() != 2 {
        return Err(Error::AgentError("Invalid server address uri".into()));
    }

    let scheme = parts[0].to_lowercase();
    match scheme.as_str() {
        "vsock" | "unix" => Ok(uri.to_string()),
        "tcp" => {
            let address = parts[1].to_lowercase();
            let mut resolved = match address.to_socket_addrs() {
                Ok(a) => a,
                Err(e) => return Err(Error::AgentError(e.to_string())),
            };
            let resolved_address = match resolved.next() {
                Some(a) => a.to_string(),
                None => {
                    return Err(Error::AgentError(
                        "Could not resolve TCP server address".into(),
                    ))
                }
            };

            Ok(format!("{}://{}", scheme, resolved_address.as_str()))
        }
        _ => Err(Error::AgentError("Unsupported protocol".into())),
    }
}

pub fn server_init(server_address: &str) -> Result<Server> {
    let agent_worker = Arc::new(VaccelRpcAgent::new());
    let aservice = create_rpc_agent(agent_worker);

    if server_address.is_empty() {
        return Err(Error::AgentError("Server address cannot be empty".into()));
    }

    let resolved_uri = resolve_uri(server_address)?;
    let server: Server = Server::new()
        .bind(&resolved_uri)?
        .register_service(aservice);

    Ok(server)
}
