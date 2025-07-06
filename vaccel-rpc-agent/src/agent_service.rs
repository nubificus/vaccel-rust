// SPDX-License-Identifier: Apache-2.0

use dashmap::DashMap;
use protobuf::Message;
use std::sync::Arc;
use thiserror::Error as ThisError;
use vaccel::{self, profiling::ProfilerManager, Resource, Session, VaccelId};
use vaccel_rpc_proto::{
    error::VaccelError,
    profiling::{ProfilingRequest, ProfilingResponse},
};

#[derive(ThisError, Debug)]
pub enum AgentServiceError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Vaccel error: {0}")]
    Vaccel(#[from] vaccel::Error),
}

impl From<AgentServiceError> for ttrpc::Error {
    fn from(e: AgentServiceError) -> Self {
        match e {
            AgentServiceError::InvalidArgument(s) => {
                ttrpc::error::get_rpc_status(ttrpc::Code::INVALID_ARGUMENT, s.clone())
            }
            AgentServiceError::NotFound(s) => {
                ttrpc::error::get_rpc_status(ttrpc::Code::NOT_FOUND, s.clone())
            }
            AgentServiceError::Internal(s) => {
                ttrpc::error::get_rpc_status(ttrpc::Code::INTERNAL, s.clone())
            }
            AgentServiceError::Vaccel(e) => {
                let mut ttrpc_status =
                    ttrpc::error::get_status(ttrpc::Code::INTERNAL, e.to_string());
                let vaccel_error = VaccelError::from(e);

                let details = vaccel_error.write_to_bytes().unwrap();
                let mut any = ttrpc::proto::Any::new();
                any.set_type_url("type.googleapis.com/vaccel.VaccelError".to_string());
                any.set_value(details);

                ttrpc_status.set_details(vec![any]);

                ttrpc::Error::RpcStatus(ttrpc_status)
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, AgentServiceError>;

pub trait IntoTtrpcResult<T> {
    fn into_ttrpc(self) -> ttrpc::Result<T>;
}

impl<T> IntoTtrpcResult<T> for Result<T> {
    fn into_ttrpc(self) -> ttrpc::Result<T> {
        self.map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
pub struct AgentService {
    pub(crate) sessions: Arc<DashMap<VaccelId, Box<Session>>>,
    pub(crate) resources: Arc<DashMap<VaccelId, Box<Resource>>>,
    pub(crate) profiler_manager: ProfilerManager,
}

unsafe impl Sync for AgentService {}
unsafe impl Send for AgentService {}

impl AgentService {
    pub const TIMERS_PREFIX: &'static str = "vaccel-rpc-agent";

    pub(crate) fn new() -> Self {
        AgentService {
            sessions: Arc::new(DashMap::new()),
            resources: Arc::new(DashMap::new()),
            profiler_manager: ProfilerManager::new(Self::TIMERS_PREFIX),
        }
    }

    pub(crate) fn do_get_profiler(&self, req: ProfilingRequest) -> Result<ProfilingResponse> {
        let mut resp = ProfilingResponse::new();
        resp.profiler = self
            .profiler_manager
            .get(req.session_id.into())
            .map(|p| p.clone().into())
            .into();
        Ok(resp)
    }
}

impl AsRef<ProfilerManager> for AgentService {
    fn as_ref(&self) -> &ProfilerManager {
        &self.profiler_manager
    }
}
