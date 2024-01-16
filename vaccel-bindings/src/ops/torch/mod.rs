use crate::ffi;

pub mod buffer;
pub mod inference;
pub mod tensor;

pub use buffer::Buffer;
pub use inference::{InferenceArgs, InferenceResult};
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

/**************/
/*  DataType  */
/**************/
#[derive(Debug, PartialEq, Default)]
pub enum DataType {
    UnknownValue(u32),
    #[default]
    Float,
    //Double,
    Int32,
    UInt8,
    Int16,
    Int8,
    /*
    String,
    Complex64,
    Bool,
    QInt8,
    QUInt8,
    QInt32,
    BFloat16,
    QInt16,
    QUInt16,
    UInt16,
    Complex128,
    Resource,
    Variant,
    */
    Int64,
    Half,
    //UInt32,
    //UInt64,
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

            // DataType::Double => ffi::VACCEL_TORCH_DOUBLE,
            // DataType::String => ffi::VACCEL_TORCH_STRING,
            // DataType::Complex64 => ffi::VACCEL_TORCH_COMPLEX64,
            // DataType::Bool => ffi::VACCEL_TORCH_BOOL,
            // DataType::QInt8 => ffi::VACCEL_TORCH_QINT8,
            // DataType::QUInt8 => ffi::VACCEL_TORCH_QUINT8,
            // DataType::QInt32 => ffi::VACCEL_TORCH_QINT32,
            // DataType::BFloat16 => ffi::VACCEL_TORCH_BFLOAT16,
            // DataType::QInt16 => ffi::VACCEL_TORCH_QINT16,
            // DataType::QUInt16 => ffi::VACCEL_TORCH_QUINT16,
            // DataType::UInt16 => ffi::VACCEL_TORCH_UINT16,
            // DataType::Complex128 => ffi::VACCEL_TORCH_COMPLEX128,
            // DataType::Resource => ffi::VACCEL_TORCH_RESOURCE,
            // DataType::Variant => ffi::VACCEL_TORCH_VARIANT,
            // DataType::UInt32 => ffi::VACCEL_TORCH_UINT32,
            // DataType::UInt64 => ffi::VACCEL_TORCH_UINT64,
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

            // ffi::VACCEL_TORCH_DOUBLE => DataType::Double,
            // ffi::VACCEL_TORCH_STRING => DataType::String,
            // ffi::VACCEL_TORCH_COMPLEX64 => DataType::Complex64,
            // ffi::VACCEL_TORCH_BOOL => DataType::Bool,
            // ffi::VACCEL_TORCH_QINT8 => DataType::QInt8,
            // ffi::VACCEL_TORCH_QUINT8 => DataType::QUInt8,
            // ffi::VACCEL_TORCH_QINT32 => DataType::QInt32,
            // ffi::VACCEL_TORCH_BFLOAT16 => DataType::BFloat16,
            // ffi::VACCEL_TORCH_QINT16 => DataType::QInt16,
            // ffi::VACCEL_TORCH_QUINT16 => DataType::QUInt16,
            // ffi::VACCEL_TORCH_UINT16 => DataType::UInt16,
            // ffi::VACCEL_TORCH_COMPLEX128 => DataType::Complex128,
            // ffi::VACCEL_TORCH_RESOURCE => DataType::Resource,
            // ffi::VACCEL_TORCH_VARIANT => DataType::Variant,
            // ffi::VACCEL_TORCH_UINT32 => DataType::UInt32,
            // ffi::VACCEL_TORCH_UINT64 => DataType::UInt64,
            unknown => DataType::UnknownValue(unknown),
        }
    }
}
