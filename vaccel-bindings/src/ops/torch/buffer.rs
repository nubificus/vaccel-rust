// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Result};
use log::warn;
use std::ptr::{self, NonNull};

/// Wrapper for the `struct vaccel_torch_buffer` C object.
#[derive(Debug)]
pub struct Buffer {
    inner: NonNull<ffi::vaccel_torch_buffer>,
    owned: bool,
    _buffer: Option<Vec<u8>>,
}

impl Buffer {
    /// Creates a new `Buffer`.
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let mut buffer = data;
        let mut ptr: *mut ffi::vaccel_torch_buffer = ptr::null_mut();
        match unsafe {
            ffi::vaccel_torch_buffer_new(&mut ptr, buffer.as_mut_ptr() as *mut _, buffer.len())
                as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| Buffer {
                inner,
                owned: true,
                _buffer: Some(buffer),
            })
            .ok_or(Error::EmptyValue)
    }

    /// Returns the data of the `Buffer` as a slice.
    pub fn as_slice(&self) -> Option<&[u8]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.data.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(inner.data as *const _, inner.size) })
        }
    }

    /// Returns the data of the `Buffer` as a mutable slice.
    pub fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        let inner = unsafe { self.inner.as_mut() };

        if inner.data.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts_mut(inner.data as *mut _, inner.size) })
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.owned {
            // Data is not owned by vaccel runtime. Unset it from the buffer
            // so we avoid double free.
            let mut data = std::ptr::null_mut();
            let mut size = Default::default();
            unsafe {
                ffi::vaccel_torch_buffer_take_data(self.inner.as_ptr(), &mut data, &mut size)
            };

            let ret = unsafe { ffi::vaccel_torch_buffer_delete(self.inner.as_ptr()) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete Buffer inner: {}", ret);
            }
        }
    }
}

impl_component_handle!(
    Buffer,
    ffi::vaccel_torch_buffer,
    inner,
    owned,
    extra_vec_fields: {
        _buffer: None,
    }
);
