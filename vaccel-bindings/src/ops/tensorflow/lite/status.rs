// SPDX-License-Identifier: Apache-2.0

use crate::{error::Status as ErrorStatus, Error, Result};
use derive_more::{From, Into};
use vaccel_rpc_proto::error::VaccelStatus;

#[derive(Debug, Clone, Copy, PartialEq, Default, From, Into)]
#[repr(transparent)]
pub struct Status(pub u8);

impl Status {
    /// # Safety
    ///
    /// `ptr` is expected to be a valid pointer to an object allocated
    /// manually or by the respective vAccel function.
    pub unsafe fn populate_ffi(&self, ptr: *mut u8) {
        if let Some(ffi) = unsafe { ptr.as_mut() } {
            *ffi = (*self).into();
        }
    }
}

impl TryFrom<VaccelStatus> for Status {
    type Error = Error;

    fn try_from(status: VaccelStatus) -> Result<Self> {
        Ok(Status(
            status
                .code
                .try_into()
                .map_err(|e| Error::ConversionFailed(format!("{}", e)))?,
        ))
    }
}

impl From<Status> for VaccelStatus {
    fn from(status: Status) -> Self {
        let mut vaccel_status = Self::new();
        vaccel_status.code = u8::from(status) as u32;
        vaccel_status
    }
}

impl From<ErrorStatus> for Status {
    fn from(status: ErrorStatus) -> Self {
        Status(status.code)
    }
}

impl From<&ErrorStatus> for Status {
    fn from(status: &ErrorStatus) -> Self {
        Status(status.code)
    }
}

impl From<Status> for ErrorStatus {
    fn from(status: Status) -> Self {
        Self::new(status.into(), "")
    }
}
