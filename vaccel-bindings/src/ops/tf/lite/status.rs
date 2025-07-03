// SPDX-License-Identifier: Apache-2.0

use crate::{error::Status as ErrorStatus, Error, Result};
use derive_more::{From, Into};
use vaccel_rpc_proto::vaccel::Status as ProtoStatus;

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

impl TryFrom<&ProtoStatus> for Status {
    type Error = Error;

    fn try_from(proto_status: &ProtoStatus) -> Result<Self> {
        Ok(Status(
            proto_status
                .code
                .try_into()
                .map_err(|e| Error::ConversionFailed(format!("{}", e)))?,
        ))
    }
}

impl TryFrom<ProtoStatus> for Status {
    type Error = Error;

    fn try_from(proto_status: ProtoStatus) -> Result<Self> {
        Status::try_from(&proto_status)
    }
}

impl From<&Status> for ProtoStatus {
    fn from(status: &Status) -> Self {
        ProtoStatus {
            code: u8::from(*status) as u32,
            ..Default::default()
        }
    }
}

impl From<Status> for ProtoStatus {
    fn from(status: Status) -> Self {
        ProtoStatus {
            code: u8::from(status) as u32,
            ..Default::default()
        }
    }
}

impl From<&ErrorStatus> for Status {
    fn from(error_status: &ErrorStatus) -> Self {
        Status(error_status.code)
    }
}

impl From<ErrorStatus> for Status {
    fn from(error_status: ErrorStatus) -> Self {
        Status::from(&error_status)
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
