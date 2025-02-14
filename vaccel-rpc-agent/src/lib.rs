// SPDX-License-Identifier: Apache-2.0

use agent_service::AgentService;
use thiserror::Error as ThisError;

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
pub use cli::Cli;

#[derive(ThisError, Debug)]
pub enum Error {
    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(String),

    /// vAccel runtime error
    #[error("vAccel error: {0}")]
    VaccelError(vaccel::Error),

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
        Error::VaccelError(err)
    }
}

impl From<ttrpc::Error> for Error {
    fn from(err: ttrpc::Error) -> Self {
        Error::TtrpcError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
