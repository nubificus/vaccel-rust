// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use derive_more::Display;
use thiserror::Error as ThisError;
use vaccel_rpc_proto::error::{VaccelError, VaccelErrorType, VaccelStatus};

#[derive(Debug, Clone, Default, Display)]
#[display("{} ({})", message, code)]
pub struct Status {
    pub code: u8,
    pub message: String,
}

impl Status {
    pub fn new(code: u8, message: &str) -> Self {
        Status {
            code,
            message: message.to_string(),
        }
    }
}

impl From<Status> for VaccelStatus {
    fn from(status: Status) -> Self {
        let mut vaccel_status = Self::new();
        vaccel_status.code = status.code as u32;
        vaccel_status.message = status.message;
        vaccel_status
    }
}

impl TryFrom<VaccelStatus> for Status {
    type Error = Error;

    fn try_from(status: VaccelStatus) -> Result<Self> {
        Ok(Self::new(
            status
                .code
                .try_into()
                .map_err(|e| Error::ConversionFailed(format!("{}", e)))?,
            &status.message,
        ))
    }
}

#[derive(ThisError, Debug, Clone)]
pub enum Error {
    #[error("FFI error: {0}")]
    Ffi(u32),

    #[error("FFI error: {error} [{status}]")]
    FfiWithStatus { error: u32, status: Status },

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Uninitialized object")]
    Uninitialized,

    #[error("Index out of bounds")]
    OutOfBounds,

    #[error("Empty value")]
    EmptyValue,

    #[error("Conversion failed: {0}")]
    ConversionFailed(String),
}

impl Error {
    pub fn get_status(&self) -> Option<&Status> {
        match self {
            Error::FfiWithStatus { status, .. } => Some(status),
            _ => None,
        }
    }
}

impl From<VaccelError> for Error {
    fn from(err: VaccelError) -> Self {
        match err.type_.enum_value().ok() {
            Some(VaccelErrorType::FFI) => Self::Ffi(err.ffi_error.unwrap_or(ffi::VACCEL_EBACKEND)),
            Some(VaccelErrorType::FFI_WITH_STATUS) => {
                let status = match err.status.into_option() {
                    Some(s) => {
                        let code = u8::try_from(s.code).unwrap_or(u8::MAX);
                        Status::new(code, &s.message)
                    }
                    None => Status::default(),
                };

                Self::FfiWithStatus {
                    error: err.ffi_error.unwrap_or(ffi::VACCEL_EBACKEND),
                    status,
                }
            }
            Some(VaccelErrorType::INVALID_ARGUMENT) => {
                let message = match err.status.into_option() {
                    Some(s) => s.message,
                    None => String::new(),
                };

                Self::InvalidArgument(message)
            }
            Some(VaccelErrorType::UNINITIALIZED) => Self::Uninitialized,
            Some(VaccelErrorType::OUT_OF_BOUNDS) => Self::OutOfBounds,
            Some(VaccelErrorType::EMPTY_VALUE) => Self::EmptyValue,
            Some(VaccelErrorType::CONVERSION_FAILED) => {
                let message = match err.status.into_option() {
                    Some(s) => s.message,
                    None => String::new(),
                };

                Self::ConversionFailed(message)
            }
            None => {
                Self::ConversionFailed("Could not convert `Error` to `VaccelError`".to_string())
            }
        }
    }
}

impl From<Error> for VaccelError {
    fn from(err: Error) -> Self {
        let mut vaccel_error = VaccelError::new();
        match err {
            Error::Ffi(error) => {
                vaccel_error.type_ = VaccelErrorType::FFI.into();
                vaccel_error.ffi_error = Some(error);
            }
            Error::FfiWithStatus { error, status } => {
                vaccel_error.type_ = VaccelErrorType::FFI_WITH_STATUS.into();
                vaccel_error.ffi_error = Some(error);
                vaccel_error.status = Some(status.into()).into();
            }
            Error::InvalidArgument(message) => {
                vaccel_error.type_ = VaccelErrorType::INVALID_ARGUMENT.into();
                let mut status = VaccelStatus::new();
                status.message = message;
                vaccel_error.status = Some(status).into();
            }
            Error::Uninitialized => {
                vaccel_error.type_ = VaccelErrorType::UNINITIALIZED.into();
            }
            Error::OutOfBounds => {
                vaccel_error.type_ = VaccelErrorType::OUT_OF_BOUNDS.into();
            }
            Error::EmptyValue => {
                vaccel_error.type_ = VaccelErrorType::EMPTY_VALUE.into();
            }
            Error::ConversionFailed(message) => {
                vaccel_error.type_ = VaccelErrorType::CONVERSION_FAILED.into();
                let mut status = VaccelStatus::new();
                status.message = message;
                vaccel_error.status = Some(status).into();
            }
        }
        vaccel_error
    }
}

pub type Result<T> = std::result::Result<T, Error>;
