// SPDX-License-Identifier: Apache-2.0

use super::{Buffer, DataType, Node, Status, Tensor, TensorAny, TensorType};
use crate::{
    ffi,
    ops::{ModelInitialize, ModelLoadUnload, ModelRun},
    Error, Resource, Result, Session,
};
use log::warn;
use protobuf::Enum;
use std::{marker::PhantomPinned, pin::Pin};
use vaccel_rpc_proto::tensorflow::{TFDataType, TFTensor};

pub struct InferenceArgs {
    run_options: *const ffi::vaccel_tf_buffer,

    in_nodes: Vec<ffi::vaccel_tf_node>,
    in_tensors: Vec<*const ffi::vaccel_tf_tensor>,

    out_nodes: Vec<ffi::vaccel_tf_node>,
}

impl Default for InferenceArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceArgs {
    pub fn new() -> Self {
        InferenceArgs {
            run_options: std::ptr::null(),
            in_nodes: vec![],
            in_tensors: vec![],
            out_nodes: vec![],
        }
    }

    pub fn set_run_options(&mut self, run_opts: Option<&Buffer>) {
        if let Some(opts) = run_opts {
            self.run_options = opts.inner();
        }
    }

    pub fn add_input(&mut self, node: &Node, tensor: &dyn TensorAny) -> Result<()> {
        self.in_nodes.push(unsafe { *node.inner() });
        self.in_tensors.push(tensor.inner()?);
        Ok(())
    }

    pub fn request_output(&mut self, node: &Node) {
        self.out_nodes.push(unsafe { *node.inner() });
    }
}

impl Drop for InferenceArgs {
    fn drop(&mut self) {
        while let Some(tensor_ptr) = self.in_tensors.pop() {
            if tensor_ptr.is_null() {
                continue;
            }
            let ret = unsafe { ffi::vaccel_tf_tensor_delete(tensor_ptr as *mut _) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete TF tensor: {}", ret);
            }
        }
    }
}

pub struct InferenceResult {
    out_tensors: Vec<*mut ffi::vaccel_tf_tensor>,
    pub status: Status,
}

impl InferenceResult {
    pub fn new(len: usize) -> Self {
        let out_tensors = vec![std::ptr::null_mut(); len];

        InferenceResult {
            out_tensors,
            status: Status::default(),
        }
    }

    pub fn from_vec(tensors: Vec<*mut ffi::vaccel_tf_tensor>) -> Self {
        InferenceResult {
            out_tensors: tensors,
            status: Status::default(),
        }
    }

    pub fn take_output<T: TensorType>(&mut self, id: usize) -> Result<Tensor<T>> {
        if id >= self.out_tensors.len() {
            return Err(Error::OutOfBounds);
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::EmptyValue);
        }

        if unsafe { DataType::from_int((*t).data_type) } != T::data_type() {
            return Err(Error::InvalidArgument("Invalid `data_type`".to_string()));
        }

        let tensor: Tensor<T> = unsafe { Tensor::from_ffi(t)? };
        self.out_tensors[id] = std::ptr::null_mut();

        Ok(tensor)
    }

    pub fn to_grpc_output(&self, id: usize) -> Result<TFTensor> {
        if id >= self.out_tensors.len() {
            return Err(Error::OutOfBounds);
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::EmptyValue);
        }

        unsafe {
            Ok(TFTensor {
                dims: std::slice::from_raw_parts((*t).dims, (*t).nr_dims as usize).to_vec(),
                type_: TFDataType::from_i32((*t).data_type as i32).unwrap().into(),
                data: std::slice::from_raw_parts((*t).data as *mut u8, (*t).size).to_vec(),
                ..Default::default()
            })
        }
    }
}

impl Drop for InferenceResult {
    fn drop(&mut self) {
        while let Some(tensor_ptr) = self.out_tensors.pop() {
            if tensor_ptr.is_null() {
                continue;
            }
            let ret = unsafe { ffi::vaccel_tf_tensor_delete(tensor_ptr as *mut _) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete TF tensor: {}", ret);
            }
        }
    }
}

pub struct Model<'a> {
    inner: Pin<&'a mut Resource>,
    _marker: PhantomPinned,
}

impl<'a> ModelInitialize<'a> for Model<'a> {
    fn new(inner: Pin<&'a mut Resource>) -> Pin<Box<Self>> {
        Box::pin(Self {
            inner,
            _marker: PhantomPinned,
        })
    }
}

impl<'a> ModelRun<'a> for Model<'a> {
    type RunArgs = InferenceArgs;
    type RunResult = InferenceResult;

    /// Run a TensorFlow session
    ///
    /// This will run using a TensorFlow session that has been previously loaded
    /// using `load()`.
    ///
    fn run(
        self: Pin<&mut Self>,
        sess: &mut Session,
        args: &mut Self::RunArgs,
    ) -> Result<Self::RunResult> {
        let mut result = InferenceResult::new(args.out_nodes.len());
        match unsafe {
            ffi::vaccel_tf_session_run(
                sess.inner_mut(),
                self.inner_mut().inner_mut(),
                args.run_options,
                args.in_nodes.as_ptr(),
                args.in_tensors.as_ptr() as *const *mut ffi::vaccel_tf_tensor,
                args.in_nodes.len() as i32,
                args.out_nodes.as_ptr(),
                result.out_tensors.as_mut_ptr(),
                args.out_nodes.len() as i32,
                result.status.inner_mut(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(result),
            err => Err(Error::FfiWithStatus {
                error: err,
                status: result.status.clone().into(),
            }),
        }
    }

    fn inner_mut(self: Pin<&mut Self>) -> Pin<&mut Resource> {
        unsafe { self.get_unchecked_mut().inner.as_mut() }
    }
}

impl<'a> ModelLoadUnload<'a> for Model<'a> {
    type LoadUnloadResult = Status;

    /// Load a TensorFlow session from a TFSavedModel
    ///
    /// The TensorFlow model must have been created and registered to
    /// a session. The operation will load the graph and keep the graph
    /// TensorFlow representation in the model struct
    ///
    /// # Arguments
    ///
    /// * `session` - The session in the context of which we perform the operation. The model needs
    ///   to be registered with this session.
    ///
    fn load(self: Pin<&mut Self>, sess: &mut Session) -> Result<Self::LoadUnloadResult> {
        let mut status = Status::default();
        match unsafe {
            ffi::vaccel_tf_session_load(
                sess.inner_mut(),
                self.inner_mut().inner_mut(),
                status.inner_mut(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(status),
            err => Err(Error::FfiWithStatus {
                error: err,
                status: status.into(),
            }),
        }
    }

    /// Delete a TensorFlow session
    ///
    /// This will unload a TensorFlow session that was previously loaded in memory
    /// using `load()`.
    fn unload(self: Pin<&mut Self>, sess: &mut Session) -> Result<Self::LoadUnloadResult> {
        let mut status = Status::default();
        match unsafe {
            ffi::vaccel_tf_session_delete(
                sess.inner_mut(),
                self.inner_mut().inner_mut(),
                status.inner_mut(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(status),
            err => Err(Error::FfiWithStatus {
                error: err,
                status: status.into(),
            }),
        }
    }
}
