// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Handle, Result};
use derive_more::Display;
use std::{
    ffi::{CStr, CString},
    ptr::{self, NonNull},
};

/// Wrapper for the `struct vaccel_tf_node` C object.
#[derive(Debug, Display)]
#[display("{}:{}", self.name().unwrap_or("".to_string()), self.id())]
pub struct Node {
    inner: NonNull<ffi::vaccel_tf_node>,
    owned: bool,
}

impl Node {
    /// Creates a new `Node`.
    pub fn new(name: &str, id: i32) -> Result<Self> {
        let c_name = CString::new(name).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `name` to `CString` [{}]", e))
        })?;

        let mut ptr: *mut ffi::vaccel_tf_node = ptr::null_mut();
        match unsafe { ffi::vaccel_tf_node_new(&mut ptr, c_name.as_ptr(), id) as u32 } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        unsafe { Self::from_ptr_owned(ptr) }
    }

    /// Returns the ID of the `Node`.
    pub fn id(&self) -> i32 {
        unsafe { self.inner.as_ref().id }
    }

    /// Returns the name of the `Node`.
    pub fn name(&self) -> Result<String> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.name.is_null() {
            return Err(Error::EmptyValue);
        }

        match unsafe { CStr::from_ptr(inner.name).to_str() } {
            Ok(n) => Ok(n.to_string()),
            Err(e) => Err(Error::ConversionFailed(format!(
                "Could not convert `name` to `CString` [{}]",
                e
            ))),
        }
    }
}

impl_component_drop!(Node, vaccel_tf_node_delete, inner, owned);

impl_component_handle!(Node, ffi::vaccel_tf_node, inner, owned);
