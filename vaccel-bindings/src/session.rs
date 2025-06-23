// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Handle, Result, VaccelId};
use log::warn;
use std::ptr::{self, NonNull};

#[derive(Debug)]
pub struct Session {
    inner: NonNull<ffi::vaccel_session>,
    owned: bool,
}

impl Session {
    /// Creates a new `Session`.
    pub fn new(flags: u32) -> Result<Self> {
        let mut ptr: *mut ffi::vaccel_session = ptr::null_mut();
        match unsafe { ffi::vaccel_session_new(&mut ptr, flags) as u32 } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        unsafe { Self::from_ptr_owned(ptr) }
    }

    /// Returns the ID of the `Session`.
    pub fn id(&self) -> VaccelId {
        VaccelId::from(unsafe { self.inner.as_ref().id })
    }

    /// Returns `true` if the `Session` has been initialized.
    pub fn initialized(&self) -> bool {
        self.id().has_id()
    }

    /// Updates the hint flags for the 'Session`.
    pub fn update(&mut self, flags: u32) {
        unsafe { self.inner.as_mut().hint = flags };
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.owned && self.initialized() {
            let ret = unsafe { ffi::vaccel_session_delete(self.inner.as_ptr()) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete Session inner: {}", ret);
            }
        }
    }
}

impl_component_handle!(Session, ffi::vaccel_session, inner, owned);
