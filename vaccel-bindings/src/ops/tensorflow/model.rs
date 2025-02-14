// SPDX-License-Identifier: Apache-2.0

use super::{Buffer, DataType, Node, Status, Tensor, TensorAny, TensorType};
use crate::{
    ffi,
    ops::{ModelInitialize, ModelLoadUnload, ModelRun},
    Error, Resource, Result, Session,
};
use protobuf::Enum;
use std::{marker::PhantomPinned, pin::Pin};
use vaccel_rpc_proto::tensorflow::{TFDataType, TFNode, TFTensor, TensorflowModelRunRequest};

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

    pub fn set_run_options(&mut self, run_opts: &Buffer) {
        self.run_options = run_opts.inner();
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

impl From<InferenceArgs> for TensorflowModelRunRequest {
    fn from(args: InferenceArgs) -> Self {
        let in_nodes: Vec<TFNode> = args.in_nodes.into_iter().map(|ref e| e.into()).collect();
        let out_nodes: Vec<TFNode> = args.out_nodes.into_iter().map(|ref e| e.into()).collect();
        let in_tensors: Vec<TFTensor> = args
            .in_tensors
            .into_iter()
            .map(|e| unsafe { e.as_ref().unwrap().into() })
            .collect();
        let run_options = unsafe {
            std::slice::from_raw_parts(
                (*args.run_options).data as *const u8,
                (*args.run_options).size,
            )
        }
        .to_vec();

        TensorflowModelRunRequest {
            in_nodes,
            out_nodes,
            in_tensors,
            run_options,
            ..Default::default()
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

    pub fn get_output<T: TensorType>(&self, id: usize) -> Result<Tensor<T>> {
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

        Ok(unsafe { Tensor::from_ffi(t)? })
    }

    pub fn get_grpc_output(&self, id: usize) -> Result<TFTensor> {
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
                status: result.status.into(),
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
