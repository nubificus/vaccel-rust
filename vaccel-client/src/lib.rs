#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use protocols::error::{VaccelError, VaccelError_oneof_error};

pub mod client;
pub mod image;
pub mod genop;
pub mod resources;
pub mod session;
pub mod tf_model;
pub mod torch_model;
pub mod util;
pub mod shared_obj;

extern crate ttrpc;

#[derive(Debug)]
pub enum Error {
    /// VSock client error
    ClientError(u32),

    /// Socket Error
    TtrpcError(ttrpc::Error),

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

impl From<protocols::error::VaccelError> for Error {
    fn from(err: VaccelError) -> Self {
        match err.error {
            Some(VaccelError_oneof_error::vaccel_error(err)) => Error::HostRuntimeError(err as u32),
            Some(VaccelError_oneof_error::agent_error(err)) => Error::HostAgentError(err as u32),
            None => Error::Undefined,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
