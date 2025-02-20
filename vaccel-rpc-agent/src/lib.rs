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
    #[error("vAccel error: {0}")]
    Vaccel(#[from] vaccel::Error),

    #[error("ttrpc error: {0}")]
    Ttrpc(#[from] ttrpc::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Agent not running")]
    NotRunning,

    #[error("Agent already running")]
    AlreadyRunning,

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
