// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, File, Result, Session, VaccelId};
use std::{
    error::Error as StdError,
    ffi::{c_char, c_void, CString},
    marker::PhantomPinned,
    pin::Pin,
    result::Result as StdResult,
};

#[derive(Debug)]
pub struct Resource {
    inner: ffi::vaccel_resource,
    // vaccel list struct uses self-referential pointers so we need to use `Pin`
    // here to make sure the underlying memory won't move
    _marker: PhantomPinned,
}

impl Resource {
    // TODO: Add from_ffi

    /// Create a new resource object
    pub fn new(paths: &[String], res_type: u32) -> Result<Pin<Box<Self>>> {
        let c_paths = match paths
            .iter()
            .map(|s| Ok(CString::new(s.as_str())?))
            .collect::<StdResult<Vec<CString>, Box<dyn StdError>>>()
        {
            Ok(p) => p,
            Err(_) => return Err(Error::InvalidArgument),
        };
        let mut p: Vec<*const c_char> = c_paths.iter().map(|p| p.as_c_str().as_ptr()).collect();

        let r = Resource {
            inner: ffi::vaccel_resource::default(),
            _marker: PhantomPinned,
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
            err => Err(Error::Runtime(err)),
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

    /// Create new resource from files
    pub fn from_files(files: &[File], res_type: u32) -> Result<Pin<Box<Self>>> {
        let mut f: Vec<*const ffi::vaccel_file> =
            files.iter().map(|f| f.inner() as *const _).collect();

        let r = Resource {
            inner: ffi::vaccel_resource::default(),
            _marker: PhantomPinned,
        };
        let mut boxed = Box::pin(r);

        match unsafe {
            ffi::vaccel_resource_init_from_files(
                &mut boxed.as_mut().get_unchecked_mut().inner,
                f.as_mut_ptr(),
                f.len(),
                res_type,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(boxed),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create new resource from in-memory data
    pub fn from_buf(data: &[u8], res_type: u32, filename: Option<&str>) -> Result<Pin<Box<Self>>> {
        let r = Resource {
            inner: ffi::vaccel_resource::default(),
            _marker: PhantomPinned,
        };
        let mut boxed = Box::pin(r);

        let c_fname = match filename {
            Some(f) => match CString::new(f) {
                Ok(f) => f,
                Err(_) => return Err(Error::InvalidArgument),
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
            )
        } as u32
        {
            ffi::VACCEL_OK => Ok(boxed),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Register a resource to a session
    pub fn register(self: Pin<&mut Self>, sess: &mut Session) -> Result<()> {
        if !self.as_ref().initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe { ffi::vaccel_resource_register(self.inner_mut(), sess.inner_mut()) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
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
            err => Err(Error::Runtime(err)),
        }
    }

    /// Release resource data
    pub fn release(self: Pin<&mut Self>) -> Result<()> {
        if !self.as_ref().initialized() {
            return Err(Error::Uninitialized);
        }

        match unsafe { ffi::vaccel_resource_release(self.inner_mut()) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    pub(crate) fn inner(self: Pin<&Self>) -> &ffi::vaccel_resource {
        &self.get_ref().inner
    }

    pub(crate) fn inner_mut(self: Pin<&mut Self>) -> &mut ffi::vaccel_resource {
        unsafe { &mut self.get_unchecked_mut().inner }
    }
}
