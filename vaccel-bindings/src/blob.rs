// SPDX-License-Identifier: Apache-2.0

use crate::{c_pointer_to_mut_slice, ffi, Error, Result};
use std::ffi::{CStr, CString};
use vaccel_rpc_proto::resource::Blob as ProtoBlob;

#[derive(Debug)]
pub struct Blob {
    inner: *mut ffi::vaccel_blob,
    owned: bool,
    _buffer: Option<Vec<u8>>,
}

impl Blob {
    pub fn new(path: &str) -> Result<Self> {
        let c_path = CString::new(path).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `path` to `CString` [{}]", e))
        })?;

        let mut inner: *mut ffi::vaccel_blob = std::ptr::null_mut();
        match unsafe { ffi::vaccel_blob_new(&mut inner, c_path.as_ptr()) as u32 } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }
        assert!(!inner.is_null());

        Ok(Blob {
            inner,
            owned: true,
            _buffer: None,
        })
    }

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

        let mut inner: *mut ffi::vaccel_blob = std::ptr::null_mut();
        match unsafe {
            ffi::vaccel_blob_from_buf(
                &mut inner,
                buf.as_ptr(),
                buf.len(),
                false,
                c_name.as_ptr(),
                c_dir.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
                randomize,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }
        assert!(!inner.is_null());

        Ok(Blob {
            inner,
            owned: true,
            _buffer: Some(buf),
        })
    }

    fn set_type(&mut self, type_: u32) -> Result<()> {
        let inner = unsafe { self.inner.as_mut().ok_or(Error::Uninitialized)? };

        inner.type_ = type_;
        Ok(())
    }

    pub fn type_(&self) -> Result<u32> {
        let inner = unsafe { self.inner.as_ref().ok_or(Error::Uninitialized)? };

        Ok(inner.type_)
    }

    pub fn name(&self) -> Result<String> {
        let inner = unsafe { self.inner.as_ref().ok_or(Error::Uninitialized)? };

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

    pub fn path(&self) -> Result<Option<String>> {
        let inner = unsafe { self.inner.as_ref().ok_or(Error::Uninitialized)? };

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

    pub fn data(&self) -> Result<Option<&[u8]>> {
        let inner = unsafe { self.inner.as_ref().ok_or(Error::Uninitialized)? };

        if inner.data.is_null() || inner.size == 0 {
            Ok(None)
        } else {
            let data = unsafe { std::slice::from_raw_parts(inner.data, inner.size) };
            Ok(Some(data))
        }
    }

    pub fn size(&self) -> Result<usize> {
        let inner = unsafe { self.inner.as_ref().ok_or(Error::Uninitialized)? };

        Ok(inner.size)
    }

    /// # Safety
    ///
    /// - `ptr` is expected to be a valid pointer to a blob object allocated
    ///   manually or by the respective vAccel functions
    /// - The pointer must remain valid for the lifetime of the returned Blob
    /// - No other code will free this pointer (Blob takes ownership)
    pub unsafe fn from_ffi(ptr: *mut ffi::vaccel_blob) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::InvalidArgument("`ptr` cannot be `null`".to_string()));
        }

        Ok(Blob {
            inner: ptr,
            owned: true,
            _buffer: None,
        })
    }

    /// # Safety
    ///
    /// - `ptr` is expected to be a valid pointer to a blob object allocated
    ///   manually or by the respective vAccel functions
    /// - The pointer must remain valid for the lifetime of the returned Blob
    /// - The pointer will be freed by other code (not this Blob)
    pub unsafe fn from_ffi_borrowed(ptr: *mut ffi::vaccel_blob) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::InvalidArgument("`ptr` cannot be `null`".to_string()));
        }

        Ok(Blob {
            inner: ptr,
            owned: false,
            _buffer: None,
        })
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_blob {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_blob {
        self.inner
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        if !self.inner.is_null() && self.owned {
            unsafe {
                ffi::vaccel_blob_delete(self.inner);
            }
        }
    }
}

impl TryFrom<&ProtoBlob> for Blob {
    type Error = Error;

    fn try_from(proto_blob: &ProtoBlob) -> Result<Self> {
        let mut blob = Self::from_buf(proto_blob.data.to_owned(), &proto_blob.name, None, false)?;

        blob.set_type(proto_blob.type_)?;
        Ok(blob)
    }
}

impl TryFrom<ProtoBlob> for Blob {
    type Error = Error;

    fn try_from(proto_blob: ProtoBlob) -> Result<Self> {
        let mut blob = Self::from_buf(proto_blob.data, &proto_blob.name, None, false)?;

        blob.set_type(proto_blob.type_)?;
        Ok(blob)
    }
}

impl TryFrom<&Blob> for ProtoBlob {
    type Error = Error;

    fn try_from(blob: &Blob) -> Result<Self> {
        Ok(ProtoBlob {
            type_: blob.type_()? as u32,
            name: blob.name().to_owned()?,
            data: blob.data()?.unwrap_or(&[]).to_owned(),
            size: blob.size()? as u32,
            ..Default::default()
        })
    }
}

impl TryFrom<Blob> for ProtoBlob {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self> {
        ProtoBlob::try_from(&blob)
    }
}

impl TryFrom<&ffi::vaccel_blob> for ProtoBlob {
    type Error = Error;

    fn try_from(blob: &ffi::vaccel_blob) -> Result<Self> {
        let name = unsafe {
            CStr::from_ptr(blob.name).to_str().map_err(|e| {
                Error::ConversionFailed(format!(
                    "Could not convert `blob.name` to `CString` [{}]",
                    e
                ))
            })?
        };
        let data = unsafe { c_pointer_to_mut_slice(blob.data, blob.size).unwrap_or(&mut []) };
        Ok(ProtoBlob {
            type_: blob.type_,
            name: name.to_owned(),
            data: data.to_owned(),
            size: blob.size as u32,
            ..Default::default()
        })
    }
}
