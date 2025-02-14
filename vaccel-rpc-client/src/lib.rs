// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use log::error;
use protobuf::Message;
use std::ffi::c_int;
use thiserror::Error;
use vaccel::ffi;
use vaccel_rpc_proto::error::VaccelError;

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
    AsyncError(#[from] tokio::task::JoinError),

    /// vAccel error
    #[error("vAccel error: {0}")]
    VaccelError(#[from] vaccel::Error),

    /// Host vAccel runtime error
    #[error("Host vAccel error: {0}")]
    HostVaccelError(vaccel::Error),

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
            Error::HostVaccelError(e) => match e {
                vaccel::Error::Ffi(error) => *error,
                vaccel::Error::FfiWithStatus { error, .. } => *error,
                _ => ffi::VACCEL_EBACKEND,
            },
            Error::ClientError(_) => ffi::VACCEL_EBACKEND,
            _ => ffi::VACCEL_EIO,
        }
    }
}

impl From<ttrpc::Error> for Error {
    fn from(err: ttrpc::Error) -> Self {
        if let ttrpc::Error::RpcStatus(ref rpc_status) = err {
            let details = rpc_status.details();
            if !details.is_empty() {
                if let Ok(vaccel_error) = VaccelError::parse_from_bytes(details[0].value()) {
                    return Error::HostVaccelError(vaccel_error.into());
                }
            }
        }

        Error::TtrpcError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait SealedFfiResult {}

impl SealedFfiResult for () {}
impl SealedFfiResult for Vec<u8> {}

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
