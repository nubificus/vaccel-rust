use crate::ffi;
use crate::session::Session;
use crate::tensorflow as tf;
use crate::tensorflow::saved_model::SavedModel;

use std::ffi::c_void;
use std::mem::size_of;
use std::os::raw::c_char;
use std::slice;

use crate::{Error, Result};

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
            VACCEL_OK => Ok(status),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Run inference on a TensorFlow model
    ///
    /// This will run inference using a TensorFlow graph that has been previously loaded
    /// using `vaccel_tf_model::load_graph`.
    ///
    pub fn inference(
        &self,
        sess: &mut Session,
        run_options: &tf::Buffer,
        in_nodes: &[tf::Node],
        in_tensors: &[tf::Tensor],
        out_nodes: &[tf::Node],
    ) -> Result<(Vec<tf::Tensor>, tf::Status)> {
        let mut status = tf::Status::new();
        let mut out_tensors: Vec<vaccel_tf_tensor> = vec![Default::default(); out_nodes.len()];

        match unsafe {
            ffi::vaccel_tf_model_run(
                sess.inner_mut(),
                self.inner_mut(),
                run_options,
                in_nodes.as_ptr(),
                in_tensors.as_ptr(),
                in_nodes.len() as i32,
                out_nodes.as_ptr(),
                out_tensors.as_mut_ptr(),
                out_nodes.len() as i32,
                status.inner_mut(),
            ) as u32
        } {
            VACCEL_OK => Ok((out_tensors, status)),
            err => Err(Error::Runtime(err)),
        }
    }
}

impl vaccel_tf_node {
    pub fn new(name: &str, id: i64) -> Self {
        vaccel_tf_node {
            name: name.as_ptr() as *mut c_char,
            id,
        }
    }
}

impl vaccel_tf_tensor {
    pub fn new<T>(data: &mut [T], dims: &mut [i64], data_type: vaccel_tf_data_type) -> Self {
        vaccel_tf_tensor {
            data: data.as_mut_ptr() as *mut c_void,
            size: (size_of::<T>() * data.len()) as u64,
            dims: dims.as_mut_ptr(),
            nr_dims: dims.len() as i32,
            data_type,
        }
    }

    pub fn as_slice<T>(&self) -> &[T] {
        let data = self.data as *const T;
        let data_count = self.size as usize / size_of::<T>();
        unsafe { slice::from_raw_parts(data, data_count) }
    }

    pub fn as_mut_slice<T>(&mut self) -> &mut [T] {
        let data = self.data as *mut T;
        let data_count = self.size as usize / size_of::<T>();
        unsafe { slice::from_raw_parts_mut(data, data_count) }
    }
}
