// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use thiserror::Error;
use vaccel_rpc_proto::error::{vaccel_error::Error as VaccelErrorType, VaccelError};

#[cfg(feature = "async")]
pub mod asynchronous;
#[cfg(not(feature = "async"))]
pub mod sync;
#[cfg(feature = "async")]
pub use asynchronous as r#async;
pub mod client;
pub mod ops;
pub mod profiling;
pub mod resource;
pub mod session;
pub mod util;

extern crate ttrpc;

#[derive(Error, Debug)]
pub enum Error {
    /// Client error
    #[error("Client error: {0}")]
    ClientError(u32),

    /// Socket error
    #[error("ttprc error: {0}")]
    TtrpcError(ttrpc::Error),

    /// Async error
    #[cfg(feature = "async")]
    #[error("Async error: {0}")]
    AsyncError(tokio::task::JoinError),

    /// Host error
    #[error("Host vAccel error: {0}")]
    HostRuntimeError(u32),

    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(u32),

    /// Invalid argument error
    #[error("Invalid argument")]
    InvalidArgument,

    /// Undefined error
    #[error("Undefined error")]
    Undefined,
}

impl From<ttrpc::Error> for Error {
    fn from(err: ttrpc::Error) -> Self {
        Error::TtrpcError(err)
    }
}

#[cfg(feature = "async")]
impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::AsyncError(err)
    }
}

impl From<vaccel_rpc_proto::error::VaccelError> for Error {
    fn from(err: VaccelError) -> Self {
        match err.error {
            Some(VaccelErrorType::VaccelError(err)) => Error::HostRuntimeError(err as u32),
            Some(VaccelErrorType::AgentError(err)) => Error::AgentError(err as u32),
            Some(_) => Error::Undefined,
            None => Error::Undefined,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
