// SPDX-License-Identifier: Apache-2.0

use crate::{c_pointer_to_mut_slice, ffi, Error, Result};
use std::ffi::{c_char, CStr, CString};
use vaccel_rpc_proto::resource::Blob as ProtoBlob;

#[derive(Debug)]
pub struct Blob {
    inner: ffi::vaccel_blob,
    blob_type: u32,
    name: CString,
    path: CString,
    path_owned: bool,
    data_owned: bool,
    data: Vec<u8>,
    size: usize,
}

// TODO: Optimize functions

impl Blob {
    pub fn new(blob_type: u32, name: &str, path: &str, path_owned: bool, data_owned: bool, data: &[u8], size: usize) -> Result<Self> {
        let mut d = data.to_owned();
        let n = CString::new(name).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `name` to `CString` [{}]", e))
        })?;
        let p = CString::new(path).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `path` to `CString` [{}]", e))
        })?;

        Ok(Blob {
            inner: ffi::vaccel_blob {
                type_: blob_type,
                name: n.as_c_str().as_ptr() as *mut c_char,
                path: p.as_c_str().as_ptr() as *mut c_char,
                path_owned,
                data_owned,
                data: if !d.is_empty() {
                    d.as_mut_ptr()
                } else {
                    std::ptr::null_mut()
                },
                size,
            },
            blob_type,
            name: n,
            path: p,
            path_owned,
            data_owned,
            data: d,
            size,
        })
    }

    /// # Safety
    ///
    /// `file_ptr` is expected to be a valid pointer to a file
    /// object allocated manually or by the respective vAccel functions.
    pub unsafe fn from_ffi(blob_ptr: *mut ffi::vaccel_blob) -> Result<Self> {
        let blob = match unsafe { blob_ptr.as_ref() } {
            Some(f) => f,
            None => {
                return Err(Error::InvalidArgument(
                    "`file` cannot be `null`".to_string(),
                ))
            }
        };

        let name = unsafe {
            CStr::from_ptr(blob.name).to_str().map_err(|e| {
                Error::ConversionFailed(format!(
                    "Could not convert `blob.name` to `CString` [{}]",
                    e
                ))
            })?
        };
        let path = unsafe {
            if blob.path.is_null() {
                ""
            } else {
                CStr::from_ptr(blob.path).to_str().map_err(|e| {
                    Error::ConversionFailed(format!(
                        "Could not convert `blob.path` to `CString` [{}]",
                        e
                    ))
                })?
            }
        };
        let data = c_pointer_to_mut_slice(blob.data, blob.size).unwrap_or(&mut []);
        Self::new(blob.type_, name, path, blob.path_owned, blob.data_owned, data, blob.size)
    }

    pub fn set_data(&mut self, d: &mut [u8]) {
        self.data = d.to_owned();
        self.inner.data = self.data.as_mut_ptr();
    }

    pub(crate) fn inner(&self) -> &ffi::vaccel_blob {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_blob {
        &mut self.inner
    }
}

impl From<&ProtoBlob> for Blob {
    fn from(blob: &ProtoBlob) -> Self {
        Self::new(
            blob.type_,
            &blob.name,
            &blob.path,
            blob.path_owned,
            blob.data_owned,
            &blob.data,
            blob.size as usize,
        )
        .unwrap()
    }
}

impl From<&Blob> for ProtoBlob {
    fn from(blob: &Blob) -> Self {
        ProtoBlob {
            type_: blob.blob_type as u32,
            name: blob.name.to_owned().into_string().unwrap(),
            path: blob.path.to_owned().into_string().unwrap(),
            path_owned: blob.path_owned,
            data_owned: blob.data_owned,
            data: blob.data.to_owned(),
            size: blob.size as u32,
            ..Default::default()
        }
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
        let path = unsafe {
            if blob.path.is_null() {
                ""
            } else {
                CStr::from_ptr(blob.path).to_str().map_err(|e| {
                    Error::ConversionFailed(format!(
                            "Could not convert `blob.path` to `CString` [{}]",
                            e
                    ))
                })?
            }
        };
        let data = unsafe { c_pointer_to_mut_slice(blob.data, blob.size).unwrap_or(&mut []) };
        Ok(ProtoBlob {
            type_: blob.type_,
            name: name.to_owned(),
            path: path.to_owned(),
            path_owned: blob.path_owned,
            data_owned: blob.data_owned,
            data: data.to_owned(),
            size: blob.size as u32,
            ..Default::default()
        })
    }
}
