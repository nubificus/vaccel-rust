// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Result};
use std::ptr::{self, NonNull};

/// Wrapper for the `struct vaccel_arg` C object.
#[derive(Debug)]
pub struct Arg {
    inner: NonNull<ffi::vaccel_arg>,
    owned: bool,
    _buffer: Option<Vec<u8>>,
}

impl Arg {
    /// Creates a new `Arg`.
    pub fn new(buf: Vec<u8>, argtype: u32) -> Result<Self> {
        let mut buffer = buf;
        let mut ptr: *mut ffi::vaccel_arg = ptr::null_mut();
        match unsafe {
            ffi::vaccel_arg_new(
                &mut ptr,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                argtype,
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
            Some(unsafe { std::slice::from_raw_parts(inner.buf as *const _, inner.size as usize) })
        }
    }

    /// Returns the size of the `Arg` buffer.
    pub fn size(&self) -> usize {
        unsafe { self.inner.as_ref().size as usize }
    }

    /// Returns the type of the `Arg`.
    pub fn argtype(&self) -> u32 {
        unsafe { self.inner.as_ref().argtype }
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
