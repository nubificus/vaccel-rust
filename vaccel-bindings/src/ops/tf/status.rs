// SPDX-License-Identifier: Apache-2.0

use crate::{error::Status as ErrorStatus, ffi, Error, Handle, Result};
use derive_more::Display;
use std::{
    ffi::{CStr, CString},
    ptr::{self, NonNull},
};
use vaccel_rpc_proto::vaccel::Status as ProtoStatus;

/// Wrapper for the `struct vaccel_tf_status` C object.
#[derive(Debug, Display)]
#[display("{} ({})", self.message().unwrap_or("".to_string()), self.code())]
pub struct Status {
    inner: NonNull<ffi::vaccel_tf_status>,
    owned: bool,
}

impl Status {
    /// Creates a new `Status`.
    pub fn new(code: u8, message: &str) -> Result<Self> {
        let c_message = CString::new(message).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `message` to `CString` [{}]", e))
        })?;

        let mut ptr: *mut ffi::vaccel_tf_status = ptr::null_mut();
        match unsafe { ffi::vaccel_tf_status_new(&mut ptr, code, c_message.as_ptr()) as u32 } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        unsafe { Self::from_ptr_owned(ptr) }
    }

    /// Returns the code of the `Status`.
    pub fn code(&self) -> u8 {
        unsafe { self.inner.as_ref().code }
    }

    /// Returns the message of the `Status`.
    pub fn message(&self) -> Result<String> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.message.is_null() {
            return Err(Error::EmptyValue);
        }

        match unsafe { CStr::from_ptr(inner.message).to_str() } {
            Ok(n) => Ok(n.to_string()),
            Err(e) => Err(Error::ConversionFailed(format!(
                "Could not convert `message` to `CString` [{}]",
                e
            ))),
        }
    }

    /// Populates a pointer of the wrapped C type with the `Status` data.
    ///
    /// # Safety
    ///
    /// `ptr` is expected to be a valid pointer to an object allocated
    /// manually or by the respective vAccel function.
    pub unsafe fn populate_ptr(&self, ptr: *mut ffi::vaccel_tf_status) -> Result<()> {
        let ffi = unsafe { ptr.as_mut().ok_or(Error::EmptyValue)? };

        ffi.code = self.inner.as_ref().code;
        unsafe {
            ffi.message = libc::strdup(self.inner.as_ref().message);
            if ffi.message.is_null() {
                return Err(Error::EmptyValue);
            }
        }

        Ok(())
    }

    /// Returns `true` if the status is OK.
    pub fn is_ok(&self) -> bool {
        self.code() == 0
    }
}

impl_component_drop!(Status, vaccel_tf_status_delete, inner, owned);

impl_component_handle!(Status, ffi::vaccel_tf_status, inner, owned);

impl TryFrom<&ProtoStatus> for Status {
    type Error = Error;

    fn try_from(proto_status: &ProtoStatus) -> Result<Self> {
        Self::new(
            proto_status.code.try_into().map_err(|e| {
                Error::ConversionFailed(format!("Could not convert `status.code` to `u8` [{}]", e))
            })?,
            &proto_status.message,
        )
    }
}

impl TryFrom<ProtoStatus> for Status {
    type Error = Error;

    fn try_from(proto_status: ProtoStatus) -> Result<Self> {
        Status::try_from(&proto_status)
    }
}

impl TryFrom<&Status> for ProtoStatus {
    type Error = Error;

    fn try_from(status: &Status) -> Result<Self> {
        Ok(ProtoStatus {
            code: status.code().into(),
            message: status.message()?,
            ..Default::default()
        })
    }
}

impl TryFrom<Status> for ProtoStatus {
    type Error = Error;

    fn try_from(status: Status) -> Result<Self> {
        ProtoStatus::try_from(&status)
    }
}

impl TryFrom<&ErrorStatus> for Status {
    type Error = Error;

    fn try_from(error_status: &ErrorStatus) -> Result<Self> {
        Self::new(error_status.code, &error_status.message)
    }
}

impl TryFrom<ErrorStatus> for Status {
    type Error = Error;

    fn try_from(error_status: ErrorStatus) -> Result<Self> {
        Status::try_from(&error_status)
    }
}

impl TryFrom<&Status> for ErrorStatus {
    type Error = Error;

    fn try_from(status: &Status) -> Result<Self> {
        Ok(Self::new(status.code(), &status.message()?))
    }
}

impl TryFrom<Status> for ErrorStatus {
    type Error = Error;

    fn try_from(status: Status) -> Result<Self> {
        ErrorStatus::try_from(&status)
    }
}
