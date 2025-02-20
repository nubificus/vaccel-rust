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
    #[error("vAccel error: {0}")]
    Vaccel(#[from] vaccel::Error),

    #[error("ttprc error: {0}")]
    Ttrpc(ttrpc::Error),

    #[cfg(feature = "async")]
    #[error("Async error: {0}")]
    Async(#[from] tokio::task::JoinError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Host vAccel error: {0}")]
    HostVaccel(vaccel::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Error: {0}")]
    Other(String),

    #[error("Unknown error")]
    Unknown,
}

impl Error {
    pub fn to_ffi(&self) -> u32 {
        match self {
            Error::HostVaccel(e) => match e {
                vaccel::Error::Ffi(error) => *error,
                vaccel::Error::FfiWithStatus { error, .. } => *error,
                _ => ffi::VACCEL_EBACKEND,
            },
            Error::Ttrpc(_) => ffi::VACCEL_EIO,
            _ => ffi::VACCEL_EBACKEND,
        }
    }
}

impl From<ttrpc::Error> for Error {
    fn from(err: ttrpc::Error) -> Self {
        if let ttrpc::Error::RpcStatus(ref rpc_status) = err {
            let details = rpc_status.details();
            if !details.is_empty() {
                if let Ok(vaccel_error) = VaccelError::parse_from_bytes(details[0].value()) {
                    return Error::HostVaccel(vaccel_error.into());
                }
            }
        }

        Error::Ttrpc(err)
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
