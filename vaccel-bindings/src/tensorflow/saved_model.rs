use crate::ffi;
use crate::VaccelId;
use crate::{Error, Result};
use std::any::Any;
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct SavedModel {
    inner: *mut ffi::vaccel_tf_saved_model,
}

impl SavedModel {
    /// Create a new Saved Model object
    pub fn new() -> Self {
        SavedModel {
            inner: unsafe { ffi::vaccel_tf_saved_model_new() },
        }
    }

    /// Create a new SavedModel from a vaccel saved model type
    pub fn from_vaccel(inner: *mut ffi::vaccel_tf_saved_model) -> Self {
        SavedModel { inner }
    }

    /// Get the id of the model
    pub fn id(&self) -> VaccelId {
        let inner = unsafe { ffi::vaccel_tf_saved_model_id(self.inner) };
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

        match unsafe { ffi::vaccel_tf_saved_model_destroy(self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Set the export directory path
    fn set_path(&mut self, path: &Path) -> Result<()> {
        let c_path = CString::new(path.as_os_str().to_str().ok_or(Error::InvalidArgument)?)
            .map_err(|_| Error::InvalidArgument)?;

        match unsafe { ffi::vaccel_tf_saved_model_set_path(self.inner, c_path.into_raw()) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create the resource from a previously exported saved model
    ///
    /// * Args
    ///
    /// `path` - The path in the filesystem to the export directory
    pub fn from_export_dir(mut self, path: &Path) -> Result<Self> {
        self.set_path(path)?;

        match unsafe { ffi::vaccel_tf_saved_model_register(self.inner) } as u32 {
            ffi::VACCEL_OK => Ok(self),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Set the in-memory protobuf data
    fn set_protobuf(&mut self, mut data: Vec<u8>) -> Result<()> {
        data.shrink_to_fit();
        let mem = data.leak();

        match unsafe {
            ffi::vaccel_tf_saved_model_set_model(self.inner, mem.as_mut_ptr(), mem.len() as u64)
                as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Set the in-memory checkpoint data
    fn set_checkpoint(&mut self, mut data: Vec<u8>) -> Result<()> {
        data.shrink_to_fit();
        let mem = data.leak();

        match unsafe {
            ffi::vaccel_tf_saved_model_set_checkpoint(
                self.inner,
                mem.as_mut_ptr(),
                mem.len() as u64,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Set the in-memory variable index data
    fn set_var_index(&mut self, mut data: Vec<u8>) -> Result<()> {
        data.shrink_to_fit();
        let mem = data.leak();

        match unsafe {
            ffi::vaccel_tf_saved_model_set_var_index(self.inner, mem.as_mut_ptr(), mem.len() as u64)
                as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create the resource from in-memory data
    pub fn from_in_memory(
        mut self,
        protobuf: Vec<u8>,
        checkpoint: Vec<u8>,
        variable_index: Vec<u8>,
    ) -> Result<Self> {
        self.set_protobuf(protobuf)?;
        self.set_checkpoint(checkpoint)?;
        self.set_var_index(variable_index)?;

        match unsafe { ffi::vaccel_tf_saved_model_register(self.inner) } as u32 {
            ffi::VACCEL_OK => Ok(self),
            err => Err(Error::Runtime(err)),
        }
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_tf_saved_model {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_tf_saved_model {
        self.inner
    }

    /// Get the path of the export directory if it exists
    pub fn get_path(&self) -> Option<PathBuf> {
        let path_str = match unsafe {
            CStr::from_ptr(ffi::vaccel_tf_saved_model_get_path(self.inner)).to_str()
        } {
            Ok(s) => s,
            Err(_) => return None,
        };

        Some(PathBuf::from(path_str))
    }

    /// Get the data of the Graph protobuf file
    pub fn get_protobuf(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_tf_saved_model_get_model(self.inner, &mut size) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size as usize) })
        } else {
            None
        }
    }

    /// Get the data of the checkpoint file
    pub fn get_checkpoint(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_tf_saved_model_get_checkpoint(self.inner, &mut size) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size as usize) })
        } else {
            None
        }
    }

    /// Get the data of the variable index file
    pub fn get_var_index(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_tf_saved_model_get_var_index(self.inner, &mut size) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size as usize) })
        } else {
            None
        }
    }
}

impl crate::resource::Resource for SavedModel {
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
