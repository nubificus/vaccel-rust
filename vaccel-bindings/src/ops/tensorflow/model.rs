// SPDX-License-Identifier: Apache-2.0

use crate::{
    ffi,
    ops::{tensorflow as tf, ModelInitialize, ModelLoadUnload, ModelRun},
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

    pub fn set_run_options(&mut self, run_opts: &tf::Buffer) {
        self.run_options = run_opts.inner();
    }

    pub fn add_input(&mut self, node: &tf::Node, tensor: &dyn tf::TensorAny) -> Result<()> {
        self.in_nodes.push(unsafe { *node.inner() });
        self.in_tensors.push(tensor.inner()?);
        Ok(())
    }

    pub fn request_output(&mut self, node: &tf::Node) {
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
    status: tf::Status,
}

impl InferenceResult {
    pub fn new(len: usize) -> Self {
        let out_tensors = vec![std::ptr::null_mut(); len];

        InferenceResult {
            out_tensors,
            status: tf::Status::new(),
        }
    }

    pub fn from_vec(tensors: Vec<*mut ffi::vaccel_tf_tensor>) -> Self {
        InferenceResult {
            out_tensors: tensors,
            status: tf::Status::new(),
        }
    }

    pub fn get_output<T: tf::TensorType>(&self, id: usize) -> Result<tf::Tensor<T>> {
        if id >= self.out_tensors.len() {
            return Err(Error::TensorFlow(tf::Code::OutOfRange));
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::TensorFlow(tf::Code::Unavailable));
        }

        let inner_data_type = unsafe { tf::DataType::from_int((*t).data_type) };
        if inner_data_type != T::data_type() {
            return Err(Error::TensorFlow(tf::Code::InvalidArgument));
        }

        Ok(unsafe { tf::Tensor::from_ffi(t).unwrap() })
    }

    pub fn get_grpc_output(&self, id: usize) -> Result<TFTensor> {
        if id >= self.out_tensors.len() {
            return Err(Error::TensorFlow(tf::Code::OutOfRange));
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::TensorFlow(tf::Code::Unavailable));
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
        args: &mut InferenceArgs,
    ) -> Result<InferenceResult> {
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
            err => Err(Error::Runtime(err)),
        }
    }

    fn inner_mut(self: Pin<&mut Self>) -> Pin<&mut Resource> {
        unsafe { self.get_unchecked_mut().inner.as_mut() }
    }
}

impl<'a> ModelLoadUnload<'a> for Model<'a> {
    type LoadResult = tf::Status;

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
    fn load(self: Pin<&mut Self>, sess: &mut Session) -> Result<tf::Status> {
        let mut status = tf::Status::new();
        match unsafe {
            ffi::vaccel_tf_session_load(
                sess.inner_mut(),
                self.inner_mut().inner_mut(),
                status.inner_mut(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(status),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Delete a TensorFlow session
    ///
    /// This will unload a TensorFlow session that was previously loaded in memory
    /// using `load()`.
    fn unload(self: Pin<&mut Self>, sess: &mut Session) -> Result<()> {
        let mut status = tf::Status::new();
        match unsafe {
            ffi::vaccel_tf_session_delete(
                sess.inner_mut(),
                self.inner_mut().inner_mut(),
                status.inner_mut(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}
