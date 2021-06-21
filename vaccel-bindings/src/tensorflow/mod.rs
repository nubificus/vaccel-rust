use crate::ffi;
use std::ffi::CStr;
use std::fmt;

pub mod saved_model;

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

#[derive(Default)]
pub struct Node {
    inner: ffi::vaccel_tf_node,
}

impl Node {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> i64 {
        self.inner.id
    }

    pub fn name(&self) -> String {
        if self.inner.name.is_null() {
            return String::new();
        }

        let cmsg = unsafe { CStr::from_ptr(self.inner.name) };
        cmsg.to_str().unwrap_or("").to_owned()
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.name(), self.id())
    }

    pub(crate) fn inner(&self) -> &ffi::vaccel_tf_node {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_tf_node {
        &mut self.inner
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Default)]
pub struct Buffer {
    inner: ffi::vaccel_tf_buffer,
}

impl Buffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.inner.data as *const u8, self.inner.size as usize)
        }
    }

    pub(crate) fn inner(&self) -> &ffi::vaccel_tf_buffer {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_tf_buffer {
        &mut self.inner
    }
}
