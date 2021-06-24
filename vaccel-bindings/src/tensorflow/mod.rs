use crate::ffi;

use std::ffi::CStr;
use std::fmt;

pub mod buffer;
pub mod node;
pub mod saved_model;
pub mod tensor;

pub use buffer::Buffer;
pub use node::Node;
pub use saved_model::SavedModel;
pub use tensor::{Tensor, TensorAny, TensorType};

#[derive(Debug)]
pub enum Code {
    Ok = 0,
    Cancelled,
    Unkown,
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
            Code::Unkown => 2,
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

#[derive(Default)]
pub struct Status {
    inner: ffi::vaccel_tf_status,
}

impl Status {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn error_code(&self) -> u8 {
        self.inner.error_code
    }

    pub fn message(&self) -> String {
        if self.inner.message.is_null() {
            return String::new();
        }

        let cmsg = unsafe { CStr::from_ptr(self.inner.message) };
        cmsg.to_str().unwrap_or("").to_owned()
    }

    pub fn is_ok(&self) -> bool {
        self.error_code() == Code::Ok.to_u8()
    }

    pub fn to_string(&self) -> String {
        format!("'{} (id:{})'", self.message(), self.error_code())
    }

    pub(crate) fn inner(&self) -> &ffi::vaccel_tf_status {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_tf_status {
        &mut self.inner
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    UnknownValue(u32),
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

impl Default for DataType {
    fn default() -> Self {
        DataType::Float
    }
}
