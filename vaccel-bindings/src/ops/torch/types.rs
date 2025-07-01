// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use num_enum::{FromPrimitive, IntoPrimitive};

/// Data types for tensors.
#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum DataType {
    Byte = ffi::VACCEL_TORCH_BYTE,
    Char = ffi::VACCEL_TORCH_CHAR,
    Short = ffi::VACCEL_TORCH_SHORT,
    Int = ffi::VACCEL_TORCH_INT,
    Long = ffi::VACCEL_TORCH_LONG,
    Half = ffi::VACCEL_TORCH_HALF,
    Float = ffi::VACCEL_TORCH_FLOAT,
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
    u8 => Byte,
    i8 => Char,
    i16 => Short,
    i32 => Int,
    i64 => Long,
    half::f16 => Half,
    f32 => Float,
}
