// SPDX-License-Identifier: Apache-2.0

use super::{Buffer, DataType, Node, Status, Tensor, TensorAny, TensorType};
use crate::{
    ffi,
    ops::{ModelInitialize, ModelLoadUnload, ModelRun},
    Error, Handle, Resource, Result, Session,
};
use log::warn;
use protobuf::Enum;
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
            self.run_options = opts.as_ptr();
        }
    }

    pub fn add_input(&mut self, node: &Node, tensor: &dyn TensorAny) -> Result<()> {
        self.in_nodes.push(unsafe { *node.as_ptr() });
        self.in_tensors.push(tensor.inner()?);
        Ok(())
    }

    pub fn request_output(&mut self, node: &Node) {
        self.out_nodes.push(unsafe { *node.as_ptr() });
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
    pub fn new(len: usize) -> Result<Self> {
        let out_tensors = vec![std::ptr::null_mut(); len];

        Ok(InferenceResult {
            out_tensors,
            status: Status::new(0, "")?,
        })
    }

    pub fn from_vec(tensors: Vec<*mut ffi::vaccel_tf_tensor>) -> Result<Self> {
        Ok(InferenceResult {
            out_tensors: tensors,
            status: Status::new(0, "")?,
        })
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

        let tensor: Tensor<T> = unsafe { Tensor::from_ptr(t)? };
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
    inner: &'a mut Resource,
}

impl<'a> ModelInitialize<'a> for Model<'a> {
    /// Creates a new 'Model` from a `Resource`.
    fn new(inner: &'a mut Resource) -> Self {
        Model { inner }
    }
}

impl<'a> ModelRun<'a> for Model<'a> {
    type RunArgs = InferenceArgs;
    type RunResult = InferenceResult;

    /// Runs inference using the model.
    ///
    /// This requires that the model has previously been loaded using `load()`.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    fn run(&mut self, sess: &mut Session, args: &mut Self::RunArgs) -> Result<Self::RunResult> {
        let mut result = InferenceResult::new(args.out_nodes.len())?;
        match unsafe {
            ffi::vaccel_tf_model_run(
                sess.as_mut_ptr(),
                self.inner.as_mut_ptr(),
                args.run_options,
                args.in_nodes.as_ptr(),
                args.in_tensors.as_ptr() as *const *mut ffi::vaccel_tf_tensor,
                args.in_nodes.len() as i32,
                args.out_nodes.as_ptr(),
                result.out_tensors.as_mut_ptr(),
                args.out_nodes.len() as i32,
                result.status.as_mut_ptr(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(result),
            err => Err(Error::FfiWithStatus {
                error: err,
                status: (&result.status).try_into()?,
            }),
        }
    }
}

impl<'a> ModelLoadUnload<'a> for Model<'a> {
    type LoadUnloadResult = Status;

    /// Loads the model.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    fn load(&mut self, sess: &mut Session) -> Result<Self::LoadUnloadResult> {
        let mut status = Status::new(0, "")?;
        match unsafe {
            ffi::vaccel_tf_model_load(
                sess.as_mut_ptr(),
                self.inner.as_mut_ptr(),
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

    /// Unloads the model.
    ///
    /// This will unload a model that was previously loaded in memory using
    /// `load()`.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    fn unload(&mut self, sess: &mut Session) -> Result<Self::LoadUnloadResult> {
        let mut status = Status::new(0, "")?;
        match unsafe {
            ffi::vaccel_tf_model_unload(
                sess.as_mut_ptr(),
                self.inner.as_mut_ptr(),
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
