use crate::{ffi, Error, Resource, Result, VaccelId};
use std::{
    any::Any,
    ffi::{CStr, CString},
    path::{Path, PathBuf},
    ptr,
};

#[derive(Debug, PartialEq)]
pub struct SingleModel {
    inner: *mut ffi::vaccel_single_model,
}

impl Default for SingleModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SingleModel {
    /// Create a new Single Model object
    pub fn new() -> Self {
        SingleModel {
            inner: unsafe { ffi::vaccel_single_model_new() },
        }
    }

    /// Create a new SingleModel from a vaccel_single_model type
    pub fn from_vaccel(inner: *mut ffi::vaccel_single_model) -> Self {
        SingleModel { inner }
    }

    /// Get the id of the model
    pub fn id(&self) -> VaccelId {
        let inner = unsafe { ffi::vaccel_single_model_get_id(self.inner) };
        VaccelId::from(inner)
    }

    /// Returns `true` if the model has been initialized
    pub fn initialized(&self) -> bool {
        self.id().has_id()
    }

    pub fn destroy(&mut self) -> Result<()> {
        if !self.initialized() {
            return Ok(());
        }

        match unsafe { ffi::vaccel_single_model_destroy(self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Set the export directory path
    fn set_path(&mut self, path: &Path) -> Result<()> {
        let c_path = CString::new(path.as_os_str().to_str().ok_or(Error::InvalidArgument)?)
            .map_err(|_| Error::InvalidArgument)?;

        match unsafe { ffi::vaccel_single_model_set_path(self.inner, c_path.into_raw()) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create the resource from a previously exported single_model
    ///
    /// * Args
    ///
    /// `path` - The path in the filesystem to the export directory
    pub fn from_export_dir(mut self, path: &Path) -> Result<Self> {
        self.set_path(path)?;

        match unsafe { ffi::vaccel_single_model_register(self.inner) } as u32 {
            ffi::VACCEL_OK => Ok(self),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Set the in-memory model file
    fn set_file(&mut self, data: &[u8]) -> Result<()> {
        match unsafe {
            ffi::vaccel_single_model_set_file(self.inner, ptr::null(), data.as_ptr(), data.len())
                as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create the resource from in-memory data
    pub fn from_in_memory(mut self, file: &[u8]) -> Result<Self> {
        self.set_file(file)?;

        match unsafe { ffi::vaccel_single_model_register(self.inner) } as u32 {
            ffi::VACCEL_OK => Ok(self),
            err => Err(Error::Runtime(err)),
        }
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_single_model {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_single_model {
        self.inner
    }

    /// Get the path of the export directory if it exists
    pub fn get_path(&self) -> Option<PathBuf> {
        let path_str =
            match unsafe { CStr::from_ptr(ffi::vaccel_single_model_get_path(self.inner)).to_str() }
            {
                Ok(s) => s,
                Err(_) => return None,
            };

        Some(PathBuf::from(path_str))
    }

    /// Get the data of the model file
    pub fn get_file(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_single_model_get_file(self.inner, &mut size) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size) })
        } else {
            None
        }
    }
}

impl Resource for SingleModel {
    fn id(&self) -> VaccelId {
        self.id()
    }

    fn initialized(&self) -> bool {
        self.initialized()
    }

    fn to_vaccel_ptr(&self) -> Option<*const ffi::vaccel_resource> {
        if !self.initialized() {
            None
        } else {
            let resource = unsafe { (*self.inner).resource };
            Some(resource)
        }
    }

    fn to_mut_vaccel_ptr(&self) -> Option<*mut ffi::vaccel_resource> {
        if !self.initialized() {
            None
        } else {
            let resource = unsafe { (*self.inner).resource };
            Some(resource)
        }
    }

    fn destroy(&mut self) -> Result<()> {
        self.destroy()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
