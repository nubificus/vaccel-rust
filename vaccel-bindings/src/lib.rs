#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

use std::fmt;

pub mod ffi;
pub mod ops;
pub mod resource;
pub mod session;
pub mod tensorflow;

pub use resource::Resource;
pub use session::Session;

#[derive(Debug)]
pub enum Error {
    // Error returned to us by vAccel runtime library
    Runtime(u32),

    // We received an invalid argument
    InvalidArgument,

    // Uninitialized vAccel object
    Uninitialized,

    // A TensorFlow Error
    TensorFlow(tensorflow::Code),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Runtime(err) => write!(f, "vAccel runtime error {}", err),
            Error::InvalidArgument => write!(f, "An invalid argument was given to us"),
            Error::Uninitialized => write!(f, "Uninitialized vAccel object"),
            Error::TensorFlow(code) => write!(f, "TensorFlow error: {:?}", code),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct VaccelId {
    inner: Option<ffi::vaccel_id_t>,
}

impl VaccelId {
    fn has_id(&self) -> bool {
        self.inner.is_some()
    }
}

impl fmt::Display for VaccelId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            None => write!(f, "'Uninitialized object'"),
            Some(id) => write!(f, "{}", id),
        }
    }
}

impl From<ffi::vaccel_id_t> for VaccelId {
    fn from(id: ffi::vaccel_id_t) -> Self {
        if id <= 0 {
            VaccelId { inner: None }
        } else {
            VaccelId { inner: Some(id) }
        }
    }
}

impl From<VaccelId> for ffi::vaccel_id_t {
    fn from(id: VaccelId) -> Self {
        match id.inner {
            None => 0,
            Some(id) => id,
        }
    }
}

impl From<VaccelId> for u32 {
    fn from(id: VaccelId) -> Self {
        match id.inner {
            None => 0,
            Some(id) => id as u32,
        }
    }
}

impl From<u32> for VaccelId {
    fn from(id: u32) -> Self {
        VaccelId {
            inner: Some(id.into()),
        }
    }
}
