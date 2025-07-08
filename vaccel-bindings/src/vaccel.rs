// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Config, Error, Handle, Result};
use derive_more::Display;
use std::num::NonZeroI64;

/// Wrapper for the `vaccel_id_t` C object.
#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
#[display("{}", self.0)]
pub struct VaccelId(NonZeroI64);

impl VaccelId {
    /// Creates a new `VaccelId` from a positive integer.
    pub fn new(value: i64) -> Result<Self> {
        Ok(VaccelId(NonZeroI64::new(value).ok_or(
            Error::InvalidArgument("ID must be positive".to_string()),
        )?))
    }

    /// Creates a new `VaccelId` from an FFI value, mapping 0 (uninitialized)
    /// to None.
    pub fn from_ffi(value: ffi::vaccel_id_t) -> Result<Option<Self>> {
        match value {
            0 => Ok(None),
            x if x > 0 => Ok(Some(VaccelId(NonZeroI64::new(x).unwrap()))),
            _ => Err(Error::InvalidArgument(format!("Invalid ID: {}", value))),
        }
    }

    /// Returns the contained value as an integer.
    pub fn get(&self) -> i64 {
        self.0.get()
    }
}

impl TryFrom<i64> for VaccelId {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        if value > 0 {
            Ok(VaccelId(NonZeroI64::new(value).unwrap()))
        } else {
            Err(Error::InvalidArgument(format!("Invalid ID: {}", value)))
        }
    }
}

impl From<VaccelId> for i64 {
    fn from(id: VaccelId) -> Self {
        id.get()
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

/// Performs cleanup for the vAccel library.
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
