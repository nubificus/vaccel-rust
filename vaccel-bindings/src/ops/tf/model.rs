// SPDX-License-Identifier: Apache-2.0

use super::{Buffer, Node, Status};
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
    pub fn tf_model_load(&mut self, resource: &mut Resource) -> Result<Status> {
        let mut status = Status::new(0, "")?;
        match unsafe {
            ffi::vaccel_tf_model_load(
                self.as_mut_ptr(),
                resource.as_mut_ptr(),
                status.as_mut_ptr(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(status),
            err => Err(Error::FfiWithStatus {
                error: err,
                status: status.try_into()?,
            }),
        }
    }

    /// Runs inference using the model.
    ///
    /// This requires that the model has previously been loaded using `load()`.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    pub fn tf_model_run<T: Tensor + Handle<CType = ffi::vaccel_tf_tensor>>(
        &mut self,
        resource: &mut Resource,
        run_options: Option<&Buffer>,
        in_nodes: &[Node],
        in_tensors: &[T],
        out_nodes: &[Node],
    ) -> Result<(Vec<T>, Status)> {
        let c_run_options = run_options
            .map(|opts| opts.as_ptr())
            .unwrap_or(std::ptr::null());

        let c_in_nodes: Vec<ffi::vaccel_tf_node> =
            in_nodes.iter().map(|n| unsafe { *n.as_ptr() }).collect();
        let c_out_nodes: Vec<ffi::vaccel_tf_node> =
            out_nodes.iter().map(|n| unsafe { *n.as_ptr() }).collect();

        let c_in_tensors: Vec<*mut ffi::vaccel_tf_tensor> =
            in_tensors.iter().map(|n| n.as_ptr() as *mut _).collect();

        let mut c_out_tensors = vec![std::ptr::null_mut(); out_nodes.len()];
        let mut status = Status::new(0, "")?;
        match unsafe {
            ffi::vaccel_tf_model_run(
                self.as_mut_ptr(),
                resource.as_mut_ptr(),
                c_run_options,
                c_in_nodes.as_ptr(),
                c_in_tensors.as_ptr(),
                in_nodes.len() as i32,
                c_out_nodes.as_ptr(),
                c_out_tensors.as_mut_ptr(),
                out_nodes.len() as i32,
                status.as_mut_ptr(),
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
                status: status.try_into()?,
            }),
        }
    }

    /// Unloads the model.
    ///
    /// This will unload a model that was previously loaded in memory using
    /// `load()`.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    pub fn tf_model_unload(&mut self, resource: &mut Resource) -> Result<Status> {
        let mut status = Status::new(0, "")?;
        match unsafe {
            ffi::vaccel_tf_model_unload(
                self.as_mut_ptr(),
                resource.as_mut_ptr(),
                status.as_mut_ptr(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(status),
            err => Err(Error::FfiWithStatus {
                error: err,
                status: status.try_into()?,
            }),
        }
    }
}

/// A model abstraction for user-friendly inference operations.
pub struct Model<'a> {
    resource: Resource,
    session: Option<&'a mut Session>,
    loaded: bool,
    run_options: Option<Buffer>,
    in_nodes: Option<Vec<Node>>,
    out_nodes: Option<Vec<Node>>,
}

impl<'a> Model<'a> {
    /// Sets the model run options.
    pub fn set_run_options(&mut self, opts: Buffer) -> &mut Self {
        self.run_options = Some(opts);
        self
    }

    /// Sets the model in nodes.
    pub fn set_in_nodes(&mut self, nodes: Vec<Node>) -> &mut Self {
        self.in_nodes = Some(nodes);
        self
    }

    /// Sets the model out nodes.
    pub fn set_out_nodes(&mut self, nodes: Vec<Node>) -> &mut Self {
        self.out_nodes = Some(nodes);
        self
    }

    /// Sets the model run options (for chaining with `new()`).
    pub fn with_run_options(mut self, opts: Buffer) -> Self {
        self.run_options = Some(opts);
        self
    }

    /// Sets the model nodes (for chaining with `new()`).
    pub fn with_nodes(mut self, input_nodes: Vec<Node>, output_nodes: Vec<Node>) -> Self {
        self.in_nodes = Some(input_nodes);
        self.out_nodes = Some(output_nodes);
        self
    }
}

impl<'a> ModelTrait<'a> for Model<'a> {
    type TensorHandle = ffi::vaccel_tf_tensor;

    fn load<P: AsRef<str>>(path: P, session: &'a mut Session) -> Result<Self> {
        let mut resource = Resource::new([path], ffi::VACCEL_RESOURCE_MODEL)?;
        resource.register(session)?;

        session.tf_model_load(&mut resource)?;

        Ok(Model {
            resource,
            session: Some(session),
            loaded: true,
            run_options: None,
            in_nodes: None,
            out_nodes: None,
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
        session.tf_model_unload(&mut self.resource)?;

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

        let in_nodes = self
            .in_nodes
            .as_ref()
            .ok_or(Error::InvalidArgument("Input nodes not set".to_string()))?;
        let out_nodes = self
            .out_nodes
            .as_ref()
            .ok_or(Error::InvalidArgument("Output nodes not set".to_string()))?;

        let session = self
            .session
            .as_mut()
            .ok_or(Error::InvalidArgument("Session not set".to_string()))?;

        let (out_tensors, _) = session.tf_model_run(
            &mut self.resource,
            self.run_options.as_ref(),
            in_nodes,
            in_tensors,
            out_nodes,
        )?;

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
