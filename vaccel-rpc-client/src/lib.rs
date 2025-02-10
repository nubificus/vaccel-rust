// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use log::error;
use protobuf::Message;
use std::ffi::c_int;
use thiserror::Error;
use vaccel::ffi;
use vaccel_rpc_proto::error::{VaccelError, VaccelErrorType};

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

extern crate ttrpc;

#[derive(Error, Debug)]
pub enum Error {
    /// Client error
    #[error("Client error: {0}")]
    ClientError(String),

    /// Socket error
    #[error("ttprc error: {0}")]
    TtrpcError(ttrpc::Error),

    /// Async error
    #[cfg(feature = "async")]
    #[error("Async error: {0}")]
    AsyncError(tokio::task::JoinError),

    /// vAccel error
    #[error("vAccel error: {0}")]
    VaccelError(vaccel::Error),

    /// Host vAccel runtime error
    #[error("Host vAccel error: {0}")]
    HostRuntimeError(u32),

    /// Agent error
    #[error("Agent error: {0}")]
    AgentError(String),

    /// Invalid argument error
    #[error("Invalid argument")]
    InvalidArgument,

    /// Undefined error
    #[error("Undefined error")]
    Undefined,

    /// Other errors
    #[error("Error: {0}")]
    Others(String),
}

impl Error {
    pub fn to_ffi(&self) -> u32 {
        match self {
            Error::HostRuntimeError(e) => *e,
            Error::ClientError(_) => ffi::VACCEL_EBACKEND,
            _ => ffi::VACCEL_EIO,
        }
    }
}

impl From<vaccel::Error> for Error {
    fn from(err: vaccel::Error) -> Self {
        Error::VaccelError(err)
    }
}

impl From<ttrpc::Error> for Error {
    fn from(err: ttrpc::Error) -> Self {
        if let ttrpc::Error::RpcStatus(ref rpc_status) = err {
            let details = rpc_status.details();
            if !details.is_empty() {
                if let Ok(vaccel_error) = VaccelError::parse_from_bytes(details[0].value()) {
                    return Error::HostRuntimeError(vaccel_error.ffi_error);
                }
            }
        }

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
        match err.type_.enum_value() {
            Ok(VaccelErrorType::RUNTIME) => Error::HostRuntimeError(err.ffi_error),
            Ok(_) => Error::Others(err.to_string()),
            _ => Error::Undefined,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait SealedFfiResult {}

pub trait IntoFfiResult {
    type FfiType;
    fn into_ffi(self) -> Self::FfiType;
}

impl IntoFfiResult for Result<i64> {
    type FfiType = ffi::vaccel_id_t;

    fn into_ffi(self) -> Self::FfiType {
        match self {
            Ok(r) => r,
            Err(e) => {
                error!("{}", e);
                -(e.to_ffi() as Self::FfiType)
            }
        }
    }
}

impl<T> IntoFfiResult for Result<T>
where
    T: SealedFfiResult,
{
    type FfiType = c_int;

    fn into_ffi(self) -> Self::FfiType {
        (match self {
            Ok(_) => ffi::VACCEL_OK,
            Err(e) => {
                error!("{}", e);
                e.to_ffi()
            }
        }) as Self::FfiType
    }
}

impl SealedFfiResult for () {}
impl SealedFfiResult for Vec<u8> {}
