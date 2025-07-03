// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use derive_more::Display;
use thiserror::Error as ThisError;
use vaccel_rpc_proto::vaccel::{
    Error as ProtoError, ErrorType as ProtoErrorType, Status as ProtoStatus,
};

/// An error status to use with operations returning status objects.
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

impl From<Status> for ProtoStatus {
    fn from(status: Status) -> Self {
        let mut vaccel_status = Self::new();
        vaccel_status.code = status.code as u32;
        vaccel_status.message = status.message;
        vaccel_status
    }
}

impl TryFrom<ProtoStatus> for Status {
    type Error = Error;

    fn try_from(status: ProtoStatus) -> Result<Self> {
        Ok(Self::new(
            status
                .code
                .try_into()
                .map_err(|e| Error::ConversionFailed(format!("{}", e)))?,
            &status.message,
        ))
    }
}

/// The core error variants.
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

impl From<ProtoError> for Error {
    fn from(err: ProtoError) -> Self {
        match err.type_.enum_value().ok() {
            Some(ProtoErrorType::FFI) => Self::Ffi(err.ffi_error.unwrap_or(ffi::VACCEL_EBACKEND)),
            Some(ProtoErrorType::FFI_WITH_STATUS) => {
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
            Some(ProtoErrorType::INVALID_ARGUMENT) => {
                let message = match err.status.into_option() {
                    Some(s) => s.message,
                    None => String::new(),
                };

                Self::InvalidArgument(message)
            }
            Some(ProtoErrorType::UNINITIALIZED) => Self::Uninitialized,
            Some(ProtoErrorType::OUT_OF_BOUNDS) => Self::OutOfBounds,
            Some(ProtoErrorType::EMPTY_VALUE) => Self::EmptyValue,
            Some(ProtoErrorType::CONVERSION_FAILED) => {
                let message = match err.status.into_option() {
                    Some(s) => s.message,
                    None => String::new(),
                };

                Self::ConversionFailed(message)
            }
            None => Self::ConversionFailed("Could not convert `Error` to `ProtoError`".to_string()),
        }
    }
}

impl From<Error> for ProtoError {
    fn from(err: Error) -> Self {
        let mut proto_error = ProtoError::new();
        match err {
            Error::Ffi(error) => {
                proto_error.type_ = ProtoErrorType::FFI.into();
                proto_error.ffi_error = Some(error);
            }
            Error::FfiWithStatus { error, status } => {
                proto_error.type_ = ProtoErrorType::FFI_WITH_STATUS.into();
                proto_error.ffi_error = Some(error);
                proto_error.status = Some(status.into()).into();
            }
            Error::InvalidArgument(message) => {
                proto_error.type_ = ProtoErrorType::INVALID_ARGUMENT.into();
                let mut status = ProtoStatus::new();
                status.message = message;
                proto_error.status = Some(status).into();
            }
            Error::Uninitialized => {
                proto_error.type_ = ProtoErrorType::UNINITIALIZED.into();
            }
            Error::OutOfBounds => {
                proto_error.type_ = ProtoErrorType::OUT_OF_BOUNDS.into();
            }
            Error::EmptyValue => {
                proto_error.type_ = ProtoErrorType::EMPTY_VALUE.into();
            }
            Error::ConversionFailed(message) => {
                proto_error.type_ = ProtoErrorType::CONVERSION_FAILED.into();
                let mut status = ProtoStatus::new();
                status.message = message;
                proto_error.status = Some(status).into();
            }
        }
        proto_error
    }
}

pub type Result<T> = std::result::Result<T, Error>;
