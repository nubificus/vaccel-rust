// SPDX-License-Identifier: Apache-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

use std::{fmt, slice};

pub mod ffi;
pub mod file;
pub mod ops;
pub mod profiling;
pub mod resource;
pub mod session;

pub use file::File;
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

    // A TensorFlow error
    #[cfg(target_pointer_width = "64")]
    TensorFlow(ops::tensorflow::Code),

    // A TensorFlow Lite error
    TensorFlowLite(ops::tensorflow::lite::Code),

    // A PyTorch error
    Torch(ops::torch::Code),

    // Other error types
    Others(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Runtime(e) => write!(f, "vAccel runtime error: {}", e),
            Error::InvalidArgument => write!(f, "Invalid argument"),
            Error::Uninitialized => write!(f, "Uninitialized vAccel object"),
            #[cfg(target_pointer_width = "64")]
            Error::TensorFlow(e) => write!(f, "TensorFlow error: {:?}", e),
            Error::TensorFlowLite(e) => write!(f, "TensorFlow Lite error: {:?}", e),
            Error::Torch(e) => write!(f, "Torch error: {:?}", e),
            Error::Others(e) => write!(f, "Error: {}", e),
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
        id.inner.unwrap_or(0)
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

/// # Safety
///
/// `buf` must be a valid pointer to an array of objects of type `T` with the provided len.
/// See also: https://doc.rust-lang.org/std/vec/struct.Vec.html#safety
pub unsafe fn c_pointer_to_vec<T>(buf: *mut T, len: usize, capacity: usize) -> Option<Vec<T>> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { Vec::from_raw_parts(buf, len, capacity) })
    }
}

/// # Safety
///
/// `buf` must be a valid pointer to an array of objects of type `T` with the provided len.
/// See also: https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety
pub unsafe fn c_pointer_to_slice<'a, T>(buf: *const T, len: usize) -> Option<&'a [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(buf, len) })
    }
}

/// # Safety
///
/// `buf` must be a valid pointer to an array of objects of type `T` with the provided len.
/// See also: https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety
pub unsafe fn c_pointer_to_mut_slice<'a, T>(buf: *mut T, len: usize) -> Option<&'a mut [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts_mut(buf, len) })
    }
}
