// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::slice;
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
pub mod resources;
pub mod session;
pub mod util;

extern crate ttrpc;

#[derive(Debug)]
pub enum Error {
    /// Client error
    ClientError(u32),

    /// Socket error
    TtrpcError(ttrpc::Error),

    /// Async Runtime error
    #[cfg(feature = "async")]
    AsyncRuntimeError(tokio::task::JoinError),

    /// File reading error
    FileReadingError,

    /// Host error
    HostRuntimeError(u32),

    /// Agent error
    HostAgentError(u32),

    /// Invalid argument error
    InvalidArgument,

    /// Undefined error
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
        Error::AsyncRuntimeError(err)
    }
}

impl From<vaccel_rpc_proto::error::VaccelError> for Error {
    fn from(err: VaccelError) -> Self {
        match err.error {
            Some(VaccelErrorType::VaccelError(err)) => Error::HostRuntimeError(err as u32),
            Some(VaccelErrorType::AgentError(err)) => Error::HostAgentError(err as u32),
            Some(_) => Error::Undefined,
            None => Error::Undefined,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn c_pointer_to_vec<T>(buf: *mut T, len: usize, capacity: usize) -> Option<Vec<T>> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { Vec::from_raw_parts(buf, len, capacity) })
    }
}

pub(crate) fn c_pointer_to_slice<'a, T>(buf: *const T, len: usize) -> Option<&'a [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(buf, len) })
    }
}

pub(crate) fn c_pointer_to_mut_slice<'a, T>(buf: *mut T, len: usize) -> Option<&'a mut [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts_mut(buf, len) })
    }
}
