// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Blob, Error, Result, Session, VaccelId};
use log::warn;
use std::{
    ffi::{c_char, c_void, CString},
    marker::PhantomPinned,
    pin::Pin,
};

#[derive(Debug)]
pub struct Resource {
    inner: ffi::vaccel_resource,
    // vaccel list struct uses self-referential pointers so we need to use `Pin`
    // here to make sure the underlying memory won't move
    _marker: PhantomPinned,
    _blobs: Option<Vec<Blob>>,
}

impl Resource {
    // TODO: Add from_ffi

    /// Create a new resource object
    pub fn new(paths: &[String], res_type: u32) -> Result<Pin<Box<Self>>> {
        let c_paths: Vec<CString> = match paths.iter().map(|s| CString::new(s.as_str())).collect() {
            Ok(p) => p,
            Err(e) => {
                return Err(Error::ConversionFailed(format!(
                    "Could not convert `paths` to `CString`s [{}]",
                    e
                )))
            }
        };
        let mut p: Vec<*const c_char> = c_paths.iter().map(|p| p.as_c_str().as_ptr()).collect();

        let r = Resource {
            // Ensure id is always initialized
            inner: ffi::vaccel_resource {
                id: -1,
                ..Default::default()
            },
            _marker: PhantomPinned,
            _blobs: None,
        };
        let mut boxed = Box::pin(r);

        match unsafe {
            ffi::vaccel_resource_init_multi(
                &mut boxed.as_mut().get_unchecked_mut().inner,
                p.as_mut_ptr(),
                p.len(),
                res_type,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(boxed),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Get the id of the resource
    pub fn id(self: Pin<&Self>) -> VaccelId {
        VaccelId::from(self.get_ref().inner.id)
    }

    /// Returns `true` if the resource has been initialized
    pub fn initialized(self: Pin<&Self>) -> bool {
        self.id().has_id()
    }

    /// Create new resource from blobs
    pub fn from_blobs(blobs: Vec<Blob>, res_type: u32) -> Result<Pin<Box<Self>>> {
        let mut b: Vec<*const ffi::vaccel_blob> =
            blobs.iter().map(|f| f.inner() as *const _).collect();

        let r = Resource {
            // Ensure id is always initialized
            inner: ffi::vaccel_resource {
                id: -1,
                ..Default::default()
            },
            _marker: PhantomPinned,
            _blobs: Some(blobs),
        };
        let mut boxed = Box::pin(r);

        match unsafe {
            ffi::vaccel_resource_init_from_blobs(
                &mut boxed.as_mut().get_unchecked_mut().inner,
                b.as_mut_ptr(),
                b.len(),
                res_type,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(boxed),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Create new resource from in-memory data
    pub fn from_buf(
        data: &[u8],
        res_type: u32,
        filename: Option<&str>,
        mem_only: bool,
    ) -> Result<Pin<Box<Self>>> {
        let r = Resource {
            // Ensure id is always initialized
            inner: ffi::vaccel_resource {
                id: -1,
                ..Default::default()
            },
            _marker: PhantomPinned,
            _blobs: None,
        };
        let mut boxed = Box::pin(r);

        let c_fname = match filename {
            Some(f) => match CString::new(f) {
                Ok(f) => f,
                Err(e) => {
                    return Err(Error::ConversionFailed(format!(
                        "Could not convert `filename` to `CString` [{}]",
                        e
                    )))
                }
            },
            None => CString::new("file").unwrap(),
        };

        match unsafe {
            ffi::vaccel_resource_init_from_buf(
                &mut boxed.as_mut().get_unchecked_mut().inner,
                data.as_ptr() as *mut c_void,
                data.len(),
                res_type,
                c_fname.as_c_str().as_ptr(),
                mem_only,
            )
        } as u32
        {
            ffi::VACCEL_OK => Ok(boxed),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Release resource data
    pub fn release(self: Pin<&mut Self>) -> Result<()> {
        if !self.as_ref().initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe { ffi::vaccel_resource_release(self.inner_mut()) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Register a resource with a session
    pub fn register(self: Pin<&mut Self>, sess: &mut Session) -> Result<()> {
        if !self.as_ref().initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe { ffi::vaccel_resource_register(self.inner_mut(), sess.inner_mut()) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Unregister a resource from a session
    pub fn unregister(self: Pin<&mut Self>, sess: &mut Session) -> Result<()> {
        if !self.as_ref().initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe { ffi::vaccel_resource_unregister(self.inner_mut(), sess.inner_mut()) as u32 }
        {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Get refcount atomically
    /// Use a vaccel function here since bindgen does not support C atomics yet
    /// (see: https://github.com/rust-lang/rust-bindgen/issues/2151)
    pub fn refcount(self: Pin<&Self>) -> Result<u32> {
        if !self.as_ref().initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe { ffi::vaccel_resource_refcount(self.inner()) } {
            rc if rc >= 0 => Ok(rc as u32),
            err => Err(Error::Ffi(-err as u32)),
        }
    }

    pub(crate) fn inner(self: Pin<&Self>) -> &ffi::vaccel_resource {
        &self.get_ref().inner
    }

    pub(crate) fn inner_mut(self: Pin<&mut Self>) -> &mut ffi::vaccel_resource {
        unsafe { &mut self.get_unchecked_mut().inner }
    }

    pub fn blobs(&self) -> Option<&Vec<Blob>> {
        self._blobs.as_ref()
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        // `new_unchecked` is okay because we know this value is never used
        // again after being dropped.
        inner_drop(unsafe { Pin::new_unchecked(self) });
        fn inner_drop(this: Pin<&mut Resource>) {
            if this.as_ref().initialized() && this.release().is_err() {
                warn!("Could not release resource");
            }
        }
    }
}
