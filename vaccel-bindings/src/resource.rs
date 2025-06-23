// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Blob, Error, Handle, Result, Session, VaccelId};
use log::warn;
use std::{
    ffi::{c_char, c_void, CString},
    ptr::{self, NonNull},
};

/// Wrapper for the `struct vaccel_resource` C object.
#[derive(Debug)]
pub struct Resource {
    inner: NonNull<ffi::vaccel_resource>,
    owned: bool,
    blobs: Option<Vec<Blob>>,
}

impl Resource {
    /// Creates a new `Resource`.
    pub fn new<I, P>(paths: I, res_type: u32) -> Result<Self>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<str>,
    {
        let c_paths: Vec<CString> = paths
            .into_iter()
            .map(|s| CString::new(s.as_ref()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                Error::ConversionFailed(format!("Could not convert `paths` to `CString`s [{}]", e))
            })?;

        let mut c_paths_ptrs: Vec<*const c_char> = c_paths.iter().map(|p| p.as_ptr()).collect();
        let mut ptr: *mut ffi::vaccel_resource = ptr::null_mut();
        match unsafe {
            ffi::vaccel_resource_multi_new(
                &mut ptr,
                c_paths_ptrs.as_mut_ptr(),
                c_paths_ptrs.len(),
                res_type,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        unsafe { Self::from_ptr_owned(ptr) }
    }

    /// Creates a new `Resource` from blobs.
    pub fn from_blobs(blobs: Vec<Blob>, res_type: u32) -> Result<Self> {
        let mut c_blobs_ptrs: Vec<*const ffi::vaccel_blob> =
            blobs.iter().map(|b| b.as_ptr()).collect();

        let mut ptr: *mut ffi::vaccel_resource = ptr::null_mut();
        match unsafe {
            ffi::vaccel_resource_from_blobs(
                &mut ptr,
                c_blobs_ptrs.as_mut_ptr(),
                c_blobs_ptrs.len(),
                res_type,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| Resource {
                inner,
                owned: true,
                blobs: Some(blobs),
            })
            .ok_or(Error::EmptyValue)
    }

    /// Creates a new `Resource` from in-memory data.
    pub fn from_buf(
        data: &[u8],
        res_type: u32,
        filename: Option<&str>,
        mem_only: bool,
    ) -> Result<Self> {
        let c_fname = CString::new(filename.unwrap_or("file")).map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `filename` to `CString` [{}]", e))
        })?;

        let mut ptr: *mut ffi::vaccel_resource = ptr::null_mut();
        match unsafe {
            ffi::vaccel_resource_from_buf(
                &mut ptr,
                data.as_ptr() as *mut c_void,
                data.len(),
                res_type,
                c_fname.as_ptr(),
                mem_only,
            )
        } as u32
        {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        unsafe { Self::from_ptr_owned(ptr) }
    }

    /// Returns the ID of the `Resource`.
    pub fn id(&self) -> VaccelId {
        VaccelId::from(unsafe { self.inner.as_ref().id })
    }

    /// Returns the remote ID of the `Resource`.
    pub fn remote_id(&self) -> VaccelId {
        VaccelId::from(unsafe { self.inner.as_ref().remote_id })
    }

    /// Returns the blobs of the `Resource`.
    pub fn blobs(&self) -> Option<&[Blob]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.blobs.is_null() || inner.nr_blobs == 0 {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(inner.blobs as *const _, inner.nr_blobs) })
        }
    }

    /// Returns `true` if the `Resource` has been initialized.
    pub fn initialized(&self) -> bool {
        self.id().has_id()
    }

    /// Registers a `Resource` with a `Session`.
    pub fn register(&mut self, sess: &mut Session) -> Result<()> {
        if !self.initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe {
            ffi::vaccel_resource_register(self.inner.as_mut(), sess.as_mut_ptr()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Unregisters a `Resource` from a `Session`.
    pub fn unregister(&mut self, sess: &mut Session) -> Result<()> {
        if !self.initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe {
            ffi::vaccel_resource_unregister(self.inner.as_mut(), sess.as_mut_ptr()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Returns the refcount of the `Resource` atomically.
    ///
    /// Uses a vaccel function instead of accessing the C field directly,
    /// because bindgen does not support C atomics yet
    /// (see: https://github.com/rust-lang/rust-bindgen/issues/2151).
    pub fn refcount(&self) -> Result<u32> {
        if !self.initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe { ffi::vaccel_resource_refcount(self.inner.as_ref()) } {
            rc if rc >= 0 => Ok(rc as u32),
            err => Err(Error::Ffi(-err as u32)),
        }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        if self.owned && self.initialized() {
            let ret = unsafe { ffi::vaccel_resource_delete(self.inner.as_ptr()) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete Resource inner: {}", ret);
            }
        }
    }
}

impl_component_handle!(
    Resource,
    ffi::vaccel_resource,
    inner,
    owned,
    extra_vec_fields: {
        blobs: None,
    }
);
