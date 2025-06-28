// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use half::f16;

/// Data types for tensors.
#[derive(Debug, PartialEq, Default)]
pub enum DataType {
    Byte,
    Char,
    Short,
    Int,
    Long,
    Half,
    #[default]
    Float,
    UnknownValue(u32),
}

impl DataType {
    /// Converts the `DataType` to the corresponding C API integer
    pub fn to_int(&self) -> u32 {
        match self {
            DataType::Byte => ffi::VACCEL_TORCH_BYTE,
            DataType::Char => ffi::VACCEL_TORCH_CHAR,
            DataType::Short => ffi::VACCEL_TORCH_SHORT,
            DataType::Int => ffi::VACCEL_TORCH_INT,
            DataType::Long => ffi::VACCEL_TORCH_LONG,
            DataType::Half => ffi::VACCEL_TORCH_HALF,
            DataType::Float => ffi::VACCEL_TORCH_FLOAT,
            DataType::UnknownValue(c) => *c,
        }
    }

    /// Creates a `DataType` from a corresponding C API integer
    pub fn from_int(val: u32) -> DataType {
        match val {
            ffi::VACCEL_TORCH_BYTE => DataType::Byte,
            ffi::VACCEL_TORCH_CHAR => DataType::Char,
            ffi::VACCEL_TORCH_SHORT => DataType::Short,
            ffi::VACCEL_TORCH_INT => DataType::Int,
            ffi::VACCEL_TORCH_LONG => DataType::Long,
            ffi::VACCEL_TORCH_HALF => DataType::Half,
            ffi::VACCEL_TORCH_FLOAT => DataType::Float,
            unknown => DataType::UnknownValue(unknown),
        }
    }
}

/// Provides basic methods for Rust-convertible tensor data types.
pub trait TensorType: Default + Clone + bytemuck::Pod {
    /// DataType of the Tensor type
    fn data_type() -> DataType;

    /// Unit value of type
    fn one() -> Self;

    /// Zero value of type
    fn zero() -> Self;
}

impl_tensor_types! {
    DataType;
    u8 => Byte,
    i8 => Char,
    i16 => Short,
    i32 => Int,
    i64 => Long,
    f16 => Half,
    f32 => Float,
}
