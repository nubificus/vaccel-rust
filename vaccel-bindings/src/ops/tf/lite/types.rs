// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use half::f16;

/// Data types for tensors.
#[derive(Debug, PartialEq, Default)]
pub enum DataType {
    UnknownValue(u32),
    NoType,
    #[default]
    Float32,
    Int32,
    UInt8,
    Int64,
    String,
    Bool,
    Int16,
    Complex64,
    Int8,
    Float16,
    Float64,
    Complex128,
    UInt64,
    Resource,
    Variant,
    UInt32,
    UInt16,
    Int4,
}

impl DataType {
    /// Converts the `DataType` to the corresponding C API integer
    pub fn to_int(&self) -> u32 {
        match self {
            DataType::NoType => ffi::VACCEL_TFLITE_NOTYPE,
            DataType::Float32 => ffi::VACCEL_TFLITE_FLOAT32,
            DataType::Int32 => ffi::VACCEL_TFLITE_INT32,
            DataType::UInt8 => ffi::VACCEL_TFLITE_UINT8,
            DataType::Int64 => ffi::VACCEL_TFLITE_INT64,
            DataType::String => ffi::VACCEL_TFLITE_STRING,
            DataType::Bool => ffi::VACCEL_TFLITE_BOOL,
            DataType::Int16 => ffi::VACCEL_TFLITE_INT16,
            DataType::Complex64 => ffi::VACCEL_TFLITE_COMPLEX64,
            DataType::Int8 => ffi::VACCEL_TFLITE_INT8,
            DataType::Float16 => ffi::VACCEL_TFLITE_FLOAT16,
            DataType::Float64 => ffi::VACCEL_TFLITE_FLOAT64,
            DataType::Complex128 => ffi::VACCEL_TFLITE_COMPLEX128,
            DataType::UInt64 => ffi::VACCEL_TFLITE_UINT64,
            DataType::Resource => ffi::VACCEL_TFLITE_RESOURCE,
            DataType::Variant => ffi::VACCEL_TFLITE_VARIANT,
            DataType::UInt32 => ffi::VACCEL_TFLITE_UINT32,
            DataType::UInt16 => ffi::VACCEL_TFLITE_UINT16,
            DataType::Int4 => ffi::VACCEL_TFLITE_INT4,
            DataType::UnknownValue(c) => *c,
        }
    }

    /// Creates a `DataType` from a corresponding C API integer
    pub fn from_int(val: u32) -> DataType {
        match val {
            ffi::VACCEL_TFLITE_NOTYPE => DataType::NoType,
            ffi::VACCEL_TFLITE_FLOAT32 => DataType::Float32,
            ffi::VACCEL_TFLITE_INT32 => DataType::Int32,
            ffi::VACCEL_TFLITE_UINT8 => DataType::UInt8,
            ffi::VACCEL_TFLITE_INT64 => DataType::Int64,
            ffi::VACCEL_TFLITE_STRING => DataType::String,
            ffi::VACCEL_TFLITE_BOOL => DataType::Bool,
            ffi::VACCEL_TFLITE_INT16 => DataType::Int16,
            ffi::VACCEL_TFLITE_COMPLEX64 => DataType::Complex64,
            ffi::VACCEL_TFLITE_INT8 => DataType::Int8,
            ffi::VACCEL_TFLITE_FLOAT16 => DataType::Float16,
            ffi::VACCEL_TFLITE_FLOAT64 => DataType::Float64,
            ffi::VACCEL_TFLITE_COMPLEX128 => DataType::Complex128,
            ffi::VACCEL_TFLITE_UINT64 => DataType::UInt64,
            ffi::VACCEL_TFLITE_RESOURCE => DataType::Resource,
            ffi::VACCEL_TFLITE_VARIANT => DataType::Variant,
            ffi::VACCEL_TFLITE_UINT32 => DataType::UInt32,
            ffi::VACCEL_TFLITE_UINT16 => DataType::UInt16,
            ffi::VACCEL_TFLITE_INT4 => DataType::Int4,
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
    f32 => Float32,
    i32 => Int32,
    u8 => UInt8,
    i64 => Int64,
    i16 => Int16,
    i8 => Int8,
    f16 => Float16,
    f64 => Float64,
    u64 => UInt64,
    u32 => UInt32,
    u16 => UInt16,
}
