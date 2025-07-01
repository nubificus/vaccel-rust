// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use num_enum::{FromPrimitive, IntoPrimitive};

/// Data types for tensors.
#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum DataType {
    Float = ffi::VACCEL_TF_FLOAT,
    Double = ffi::VACCEL_TF_DOUBLE,
    Int32 = ffi::VACCEL_TF_INT32,
    UInt8 = ffi::VACCEL_TF_UINT8,
    Int16 = ffi::VACCEL_TF_INT16,
    Int8 = ffi::VACCEL_TF_INT8,
    String = ffi::VACCEL_TF_STRING,
    Complex64 = ffi::VACCEL_TF_COMPLEX64,
    Int64 = ffi::VACCEL_TF_INT64,
    Bool = ffi::VACCEL_TF_BOOL,
    QInt8 = ffi::VACCEL_TF_QINT8,
    QUInt8 = ffi::VACCEL_TF_QUINT8,
    QInt32 = ffi::VACCEL_TF_QINT32,
    BFloat16 = ffi::VACCEL_TF_BFLOAT16,
    QInt16 = ffi::VACCEL_TF_QINT16,
    QUInt16 = ffi::VACCEL_TF_QUINT16,
    UInt16 = ffi::VACCEL_TF_UINT16,
    Complex128 = ffi::VACCEL_TF_COMPLEX128,
    Half = ffi::VACCEL_TF_HALF,
    Resource = ffi::VACCEL_TF_RESOURCE,
    Variant = ffi::VACCEL_TF_VARIANT,
    UInt32 = ffi::VACCEL_TF_UINT32,
    UInt64 = ffi::VACCEL_TF_UINT64,
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
    f32 => Float,
    f64 => Double,
    i32 => Int32,
    u8 => UInt8,
    i16 => Int16,
    i8 => Int8,
    i64 => Int64,
    half::bf16 => BFloat16,
    u16 => UInt16,
    half::f16 => Half,
    u32 => UInt32,
    u64 => UInt64,
}
