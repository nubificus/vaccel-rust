// SPDX-License-Identifier: Apache-2.0

pub mod agent;
mod agent_service;
#[cfg(feature = "async")]
mod asynchronous;
pub mod cli;
mod ops;
mod resource;
mod session;
#[cfg(not(feature = "async"))]
mod sync;

pub use agent::Agent;
use agent_service::AgentService;
pub use cli::Cli;

use thiserror::Error as ThisError;
use vaccel_rpc_proto::error::VaccelError;

#[derive(ThisError, Debug)]
pub enum Error {
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(String),

    /// vAccel runtime error
    #[error("vAccel error: {0}")]
    RuntimeError(String),

    /// Socket error
    #[error("ttrpc error: {0}")]
    TtrpcError(ttrpc::Error),

    /// CLI error
    #[error("CLI error: {0}")]
    CliError(String),

    /// Undefined error
    #[error("Undefined error")]
    Undefined,
}

impl From<vaccel::Error> for Error {
    fn from(err: vaccel::Error) -> Self {
        Error::RuntimeError(format!("{}", err))
    }
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
