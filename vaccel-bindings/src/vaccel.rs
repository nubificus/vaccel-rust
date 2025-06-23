// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Config, Error, Handle, Result};
use derive_more::Display;

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
