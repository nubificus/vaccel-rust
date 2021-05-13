use crate::resource::VaccelResource;
use crate::{
    vaccel_id_t, vaccel_resource, vaccel_tf_model, vaccel_tf_model_destroy, vaccel_tf_model_get_id,
    vaccel_tf_model_new, vaccel_tf_model_new_from_buffer, VACCEL_ENOENT, VACCEL_OK,
};
use crate::{Error, Result};
use std::any::Any;
use std::path::Path;

impl vaccel_tf_model {
    /// Create a new TensorFlow model from a protobuf file
    ///
    /// Create a new TensorFlow model in the database of the vAccel runtime from
    /// a protobuf file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the protobuf model from which we will create the model
    ///
    /// # Examples
    ///
    /// ```
    /// use vaccel_bindings::vaccel_tf_model;
    /// use std::path::Path;
    ///
    /// let pb_path = Path::new("/path/to/model.pb");
    /// let model = vaccel_tf_model::new(pb_path);
    /// ...
    /// ```
    pub fn new(path: &Path) -> Result<Box<Self>> {
        let mut model = Box::new(vaccel_tf_model::default());

        #[cfg(target_arch = "x86_64")]
        let path_str = match path.to_str() {
            Some(s) => s.as_ptr() as *const i8,
            None => return Err(Error::Runtime(VACCEL_ENOENT)),
        };

        #[cfg(target_arch = "aarch64")]
        let path_str = match path.to_str() {
            Some(s) => s.as_ptr(),
            None => return Err(Error::Runtime(VACCEL_ENOENT)),
        };

        match unsafe { vaccel_tf_model_new(&mut *model, path_str) as u32 } {
            VACCEL_OK => Ok(model),
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
    ///
    /// # Examples
    ///
    /// ```
    /// use vaccel_bindings::vaccel_tf_model;
    /// use std::path::Path;
    ///
    /// let path = Path::new("/path/to/model.pb");
    ///
    /// // Read file to buffer
    /// let mut file = File::open(path).map_err(|_| VACCEL_EIO)?;
    /// let mut buff = Vec::new();
    /// file.read_to_end(&mut buff).map_err(|_| VACCEL_EIO)?;
    ///
    /// // Create model from file's data
    /// let model = vaccel_tf_model::from_buffer(buff);
    /// ```
    pub fn from_buffer(data: &[u8]) -> Result<Box<Self>> {
        let mut model = Box::new(vaccel_tf_model::default());

        match unsafe {
            vaccel_tf_model_new_from_buffer(&mut *model, data.as_ptr(), data.len() as u64) as u32
        } {
            VACCEL_OK => Ok(model),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Remove the TensorFlow model from the database of vAccel runtime
    ///
    /// # Examples
    ///
    /// ```
    /// use vaccel_bindings::vaccel_tf_model;
    /// use std::path::Path;
    ///
    /// let path = Path::new("/path/to/model.pb");
    ///
    /// let model = vaccel_tf_model::new(path).unwrap();
    /// ...
    ///
    /// assert(model.destroy().is_ok());
    /// ```
    pub fn destroy(&mut self) -> Result<()> {
        match unsafe { vaccel_tf_model_destroy(self) as u32 } {
            VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}

impl VaccelResource for vaccel_tf_model {
    fn id(&self) -> Option<vaccel_id_t> {
        match unsafe { vaccel_tf_model_get_id(self) } {
            -1 => None,
            id => Some(id),
        }
    }

    fn initialized(&self) -> bool {
        !self.resource.is_null()
    }

    fn to_mut_vaccel_ptr(&self) -> Option<*mut vaccel_resource> {
        match self.initialized() {
            true => Some(self.resource),
            false => None,
        }
    }

    fn to_vaccel_ptr(&self) -> Option<*const vaccel_resource> {
        match self.initialized() {
            true => Some(self.resource),
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
