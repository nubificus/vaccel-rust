// SPDX-License-Identifier: Apache-2.0

use crate::{c_pointer_to_mut_slice, ffi, Error, Result};
use std::ffi::{c_char, CStr, CString};
use vaccel_rpc_proto::resource::File as ProtoFile;

#[derive(Debug)]
pub struct File {
    inner: ffi::vaccel_file,
    name: CString,
    path: CString,
    path_owned: bool,
    data: Vec<u8>,
    size: usize,
}

// TODO: Optimize functions

impl File {
    pub fn new(name: &str, path: &str, path_owned: bool, data: &[u8], size: usize) -> Result<Self> {
        let mut d = data.to_owned();
        let n = CString::new(name).map_err(|_| Error::InvalidArgument)?;
        let p = CString::new(path).map_err(|_| Error::InvalidArgument)?;
        Ok(File {
            inner: ffi::vaccel_file {
                name: n.as_c_str().as_ptr() as *mut c_char,
                path: p.as_c_str().as_ptr() as *mut c_char,
                path_owned,
                data: if !d.is_empty() {
                    d.as_mut_ptr()
                } else {
                    std::ptr::null_mut()
                },
                size,
            },
            name: n,
            path: p,
            path_owned,
            data: d,
            size,
        })
    }

    /// # Safety
    ///
    /// `file_ptr` is expected to be a valid pointer to a file
    /// object allocated manually or by the respective vAccel functions.
    pub unsafe fn from_ffi(file_ptr: *mut ffi::vaccel_file) -> Result<Self> {
        let file = match unsafe { file_ptr.as_ref() } {
            Some(f) => f,
            None => return Err(Error::InvalidArgument),
        };

        let name = unsafe {
            CStr::from_ptr(file.name)
                .to_str()
                .map_err(|_| Error::InvalidArgument)?
        };
        let path = unsafe {
            CStr::from_ptr(file.path)
                .to_str()
                .map_err(|_| Error::InvalidArgument)?
        };
        let data = c_pointer_to_mut_slice(file.data, file.size).unwrap_or(&mut []);
        Self::new(name, path, file.path_owned, data, file.size)
    }

    pub fn set_data(&mut self, d: &mut [u8]) {
        self.data = d.to_owned();
        self.inner.data = self.data.as_mut_ptr();
    }

    pub(crate) fn inner(&self) -> &ffi::vaccel_file {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_file {
        &mut self.inner
    }
}

impl From<&ProtoFile> for File {
    fn from(file: &ProtoFile) -> Self {
        Self::new(
            &file.name,
            &file.path,
            file.path_owned,
            &file.data,
            file.size as usize,
        )
        .unwrap()
    }
}

impl From<&File> for ProtoFile {
    fn from(file: &File) -> Self {
        ProtoFile {
            name: file.name.to_owned().into_string().unwrap(),
            path: file.path.to_owned().into_string().unwrap(),
            path_owned: file.path_owned,
            data: file.data.to_owned(),
            size: file.size as u32,
            ..Default::default()
        }
    }
}

impl TryFrom<&ffi::vaccel_file> for ProtoFile {
    type Error = Error;

    fn try_from(file: &ffi::vaccel_file) -> Result<Self> {
        let name = unsafe {
            CStr::from_ptr(file.name)
                .to_str()
                .map_err(|_| Error::InvalidArgument)?
        };
        let path = unsafe {
            CStr::from_ptr(file.path)
                .to_str()
                .map_err(|_| Error::InvalidArgument)?
        };
        let data = unsafe { c_pointer_to_mut_slice(file.data, file.size).unwrap_or(&mut []) };
        Ok(ProtoFile {
            name: name.to_owned(),
            path: path.to_owned(),
            path_owned: file.path_owned,
            data: data.to_owned(),
            size: file.size as u32,
            ..Default::default()
        })
    }
}
