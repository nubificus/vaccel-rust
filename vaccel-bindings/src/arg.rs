// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Result};
use num_enum::{FromPrimitive, IntoPrimitive};
use std::ptr::{self, NonNull};

/// Wrapper for the `struct vaccel_arg` C object.
#[derive(Debug)]
pub struct Arg {
    inner: NonNull<ffi::vaccel_arg>,
    owned: bool,
    _buffer: Option<Vec<u8>>,
}

/// The arg types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ArgType {
    Raw = ffi::VACCEL_ARG_RAW,
    Int8 = ffi::VACCEL_ARG_INT8,
    Int8Array = ffi::VACCEL_ARG_INT8_ARRAY,
    Int16 = ffi::VACCEL_ARG_INT16,
    Int16Array = ffi::VACCEL_ARG_INT16_ARRAY,
    Int32 = ffi::VACCEL_ARG_INT32,
    Int32Array = ffi::VACCEL_ARG_INT32_ARRAY,
    Int64 = ffi::VACCEL_ARG_INT64,
    Int64Array = ffi::VACCEL_ARG_INT64_ARRAY,
    Uint8 = ffi::VACCEL_ARG_UINT8,
    Uint8Array = ffi::VACCEL_ARG_UINT8_ARRAY,
    Uint16 = ffi::VACCEL_ARG_UINT16,
    Uint16Array = ffi::VACCEL_ARG_UINT16_ARRAY,
    Uint32 = ffi::VACCEL_ARG_UINT32,
    Uint32Array = ffi::VACCEL_ARG_UINT32_ARRAY,
    Uint64 = ffi::VACCEL_ARG_UINT64,
    Uint64Array = ffi::VACCEL_ARG_UINT64_ARRAY,
    Float32 = ffi::VACCEL_ARG_FLOAT32,
    Float32Array = ffi::VACCEL_ARG_FLOAT32_ARRAY,
    Float64 = ffi::VACCEL_ARG_FLOAT64,
    Float64Array = ffi::VACCEL_ARG_FLOAT64_ARRAY,
    Bool = ffi::VACCEL_ARG_BOOL,
    BoolArray = ffi::VACCEL_ARG_BOOL_ARRAY,
    Char = ffi::VACCEL_ARG_CHAR,
    CharArray = ffi::VACCEL_ARG_CHAR_ARRAY,
    Uchar = ffi::VACCEL_ARG_UCHAR,
    UcharArray = ffi::VACCEL_ARG_UCHAR_ARRAY,
    String = ffi::VACCEL_ARG_STRING,
    Buffer = ffi::VACCEL_ARG_BUFFER,
    Custom = ffi::VACCEL_ARG_CUSTOM,
    #[num_enum(catch_all)]
    Unknown(u32),
}

impl Arg {
    /// Creates a new `Arg`.
    pub fn new(buf: &[u8], arg_type: ArgType, custom_type_id: u32) -> Result<Self> {
        let mut ptr: *mut ffi::vaccel_arg = ptr::null_mut();
        match unsafe {
            ffi::vaccel_arg_new(
                &mut ptr,
                buf.as_ptr() as *const _,
                buf.len(),
                arg_type.into(),
                custom_type_id,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| Arg {
                inner,
                owned: true,
                _buffer: None,
            })
            .ok_or(Error::EmptyValue)
    }

    /// Creates a new `Arg` by consuming a vector of existing data.
    pub fn from_buf(buf: Vec<u8>, arg_type: ArgType, custom_type_id: u32) -> Result<Self> {
        let mut buffer = buf;
        let mut ptr: *mut ffi::vaccel_arg = ptr::null_mut();
        match unsafe {
            ffi::vaccel_arg_from_buf(
                &mut ptr,
                buffer.as_mut_ptr() as *mut _,
                buffer.len(),
                arg_type.into(),
                custom_type_id,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| Arg {
                inner,
                owned: true,
                _buffer: Some(buffer),
            })
            .ok_or(Error::EmptyValue)
    }

    /// Returns the buffer of the `Arg`.
    pub fn buf(&self) -> Option<&[u8]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.buf.is_null() || inner.size == 0 {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(inner.buf as *const _, inner.size) })
        }
    }

    /// Returns the size of the `Arg` buffer.
    pub fn size(&self) -> usize {
        unsafe { self.inner.as_ref().size }
    }

    /// Returns the type of the `Arg`.
    pub fn type_(&self) -> ArgType {
        ArgType::from(unsafe { self.inner.as_ref().type_ })
    }

    /// Returns the custom type ID of the `Arg`.
    pub fn custom_type_id(&self) -> u32 {
        unsafe { self.inner.as_ref().custom_type_id }
    }
}

impl_component_drop!(Arg, vaccel_arg_delete, inner, owned);

impl_component_handle!(
    Arg,
    ffi::vaccel_arg,
    inner,
    owned,
    extra_vec_fields: {
        _buffer: None,
    }
);
