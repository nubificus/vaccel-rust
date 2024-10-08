// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Resource, Result, VaccelId};
use std::{
    any::Any,
    ffi::{CStr, CString},
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
pub struct TFSavedModel {
    inner: *mut ffi::vaccel_tf_saved_model,
}

impl Default for TFSavedModel {
    fn default() -> Self {
        Self::new()
    }
}

impl TFSavedModel {
    /// Create a new Saved Model object
    pub fn new() -> Self {
        TFSavedModel {
            inner: unsafe { ffi::vaccel_tf_saved_model_new() },
        }
    }

    /// Create a new TFSavedModel from a vaccel_tf_saved_model type
    pub fn from_vaccel(inner: *mut ffi::vaccel_tf_saved_model) -> Self {
        TFSavedModel { inner }
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
    fn set_protobuf(&mut self, data: &[u8]) -> Result<()> {
        match unsafe {
            ffi::vaccel_tf_saved_model_set_model(self.inner, data.as_ptr(), data.len()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Set the in-memory checkpoint data
    fn set_checkpoint(&mut self, data: &[u8]) -> Result<()> {
        match unsafe {
            ffi::vaccel_tf_saved_model_set_checkpoint(self.inner, data.as_ptr(), data.len()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Set the in-memory variable index data
    fn set_var_index(&mut self, data: &[u8]) -> Result<()> {
        match unsafe {
            ffi::vaccel_tf_saved_model_set_var_index(self.inner, data.as_ptr(), data.len()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create the resource from in-memory data
    pub fn from_in_memory(
        mut self,
        protobuf: &[u8],
        checkpoint: &[u8],
        variable_index: &[u8],
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
            Some(unsafe { std::slice::from_raw_parts(ptr, size) })
        } else {
            None
        }
    }

    /// Get the data of the checkpoint file
    pub fn get_checkpoint(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_tf_saved_model_get_checkpoint(self.inner, &mut size) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size) })
        } else {
            None
        }
    }

    /// Get the data of the variable index file
    pub fn get_var_index(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_tf_saved_model_get_var_index(self.inner, &mut size) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size) })
        } else {
            None
        }
    }
}

impl Resource for TFSavedModel {
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
