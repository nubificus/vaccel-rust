// SPDX-License-Identifier: Apache-2.0

use derive_more::Display;
use thiserror::Error as ThisError;

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

pub type Result<T> = std::result::Result<T, Error>;
