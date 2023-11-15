pub mod cli;
pub mod genop;
pub mod image;
pub mod resources;
#[cfg(feature = "async")]
pub mod rpc_async;
#[cfg(not(feature = "async"))]
pub mod rpc_sync;
pub mod session;
pub mod tf_model;
pub mod torch_model;

use dashmap::DashMap;
#[cfg(feature = "async")]
use protocols::asynchronous::agent_ttrpc::{create_vaccel_agent, VaccelAgent};
#[cfg(not(feature = "async"))]
use protocols::sync::agent_ttrpc::{create_vaccel_agent, VaccelAgent};
use protocols::{
    error::VaccelError,
    profiling::{ProfilingRequest, ProfilingResponse},
};
use std::{default::Default, error::Error, sync::Arc};
#[cfg(feature = "async")]
use ttrpc::asynchronous::Server;
#[cfg(not(feature = "async"))]
use ttrpc::sync::Server;
use vaccel::{profiling::ProfRegions, Resource, Session, VaccelId};

#[derive(Clone)]
pub struct Agent {
    pub sessions: Arc<DashMap<VaccelId, Box<Session>>>,
    pub resources: Arc<DashMap<VaccelId, Box<dyn Resource>>>,
    pub timers: Arc<DashMap<u32, ProfRegions>>,
}

unsafe impl Sync for Agent {}
unsafe impl Send for Agent {}

pub(crate) fn ttrpc_error(code: ttrpc::Code, msg: String) -> ttrpc::Error {
    ttrpc::Error::RpcStatus(ttrpc::error::get_status(code, msg))
}

pub(crate) fn vaccel_error(err: vaccel::Error) -> VaccelError {
    let mut grpc_error = VaccelError::new();

    match err {
        vaccel::Error::Runtime(e) => grpc_error.set_vaccel_error(e as i64),
        vaccel::Error::InvalidArgument => grpc_error.set_agent_error(1i64),
        vaccel::Error::Uninitialized => grpc_error.set_agent_error(2i64),
        vaccel::Error::TensorFlow(_) => grpc_error.set_agent_error(3i64),
        vaccel::Error::Torch(_) => grpc_error.set_agent_error(4i64),
    }

    grpc_error
}

pub fn server_init(server_address: &str) -> Result<Server, Box<dyn Error>> {
    let vaccel_agent = Box::new(Agent {
        sessions: Arc::new(DashMap::new()),
        resources: Arc::new(DashMap::new()),
        timers: Arc::new(DashMap::new()),
    }) as Box<dyn VaccelAgent + Send + Sync>;

    let agent_worker = Arc::new(vaccel_agent);
    let aservice = create_vaccel_agent(agent_worker);

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

impl Agent {
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
