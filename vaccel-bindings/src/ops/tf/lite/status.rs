// SPDX-License-Identifier: Apache-2.0

use crate::{error::Status as ErrorStatus, Error, Result};
use derive_more::{From, Into};

/// Wrapper for the TFLite status.
#[derive(Debug, Clone, Copy, PartialEq, Default, From, Into)]
#[repr(transparent)]
pub struct Status(pub u8);

impl Status {
    /// Populates a pointer of the wrapped C type with the `Status` data.
    ///
    /// # Safety
    ///
    /// `ptr` is expected to be a valid pointer to an object allocated
    /// manually or by the respective vAccel function.
    pub unsafe fn populate_ptr(&self, ptr: *mut u8) -> Result<()> {
        let ffi = unsafe { ptr.as_mut().ok_or(Error::EmptyValue)? };
        *ffi = (*self).into();
        Ok(())
    }
}

impl From<&ErrorStatus> for Status {
    fn from(error_status: &ErrorStatus) -> Self {
        Self(error_status.code)
    }
}

impl From<ErrorStatus> for Status {
    fn from(error_status: ErrorStatus) -> Self {
        Self::from(&error_status)
    }
}

impl From<&Status> for ErrorStatus {
    fn from(status: &Status) -> Self {
        Self::new((*status).into(), "")
    }
}

impl From<Status> for ErrorStatus {
    fn from(status: Status) -> Self {
        Self::new(status.into(), "")
    }
}
