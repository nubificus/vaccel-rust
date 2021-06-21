use crate::ffi::VACCEL_OK;
use crate::ffi::{
    vaccel_id_t, vaccel_resource, vaccel_tf_model, vaccel_tf_model_destroy, vaccel_tf_model_get_id,
    vaccel_tf_model_new, vaccel_tf_model_new_from_buffer,
};
use crate::resource::VaccelResource;
use crate::{Error, Result};
use std::any::Any;
use std::ffi::CString;
use std::path::Path;

#[derive(Debug)]
pub struct FrozenModel {
    inner: *mut vaccel_tf_model,
}

impl FrozenModel {
    /// Create a new TensorFlow model from a protobuf file
    ///
    /// Create a new TensorFlow model in the database of the vAccel runtime from
    /// a protobuf file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the protobuf model from which we will create the model
    pub fn new(path: &Path) -> Result<Self> {
        let mut model = Box::new(vaccel_tf_model::default());

        // We create a CString to ensure that the path we pass to libvaccel
        // is null terminated
        let c_str = CString::new(path.as_os_str().to_str().ok_or(Error::InvalidArgument)?)
            .map_err(|_| Error::InvalidArgument)?;

        match unsafe { vaccel_tf_model_new(&mut *model, c_str.as_ptr()) as u32 } {
            VACCEL_OK => Ok(FrozenModel {
                inner: Box::into_raw(model),
            }),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create a new TensorFlow model from a byte array
    ///
    /// Create a new TensorFlow model in the database of the vAccel runtime from
    /// a byte array. This assumes that the byte array contains the data of a
    /// protobuf file
    ///
    /// # Arguments
    ///
    /// * `data` - The slice that holds the data of the protobuf binary file
    pub fn from_buffer(data: &[u8]) -> Result<Self> {
        let mut model = Box::new(vaccel_tf_model::default());

        match unsafe {
            vaccel_tf_model_new_from_buffer(&mut *model, data.as_ptr(), data.len() as u64) as u32
        } {
            VACCEL_OK => Ok(FrozenModel {
                inner: Box::into_raw(model),
            }),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Remove the TensorFlow model from the database of vAccel runtime
    pub fn destroy(&mut self) -> Result<()> {
        match unsafe { vaccel_tf_model_destroy(self.inner) as u32 } {
            VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    pub fn initialized(&self) -> bool {
        if self.inner.is_null() {
            return false;
        }

        unsafe { !(*self.inner).resource.is_null() }
    }

    pub fn inner() -> Option<*mut vaccel_tf_model>
}

impl VaccelResource for FrozenModel {
    fn id(&self) -> vaccel_id_t {
        unsafe { vaccel_tf_model_get_id(self.inner) }
    }

    fn initialized(&self) -> bool {
        self.initialized()
    }

    fn to_mut_vaccel_ptr(&self) -> Option<*mut vaccel_resource> {
        match self.initialized() {
            true => Some((*self.inner).resource),
            false => None,
        }
    }

    fn to_vaccel_ptr(&self) -> Option<*const vaccel_resource> {
        match self.initialized() {
            true => Some((*self.inner).resource),
            false => None,
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
