// SPDX-License-Identifier: Apache-2.0

use crate::ffi;

pub mod buffer;
pub mod model;
pub mod tensor;

pub use buffer::Buffer;
pub use model::{InferenceArgs, InferenceResult, Model};
pub use tensor::{Tensor, TensorAny, TensorType};

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
