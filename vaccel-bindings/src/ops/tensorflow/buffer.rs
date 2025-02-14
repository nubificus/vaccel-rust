// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Result};
use std::ops::{Deref, DerefMut};

pub struct Buffer {
    inner: *mut ffi::vaccel_tf_buffer,
    vaccel_owned: bool,
}

impl Buffer {
    pub fn new(data: &[u8]) -> Result<Self> {
        let mut inner: *mut ffi::vaccel_tf_buffer = std::ptr::null_mut();
        match unsafe {
            ffi::vaccel_tf_buffer_new(&mut inner, data.as_ptr() as *mut _, data.len()) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Runtime(err)),
        }
        assert!(!inner.is_null());
        unsafe { assert!(!(*inner).data.is_null()) };
        unsafe { assert!(!(*inner).size > 0) };

        Ok(Buffer {
            inner,
            vaccel_owned: false,
        })
    }

    /// # Safety
    ///
    /// `buffer` is expected to be a valid pointer to an object allocated
    /// manually or by the respective vAccel function.
    pub unsafe fn from_ffi(buffer: *mut ffi::vaccel_tf_buffer) -> Result<Self> {
        if buffer.is_null() || (*buffer).data.is_null() || (*buffer).size == 0 {
            return Err(Error::InvalidArgument);
        }

        Ok(Buffer {
            inner: buffer,
            vaccel_owned: true,
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((*self.inner).data as *const u8, (*self.inner).size) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut((*self.inner).data as *mut u8, (*self.inner).size) }
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_tf_buffer {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_tf_buffer {
        self.inner
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if !self.vaccel_owned {
            // Data is not owned from vaccel runtime. Unset it from
            // the buffer so we avoid double free.
            let mut data = std::ptr::null_mut();
            let mut size = Default::default();
            unsafe { ffi::vaccel_tf_buffer_take_data(self.inner, &mut data, &mut size) };
        }

        unsafe { ffi::vaccel_tf_buffer_delete(self.inner) };
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}
