// SPDX-License-Identifier: Apache-2.0

use crate::ffi;

pub mod buffer;
pub mod model;
pub mod tensor;

pub use buffer::Buffer;
pub use model::{InferenceArgs, InferenceResult, Model};
pub use tensor::{Tensor, TensorAny, TensorType};

#[derive(Debug)]
pub enum Code {
    Ok = 0,
    Cancelled,
    Unknown,
    InvalidArgument,
    DeadlineExceeded,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    FailedPrecondition,
    Aborted,
    OutOfRange,
    Unimplemented,
    Internal,
    Unavailable,
    DataLoss,
    Unauthenticated,
}

impl Code {
    pub(crate) fn to_u8(&self) -> u8 {
        match self {
            Code::Ok => 0,
            Code::Cancelled => 1,
            Code::Unknown => 2,
            Code::InvalidArgument => 3,
            Code::DeadlineExceeded => 4,
            Code::NotFound => 5,
            Code::AlreadyExists => 6,
            Code::PermissionDenied => 7,
            Code::ResourceExhausted => 8,
            Code::FailedPrecondition => 9,
            Code::Aborted => 10,
            Code::OutOfRange => 11,
            Code::Unimplemented => 12,
            Code::Internal => 13,
            Code::Unavailable => 14,
            Code::DataLoss => 15,
            Code::Unauthenticated => 16,
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub enum DataType {
    UInt8,
    Int8,
    Int16,
    Int32,
    Int64,
    Half,
    #[default]
    Float,
    UnknownValue(u32),
}

impl DataType {
    pub fn to_int(&self) -> u32 {
        match self {
            DataType::UInt8 => ffi::VACCEL_TORCH_BYTE,
            DataType::Int8 => ffi::VACCEL_TORCH_CHAR,
            DataType::Int16 => ffi::VACCEL_TORCH_SHORT,
            DataType::Int32 => ffi::VACCEL_TORCH_INT,
            DataType::Int64 => ffi::VACCEL_TORCH_LONG,
            DataType::Half => ffi::VACCEL_TORCH_HALF,
            DataType::Float => ffi::VACCEL_TORCH_FLOAT,
            DataType::UnknownValue(c) => *c,
        }
    }

    pub fn from_int(val: u32) -> DataType {
        match val {
            ffi::VACCEL_TORCH_BYTE => DataType::UInt8,
            ffi::VACCEL_TORCH_CHAR => DataType::Int8,
            ffi::VACCEL_TORCH_SHORT => DataType::Int16,
            ffi::VACCEL_TORCH_INT => DataType::Int32,
            ffi::VACCEL_TORCH_LONG => DataType::Int64,
            ffi::VACCEL_TORCH_HALF => DataType::Half,
            ffi::VACCEL_TORCH_FLOAT => DataType::Float,
            unknown => DataType::UnknownValue(unknown),
        }
    }
}
