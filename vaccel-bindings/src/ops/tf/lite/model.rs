// SPDX-License-Identifier: Apache-2.0

use super::Status;
use crate::{
    ffi,
    ops::{Model as ModelTrait, Tensor},
    Error, Handle, Resource, Result, Session,
};
use log::warn;

impl Session {
    /// Loads the model.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    pub fn tflite_model_load(&mut self, resource: &mut Resource) -> Result<()> {
        match unsafe {
            ffi::vaccel_tflite_model_load(self.as_mut_ptr(), resource.as_mut_ptr()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Runs inference using the model.
    ///
    /// This requires that the model has previously been loaded using `load()`.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    pub fn tflite_model_run<T: Tensor + Handle<CType = ffi::vaccel_tflite_tensor>>(
        &mut self,
        resource: &mut Resource,
        in_tensors: &[T],
        nr_out_tensors: usize,
    ) -> Result<(Vec<T>, Status)> {
        let c_in_tensors: Vec<*mut ffi::vaccel_tflite_tensor> =
            in_tensors.iter().map(|n| n.as_ptr() as *mut _).collect();

        let mut c_out_tensors = vec![std::ptr::null_mut(); nr_out_tensors];
        let mut status = Status(0);
        match unsafe {
            ffi::vaccel_tflite_model_run(
                self.as_mut_ptr(),
                resource.as_mut_ptr(),
                c_in_tensors.as_ptr(),
                c_in_tensors.len() as i32,
                c_out_tensors.as_mut_ptr(),
                nr_out_tensors as i32,
                &mut status.0 as *mut _,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok((
                c_out_tensors
                    .into_iter()
                    .map(|ptr| unsafe { T::from_ptr(ptr) })
                    .collect::<Result<Vec<_>>>()?,
                status,
            )),
            err => Err(Error::FfiWithStatus {
                error: err,
                status: status.into(),
            }),
        }
    }

    /// Unloads the model.
    ///
    /// This will unload a model that was previously loaded in memory using
    /// `load()`.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    pub fn tflite_model_unload(&mut self, resource: &mut Resource) -> Result<()> {
        match unsafe {
            ffi::vaccel_tflite_model_unload(self.as_mut_ptr(), resource.as_mut_ptr()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }
}

/// A model abstraction for user-friendly inference operations.
pub struct Model<'a> {
    resource: Resource,
    session: Option<&'a mut Session>,
    loaded: bool,
    nr_out_tensors: usize,
}

impl<'a> Model<'a> {
    /// Sets the model number of output tensors.
    pub fn set_nr_out_tensors(&mut self, val: usize) -> &mut Self {
        self.nr_out_tensors = val;
        self
    }

    /// Sets the model number of output tensors (for chaining with `new()`).
    pub fn with_nr_out_tensors(mut self, val: usize) -> Self {
        self.nr_out_tensors = val;
        self
    }
}

impl<'a> ModelTrait<'a> for Model<'a> {
    type TensorHandle = ffi::vaccel_tflite_tensor;

    fn load<P: AsRef<str>>(path: P, session: &'a mut Session) -> Result<Self> {
        let mut resource = Resource::new([path], ffi::VACCEL_RESOURCE_MODEL)?;
        resource.register(session)?;

        session.tflite_model_load(&mut resource)?;

        Ok(Model {
            resource,
            session: Some(session),
            loaded: true,
            nr_out_tensors: 1,
        })
    }

    fn unload(&mut self) -> Result<()> {
        if !self.loaded {
            return Err(Error::Uninitialized);
        }

        let session = self
            .session
            .take()
            .ok_or(Error::InvalidArgument("Session not set".to_string()))?;
        session.tflite_model_unload(&mut self.resource)?;

        self.resource.unregister(session)?;
        self.loaded = false;

        Ok(())
    }

    fn run<T: Tensor + Handle<CType = Self::TensorHandle>>(
        &mut self,
        in_tensors: &[T],
    ) -> Result<Vec<T>> {
        if !self.loaded {
            return Err(Error::Uninitialized);
        }

        let session = self
            .session
            .as_mut()
            .ok_or(Error::InvalidArgument("Session not set".to_string()))?;

        let (out_tensors, _) =
            session.tflite_model_run(&mut self.resource, in_tensors, self.nr_out_tensors)?;

        Ok(out_tensors)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }
}

impl<'a> Drop for Model<'a> {
    fn drop(&mut self) {
        if self.loaded {
            if let Err(e) = self.unload() {
                warn!("Could not unload model: {}", e);
            }
        }
    }
}
