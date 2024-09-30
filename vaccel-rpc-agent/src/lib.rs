// SPDX-License-Identifier: Apache-2.0

pub mod ops;
pub mod resources;
#[cfg(feature = "async")]
pub mod rpc_async;
#[cfg(not(feature = "async"))]
pub mod rpc_sync;
pub mod session;

use dashmap::DashMap;
use std::{default::Default, error::Error, sync::Arc};
#[cfg(feature = "async")]
use ttrpc::asynchronous::Server;
#[cfg(not(feature = "async"))]
use ttrpc::sync::Server;
use vaccel::{profiling::ProfRegions, Resource, Session, VaccelId};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::{create_rpc_agent, RpcAgent};
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::create_rpc_agent;
use vaccel_rpc_proto::{
    error::VaccelError,
    profiling::{ProfilingRequest, ProfilingResponse},
};

#[derive(Clone)]
pub struct VaccelRpcAgent {
    pub sessions: Arc<DashMap<VaccelId, Box<Session>>>,
    pub resources: Arc<DashMap<VaccelId, Box<dyn Resource>>>,
    pub timers: Arc<DashMap<u32, ProfRegions>>,
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
            .entry(req.session_id)
            .or_insert_with(|| ProfRegions::new("vaccel-agent"));

        Ok(ProfilingResponse {
            result: Some(timers.clone().into()).into(),
            ..Default::default()
        })
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

pub fn server_init(server_address: &str) -> Result<Server, Box<dyn Error>> {
    let agent_worker = Arc::new(VaccelRpcAgent::new());
    let aservice = create_rpc_agent(agent_worker);

    if server_address.is_empty() {
        return Err("Server address cannot be empty".into());
    }

    let fields: Vec<&str> = server_address.split("://").collect();
    if fields.len() != 2 {
        return Err("Invalid address".into());
    }

    let scheme = fields[0].to_lowercase();
    let server: Server = match scheme.as_str() {
        "vsock" | "unix" | "tcp" => Server::new()
            .bind(server_address)?
            .register_service(aservice),
        _ => return Err("Unsupported protocol".into()),
    };

    Ok(server)
}
