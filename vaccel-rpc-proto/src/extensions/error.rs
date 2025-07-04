// SPDX-License-Identifier: Apache-2.0

use crate::vaccel::{Error, ErrorType, Status};
use vaccel::{error::Status as VaccelStatus, ffi, Error as VaccelError, Result};

impl TryFrom<&Status> for VaccelStatus {
    type Error = VaccelError;

    fn try_from(status: &Status) -> Result<Self> {
        Ok(Self::new(
            status
                .code
                .try_into()
                .map_err(|e| VaccelError::ConversionFailed(format!("{}", e)))?,
            &status.message,
        ))
    }
}

impl TryFrom<Status> for VaccelStatus {
    type Error = VaccelError;

    fn try_from(status: Status) -> Result<Self> {
        Self::try_from(&status)
    }
}

impl From<&VaccelStatus> for Status {
    fn from(vaccel: &VaccelStatus) -> Self {
        Self {
            code: vaccel.code as u32,
            message: vaccel.message.to_owned(),
            ..Default::default()
        }
    }
}

impl From<VaccelStatus> for Status {
    fn from(vaccel: VaccelStatus) -> Self {
        Self {
            code: vaccel.code as u32,
            message: vaccel.message,
            ..Default::default()
        }
    }
}

impl From<Error> for VaccelError {
    fn from(err: Error) -> Self {
        match err.type_.enum_value().ok() {
            Some(ErrorType::FFI) => Self::Ffi(err.ffi_error.unwrap_or(ffi::VACCEL_EBACKEND)),
            Some(ErrorType::FFI_WITH_STATUS) => {
                let status = match err.status.into_option() {
                    Some(s) => {
                        let code = u8::try_from(s.code).unwrap_or(u8::MAX);
                        VaccelStatus::new(code, &s.message)
                    }
                    None => VaccelStatus::default(),
                };

                Self::FfiWithStatus {
                    error: err.ffi_error.unwrap_or(ffi::VACCEL_EBACKEND),
                    status,
                }
            }
            Some(ErrorType::INVALID_ARGUMENT) => {
                let message = match err.status.into_option() {
                    Some(s) => s.message,
                    None => String::new(),
                };

                Self::InvalidArgument(message)
            }
            Some(ErrorType::UNINITIALIZED) => Self::Uninitialized,
            Some(ErrorType::OUT_OF_BOUNDS) => Self::OutOfBounds,
            Some(ErrorType::EMPTY_VALUE) => Self::EmptyValue,
            Some(ErrorType::CONVERSION_FAILED) => {
                let message = match err.status.into_option() {
                    Some(s) => s.message,
                    None => String::new(),
                };

                Self::ConversionFailed(message)
            }
            None => {
                Self::ConversionFailed("Could not convert proto `Error` to `Error`".to_string())
            }
        }
    }
}

impl From<VaccelError> for Error {
    fn from(vaccel: VaccelError) -> Self {
        let mut err = Error::new();
        match vaccel {
            VaccelError::Ffi(error) => {
                err.type_ = ErrorType::FFI.into();
                err.ffi_error = Some(error);
            }
            VaccelError::FfiWithStatus { error, status } => {
                err.type_ = ErrorType::FFI_WITH_STATUS.into();
                err.ffi_error = Some(error);
                err.status = Some(status.into()).into();
            }
            VaccelError::InvalidArgument(message) => {
                err.type_ = ErrorType::INVALID_ARGUMENT.into();
                let mut status = Status::new();
                status.message = message;
                err.status = Some(status).into();
            }
            VaccelError::Uninitialized => {
                err.type_ = ErrorType::UNINITIALIZED.into();
            }
            VaccelError::OutOfBounds => {
                err.type_ = ErrorType::OUT_OF_BOUNDS.into();
            }
            VaccelError::EmptyValue => {
                err.type_ = ErrorType::EMPTY_VALUE.into();
            }
            VaccelError::ConversionFailed(message) => {
                err.type_ = ErrorType::CONVERSION_FAILED.into();
                let mut status = Status::new();
                status.message = message;
                err.status = Some(status).into();
            }
        }
        err
    }
}
