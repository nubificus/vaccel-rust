use crate::ffi;

pub mod inference;
pub mod tensor;

pub use inference::{InferenceArgs, InferenceResult};
pub use tensor::{Tensor, TensorAny, TensorType};

#[derive(Debug)]
pub enum Code {
    Ok = 0,
    Error,
    DelegateError,
    ApplicationError,
    DelegateDataNotFound,
    DelegateDataWriteError,
    DelegateDataReadError,
    UnresolvedOps,
    Cancelled,
}

impl Code {
    pub(crate) fn to_u8(&self) -> u8 {
        match self {
            Code::Ok => 0,
            Code::Error => 1,
            Code::DelegateError => 2,
            Code::ApplicationError => 3,
            Code::DelegateDataNotFound => 4,
            Code::DelegateDataWriteError => 5,
            Code::DelegateDataReadError => 6,
            Code::UnresolvedOps => 7,
            Code::Cancelled => 8,
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub enum Type {
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

impl Type {
    pub fn to_int(&self) -> u32 {
        match self {
            Type::NoType => ffi::VACCEL_TFLITE_NOTYPE,
            Type::Float32 => ffi::VACCEL_TFLITE_FLOAT32,
            Type::Int32 => ffi::VACCEL_TFLITE_INT32,
            Type::UInt8 => ffi::VACCEL_TFLITE_UINT8,
            Type::Int64 => ffi::VACCEL_TFLITE_INT64,
            Type::String => ffi::VACCEL_TFLITE_STRING,
            Type::Bool => ffi::VACCEL_TFLITE_BOOL,
            Type::Int16 => ffi::VACCEL_TFLITE_INT16,
            Type::Complex64 => ffi::VACCEL_TFLITE_COMPLEX64,
            Type::Int8 => ffi::VACCEL_TFLITE_INT8,
            Type::Float16 => ffi::VACCEL_TFLITE_FLOAT16,
            Type::Float64 => ffi::VACCEL_TFLITE_FLOAT64,
            Type::Complex128 => ffi::VACCEL_TFLITE_COMPLEX128,
            Type::UInt64 => ffi::VACCEL_TFLITE_UINT64,
            Type::Resource => ffi::VACCEL_TFLITE_RESOURCE,
            Type::Variant => ffi::VACCEL_TFLITE_VARIANT,
            Type::UInt32 => ffi::VACCEL_TFLITE_UINT32,
            Type::UInt16 => ffi::VACCEL_TFLITE_UINT16,
            Type::Int4 => ffi::VACCEL_TFLITE_INT4,
            Type::UnknownValue(c) => *c,
        }
    }

    pub fn from_int(val: u32) -> Type {
        match val {
            ffi::VACCEL_TFLITE_NOTYPE => Type::NoType,
            ffi::VACCEL_TFLITE_FLOAT32 => Type::Float32,
            ffi::VACCEL_TFLITE_INT32 => Type::Int32,
            ffi::VACCEL_TFLITE_UINT8 => Type::UInt8,
            ffi::VACCEL_TFLITE_INT64 => Type::Int64,
            ffi::VACCEL_TFLITE_STRING => Type::String,
            ffi::VACCEL_TFLITE_BOOL => Type::Bool,
            ffi::VACCEL_TFLITE_INT16 => Type::Int16,
            ffi::VACCEL_TFLITE_COMPLEX64 => Type::Complex64,
            ffi::VACCEL_TFLITE_INT8 => Type::Int8,
            ffi::VACCEL_TFLITE_FLOAT16 => Type::Float16,
            ffi::VACCEL_TFLITE_FLOAT64 => Type::Float64,
            ffi::VACCEL_TFLITE_COMPLEX128 => Type::Complex128,
            ffi::VACCEL_TFLITE_UINT64 => Type::UInt64,
            ffi::VACCEL_TFLITE_RESOURCE => Type::Resource,
            ffi::VACCEL_TFLITE_VARIANT => Type::Variant,
            ffi::VACCEL_TFLITE_UINT32 => Type::UInt32,
            ffi::VACCEL_TFLITE_UINT16 => Type::UInt16,
            ffi::VACCEL_TFLITE_INT4 => Type::Int4,
            unknown => Type::UnknownValue(unknown),
        }
    }
}
