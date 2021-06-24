use crate::ffi;
use crate::session::Session;
use crate::tensorflow as tf;
use crate::tensorflow::saved_model::SavedModel;
use crate::{Error, Result};

use protobuf::{ProtobufEnum, RepeatedField};
use protocols::tensorflow::{TFDataType, TFNode, TFTensor, TensorflowModelRunRequest};

pub struct InferenceArgs {
    run_options: *const ffi::vaccel_tf_buffer,

    in_nodes: Vec<ffi::vaccel_tf_node>,
    in_tensors: Vec<*const ffi::vaccel_tf_tensor>,

    out_nodes: Vec<ffi::vaccel_tf_node>,
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

    pub fn add_input(&mut self, node: &tf::Node, tensor: &dyn tf::TensorAny) {
        self.in_nodes.push(unsafe { *node.inner() });
        self.in_tensors.push(tensor.inner());
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
                (*args.run_options).size as usize,
            )
        }
        .to_owned();

        TensorflowModelRunRequest {
            in_nodes: RepeatedField::from_vec(in_nodes),
            out_nodes: RepeatedField::from_vec(out_nodes),
            in_tensors: RepeatedField::from_vec(in_tensors),
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

        Ok(unsafe { tf::Tensor::from_vaccel_tensor(t).unwrap() })
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
                dims: std::slice::from_raw_parts((*t).dims as *mut u64, (*t).nr_dims as usize)
                    .to_owned(),
                field_type: TFDataType::from_i32((*t).data_type as i32).unwrap(),
                data: std::slice::from_raw_parts((*t).data as *mut u8, (*t).size as usize)
                    .to_owned(),
                ..Default::default()
            })
        }
    }
}

impl SavedModel {
    /// Load a TensorFlow graph from a model
    ///
    /// The TensorFlow model must have been created and registered to
    /// a session. The operation will load the graph and keep the graph
    /// TensorFlow representation in the model struct
    ///
    /// # Arguments
    ///
    /// * `session` - The session in the context of which we perform the operation. The model needs
    /// to be registered with this session.
    ///
    pub fn load_graph(&mut self, sess: &mut Session) -> Result<tf::Status> {
        let mut status = tf::Status::new();

        match unsafe {
            ffi::vaccel_tf_model_load_graph(sess.inner_mut(), self.inner_mut(), status.inner_mut())
                as u32
        } {
            ffi::VACCEL_OK => Ok(status),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Run inference on a TensorFlow model
    ///
    /// This will run inference using a TensorFlow graph that has been previously loaded
    /// using `vaccel_tf_model::load_graph`.
    ///
    pub fn inference(
        &mut self,
        sess: &mut Session,
        args: &mut InferenceArgs,
    ) -> Result<InferenceResult> {
        let mut result = InferenceResult::new(args.out_nodes.len());

        match unsafe {
            ffi::vaccel_tf_model_run(
                sess.inner_mut(),
                self.inner_mut(),
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
}
