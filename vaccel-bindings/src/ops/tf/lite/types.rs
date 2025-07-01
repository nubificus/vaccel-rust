// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use num_enum::{FromPrimitive, IntoPrimitive};

/// Data types for tensors.
#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum DataType {
    NoType = ffi::VACCEL_TFLITE_NOTYPE,
    Float32 = ffi::VACCEL_TFLITE_FLOAT32,
    Int32 = ffi::VACCEL_TFLITE_INT32,
    UInt8 = ffi::VACCEL_TFLITE_UINT8,
    Int64 = ffi::VACCEL_TFLITE_INT64,
    String = ffi::VACCEL_TFLITE_STRING,
    Bool = ffi::VACCEL_TFLITE_BOOL,
    Int16 = ffi::VACCEL_TFLITE_INT16,
    Complex64 = ffi::VACCEL_TFLITE_COMPLEX64,
    Int8 = ffi::VACCEL_TFLITE_INT8,
    Float16 = ffi::VACCEL_TFLITE_FLOAT16,
    Float64 = ffi::VACCEL_TFLITE_FLOAT64,
    Complex128 = ffi::VACCEL_TFLITE_COMPLEX128,
    UInt64 = ffi::VACCEL_TFLITE_UINT64,
    Resource = ffi::VACCEL_TFLITE_RESOURCE,
    Variant = ffi::VACCEL_TFLITE_VARIANT,
    UInt32 = ffi::VACCEL_TFLITE_UINT32,
    UInt16 = ffi::VACCEL_TFLITE_UINT16,
    Int4 = ffi::VACCEL_TFLITE_INT4,
    #[num_enum(catch_all)]
    Unknown(u32),
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
    half::f16 => Float16,
    f64 => Float64,
    u64 => UInt64,
    u32 => UInt32,
    u16 => UInt16,
}
