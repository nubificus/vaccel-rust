// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use half::f16;

/// Data types for tensors.
#[derive(Debug, PartialEq, Default)]
pub enum DataType {
    UnknownValue(u32),
    #[default]
    Float,
    Double,
    Int32,
    UInt8,
    Int16,
    Int8,
    String,
    Complex64,
    Int64,
    Bool,
    QInt8,
    QUInt8,
    QInt32,
    BFloat16,
    QInt16,
    QUInt16,
    UInt16,
    Complex128,
    Half,
    Resource,
    Variant,
    UInt32,
    UInt64,
}

impl DataType {
    /// Converts the `DataType` to the corresponding C API integer
    pub fn to_int(&self) -> u32 {
        match self {
            DataType::Float => ffi::VACCEL_TF_FLOAT,
            DataType::Double => ffi::VACCEL_TF_DOUBLE,
            DataType::Int32 => ffi::VACCEL_TF_INT32,
            DataType::UInt8 => ffi::VACCEL_TF_UINT8,
            DataType::Int16 => ffi::VACCEL_TF_INT16,
            DataType::Int8 => ffi::VACCEL_TF_INT8,
            DataType::String => ffi::VACCEL_TF_STRING,
            DataType::Complex64 => ffi::VACCEL_TF_COMPLEX64,
            DataType::Int64 => ffi::VACCEL_TF_INT64,
            DataType::Bool => ffi::VACCEL_TF_BOOL,
            DataType::QInt8 => ffi::VACCEL_TF_QINT8,
            DataType::QUInt8 => ffi::VACCEL_TF_QUINT8,
            DataType::QInt32 => ffi::VACCEL_TF_QINT32,
            DataType::BFloat16 => ffi::VACCEL_TF_BFLOAT16,
            DataType::QInt16 => ffi::VACCEL_TF_QINT16,
            DataType::QUInt16 => ffi::VACCEL_TF_QUINT16,
            DataType::UInt16 => ffi::VACCEL_TF_UINT16,
            DataType::Complex128 => ffi::VACCEL_TF_COMPLEX128,
            DataType::Half => ffi::VACCEL_TF_HALF,
            DataType::Resource => ffi::VACCEL_TF_RESOURCE,
            DataType::Variant => ffi::VACCEL_TF_VARIANT,
            DataType::UInt32 => ffi::VACCEL_TF_UINT32,
            DataType::UInt64 => ffi::VACCEL_TF_UINT64,
            DataType::UnknownValue(c) => *c,
        }
    }

    /// Creates a `DataType` from a corresponding C API integer
    pub fn from_int(val: u32) -> DataType {
        match val {
            ffi::VACCEL_TF_FLOAT => DataType::Float,
            ffi::VACCEL_TF_DOUBLE => DataType::Double,
            ffi::VACCEL_TF_INT32 => DataType::Int32,
            ffi::VACCEL_TF_UINT8 => DataType::UInt8,
            ffi::VACCEL_TF_INT16 => DataType::Int16,
            ffi::VACCEL_TF_INT8 => DataType::Int8,
            ffi::VACCEL_TF_STRING => DataType::String,
            ffi::VACCEL_TF_COMPLEX64 => DataType::Complex64,
            ffi::VACCEL_TF_INT64 => DataType::Int64,
            ffi::VACCEL_TF_BOOL => DataType::Bool,
            ffi::VACCEL_TF_QINT8 => DataType::QInt8,
            ffi::VACCEL_TF_QUINT8 => DataType::QUInt8,
            ffi::VACCEL_TF_BFLOAT16 => DataType::BFloat16,
            ffi::VACCEL_TF_QINT16 => DataType::QInt16,
            ffi::VACCEL_TF_QUINT16 => DataType::QUInt16,
            ffi::VACCEL_TF_UINT16 => DataType::UInt16,
            ffi::VACCEL_TF_COMPLEX128 => DataType::Complex128,
            ffi::VACCEL_TF_HALF => DataType::Half,
            ffi::VACCEL_TF_RESOURCE => DataType::Resource,
            ffi::VACCEL_TF_VARIANT => DataType::Variant,
            ffi::VACCEL_TF_UINT32 => DataType::UInt32,
            ffi::VACCEL_TF_UINT64 => DataType::UInt64,
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
    f32 => Float,
    f64 => Double,
    i32 => Int32,
    u8 => UInt8,
    i16 => Int16,
    i8 => Int8,
    i64 => Int64,
    u16 => UInt16,
    f16 => Half,
    u32 => UInt32,
    u64 => UInt64,
}
