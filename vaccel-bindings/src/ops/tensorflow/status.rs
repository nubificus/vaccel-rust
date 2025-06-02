// SPDX-License-Identifier: Apache-2.0

use crate::{error::Status as ErrorStatus, ffi, Error, Result};
use derive_more::Display;
use std::ffi::{CStr, CString};
use vaccel_rpc_proto::error::VaccelStatus;

#[derive(Debug, Default, Display, Clone)]
#[display("{} ({})", self.message(), self.error_code())]
pub struct Status {
    inner: ffi::vaccel_tf_status,
}

impl Status {
    pub fn new(error_code: u8, message: &str) -> Result<Self> {
        let mut inner = ffi::vaccel_tf_status::default();
        let c_message = CString::new(message).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `message` to `CString` [{}]", e))
        })?;

        match unsafe {
            ffi::vaccel_tf_status_init(&mut inner, error_code, c_message.as_c_str().as_ptr()) as u32
        } {
            ffi::VACCEL_OK => Ok(Self { inner }),
            err => Err(Error::Ffi(err)),
        }
    }

    pub fn error_code(&self) -> u8 {
        self.inner.error_code
    }

    pub fn message(&self) -> String {
        if self.inner.message.is_null() {
            return String::new();
        }

        let c_message = unsafe { CStr::from_ptr(self.inner.message) };
        c_message.to_str().unwrap_or("").to_owned()
    }

    /// # Safety
    ///
    /// `ptr` is expected to be a valid pointer to an object allocated
    /// manually or by the respective vAccel function.
    pub unsafe fn populate_ffi(&self, ptr: *mut ffi::vaccel_tf_status) {
        if let Some(ffi) = unsafe { ptr.as_mut() } {
            ffi.error_code = self.inner.error_code;
            ffi.message = self.inner.message;
        }
    }

    pub fn is_ok(&self) -> bool {
        self.error_code() == 0
    }

    pub(crate) fn inner(&self) -> &ffi::vaccel_tf_status {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_tf_status {
        &mut self.inner
    }
}

impl TryFrom<VaccelStatus> for Status {
    type Error = Error;

    fn try_from(status: VaccelStatus) -> Result<Self> {
        Self::new(
            status.code.try_into().map_err(|e| {
                Error::ConversionFailed(format!("Could not convert `status.code` to `u8` [{}]", e))
            })?,
            &status.message,
        )
    }
}

impl From<Status> for VaccelStatus {
    fn from(status: Status) -> Self {
        let mut vaccel_status = Self::new();
        vaccel_status.code = status.error_code() as u32;
        vaccel_status.message = status.message();
        vaccel_status
    }
}

impl TryFrom<ErrorStatus> for Status {
    type Error = Error;

    fn try_from(status: ErrorStatus) -> Result<Self> {
        Self::new(status.code, &status.message)
    }
}

impl TryFrom<&ErrorStatus> for Status {
    type Error = Error;

    fn try_from(status: &ErrorStatus) -> Result<Self> {
        Self::new(status.code, &status.message)
    }
}

impl From<Status> for ErrorStatus {
    fn from(status: Status) -> Self {
        Self::new(status.error_code(), &status.message())
    }
}
