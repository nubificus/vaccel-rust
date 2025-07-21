// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Handle, Result};
use num_enum::{FromPrimitive, IntoPrimitive};
use std::{
    ffi::{CStr, CString},
    ptr::{self, NonNull},
};

/// The blob types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum BlobType {
    File = ffi::VACCEL_BLOB_FILE,
    Buffer = ffi::VACCEL_BLOB_BUFFER,
    Mapped = ffi::VACCEL_BLOB_MAPPED,
    #[num_enum(catch_all)]
    Unknown(u32),
}

/// Wrapper for the `struct vaccel_blob` C object.
#[derive(Debug)]
pub struct Blob {
    inner: NonNull<ffi::vaccel_blob>,
    owned: bool,
    _buffer: Option<Vec<u8>>,
}

impl Blob {
    /// Creates a new `Blob` from an existing filesystem path.
    pub fn new(path: &str) -> Result<Self> {
        let c_path = CString::new(path).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `path` to `CString` [{}]", e))
        })?;

        let mut ptr: *mut ffi::vaccel_blob = ptr::null_mut();
        match unsafe { ffi::vaccel_blob_new(&mut ptr, c_path.as_ptr()) as u32 } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        unsafe { Self::from_ptr_owned(ptr) }
    }

    /// Creates a new `Blob` from a buffer.
    pub fn from_buf(buf: Vec<u8>, name: &str, dir: Option<&str>, randomize: bool) -> Result<Self> {
        let c_name = CString::new(name).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `name` to `CString` [{}]", e))
        })?;

        let c_dir = dir
            .map(|d| {
                CString::new(d).map_err(|e| {
                    Error::ConversionFailed(format!("Could not convert `dir` to `CString` [{}]", e))
                })
            })
            .transpose()?;

        let mut ptr: *mut ffi::vaccel_blob = ptr::null_mut();
        match unsafe {
            ffi::vaccel_blob_from_buf(
                &mut ptr,
                buf.as_ptr(),
                buf.len(),
                false,
                c_name.as_ptr(),
                c_dir.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                randomize,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| Blob {
                inner,
                owned: true,
                _buffer: Some(buf),
            })
            .ok_or(Error::EmptyValue)
    }

    /// Sets the type of the `Blob`.
    #[doc(hidden)]
    pub fn set_type(&mut self, ty: BlobType) {
        unsafe { self.inner.as_mut().type_ = ty.into() };
    }

    /// Returns the type of the `Blob`.
    pub fn type_(&self) -> BlobType {
        BlobType::from(unsafe { self.inner.as_ref().type_ })
    }

    /// Returns the name of the `Blob`.
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

    /// Returns the path of the `Blob`.
    pub fn path(&self) -> Result<Option<String>> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.path.is_null() {
            Ok(None)
        } else {
            match unsafe { CStr::from_ptr(inner.path).to_str() } {
                Ok(p) => Ok(Some(p.to_string())),
                Err(e) => Err(Error::ConversionFailed(format!(
                    "Could not convert `path` to `CString` [{}]",
                    e
                ))),
            }
        }
    }

    /// Returns the data of the `Blob`.
    pub fn data(&self) -> Option<&[u8]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.data.is_null() || inner.size == 0 {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(inner.data, inner.size) })
        }
    }

    /// Returns the size of the `Blob` data.
    pub fn size(&self) -> usize {
        unsafe { self.inner.as_ref().size }
    }
}

impl_component_drop!(Blob, vaccel_blob_delete, inner, owned);

impl_component_handle!(
    Blob,
    ffi::vaccel_blob,
    inner,
    owned,
    extra_vec_fields: {
        _buffer: None,
    }
);
