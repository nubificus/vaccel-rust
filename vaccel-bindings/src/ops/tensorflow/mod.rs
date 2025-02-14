// SPDX-License-Identifier: Apache-2.0

use crate::ffi;

pub mod buffer;
pub mod lite;
#[cfg(target_pointer_width = "64")]
pub mod model;
#[cfg(target_pointer_width = "64")]
pub mod node;
pub mod status;
#[cfg(target_pointer_width = "64")]
pub mod tensor;

pub use buffer::Buffer;
#[cfg(target_pointer_width = "64")]
pub use model::{InferenceArgs, InferenceResult, Model};
#[cfg(target_pointer_width = "64")]
pub use node::Node;
pub use status::Status;
#[cfg(target_pointer_width = "64")]
pub use tensor::{Tensor, TensorAny, TensorType};

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
