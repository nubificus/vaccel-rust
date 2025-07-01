// SPDX-License-Identifier: Apache-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use derive_more::Display;
use std::slice;

#[macro_use]
pub mod handle;
pub use handle::Handle;

pub mod arg;
pub mod blob;
pub mod config;
pub mod error;
pub mod ffi;
pub mod ops;
pub mod profiling;
pub mod resource;
pub mod session;

pub use arg::Arg;
pub use blob::Blob;
pub use config::Config;
pub use error::{Error, Result};
pub use resource::{Resource, ResourceType};
pub use session::Session;

/// Wrapper for the `vaccel_id_t` C object.
#[derive(PartialEq, Eq, Hash, Debug, Default, Display)]
#[display("{}", inner.map_or("No value".to_string(), |v| v.to_string()))]
pub struct VaccelId {
    inner: Option<ffi::vaccel_id_t>,
}

impl VaccelId {
    /// Returns `true` if the `VaccelId` holds an actual ID.
    pub fn has_id(&self) -> bool {
        self.inner.is_some()
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

/// Wrapper for `slice::from_raw_parts()` with null pointer checking.
///
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

/// Wrapper for `slice::from_raw_parts_mut()` with null pointer checking.
///
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

/// Bootstraps the vAccel library using the provided config.
pub fn bootstrap_with_config(config: &mut Config) -> Result<()> {
    match unsafe { ffi::vaccel_bootstrap_with_config(config.as_mut_ptr()) as u32 } {
        ffi::VACCEL_OK => Ok(()),
        err => Err(Error::Ffi(err)),
    }
}

/// Bootstraps the vAccel library.
pub fn bootstrap() -> Result<()> {
    match unsafe { ffi::vaccel_bootstrap() as u32 } {
        ffi::VACCEL_OK => Ok(()),
        err => Err(Error::Ffi(err)),
    }
}

/// Perfoms cleanup for the vAccel library.
pub fn cleanup() -> Result<()> {
    match unsafe { ffi::vaccel_cleanup() as u32 } {
        ffi::VACCEL_OK => Ok(()),
        err => Err(Error::Ffi(err)),
    }
}

/// Returns `true` if the vAccel library is initialized.
pub fn is_initialized() -> bool {
    unsafe { ffi::vaccel_is_initialized() }
}
