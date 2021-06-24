use crate::ffi;
use crate::{Error, Result};

use std::ops::{Deref, DerefMut};

pub struct Buffer {
    inner: *mut ffi::vaccel_tf_buffer,
    vaccel_owned: bool,
}

impl Buffer {
    pub fn new(data: &[u8]) -> Self {
        let inner =
            unsafe { ffi::vaccel_tf_buffer_new(data.as_ptr() as *mut _, data.len() as u64) };
        assert!(!inner.is_null(), "Memory allocation failure");

        Buffer {
            inner,
            vaccel_owned: false,
        }
    }

    pub unsafe fn from_vaccel_buffer(buffer: *mut ffi::vaccel_tf_buffer) -> Result<Self> {
        let mut size = Default::default();
        let data = ffi::vaccel_tf_buffer_get_data(buffer, &mut size);
        if data.is_null() || size == 0 {
            return Err(Error::InvalidArgument);
        }

        Ok(Buffer {
            inner: buffer,
            vaccel_owned: true,
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let mut size = Default::default();
            let ptr = ffi::vaccel_tf_buffer_get_data(self.inner, &mut size) as *const u8;
            std::slice::from_raw_parts(ptr, size as usize)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            let mut size = Default::default();
            let ptr = ffi::vaccel_tf_buffer_get_data(self.inner, &mut size) as *mut u8;
            std::slice::from_raw_parts_mut(ptr, size as usize)
        }
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
            let mut size = Default::default();
            unsafe { ffi::vaccel_tf_buffer_take_data(self.inner, &mut size) };
        }

        unsafe { ffi::vaccel_tf_buffer_destroy(self.inner) }
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
