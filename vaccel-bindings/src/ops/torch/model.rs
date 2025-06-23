// SPDX-License-Identifier: Apache-2.0

use super::{Buffer, DataType, Tensor, TensorAny, TensorType};
use crate::{
    ffi,
    ops::{ModelInitialize, ModelLoadUnload, ModelRun},
    Error, Handle, Resource, Result, Session,
};
use log::warn;
use protobuf::Enum;
use vaccel_rpc_proto::torch::{TorchDataType, TorchTensor};

pub struct InferenceArgs {
    run_options: *const ffi::vaccel_torch_buffer,
    in_tensors: Vec<*const ffi::vaccel_torch_tensor>,
    nr_outputs: i32,
}

impl Default for InferenceArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceArgs {
    pub fn new() -> Self {
        InferenceArgs {
            run_options: std::ptr::null::<ffi::vaccel_torch_buffer>(),
            in_tensors: vec![],
            nr_outputs: 0,
        }
    }

    pub fn set_run_options(&mut self, run_opts: Option<&Buffer>) {
        if let Some(opts) = run_opts {
            self.run_options = opts.as_ptr();
        }
    }

    // TODO: &TorchTensor -> TensorAny
    pub fn add_input(&mut self, tensor: &dyn TensorAny) -> Result<()> {
        self.in_tensors.push(tensor.inner()?);
        Ok(())
    }

    pub fn set_nr_outputs(&mut self, nr_outputs: i32) {
        self.nr_outputs = nr_outputs;
    }
}

impl Drop for InferenceArgs {
    fn drop(&mut self) {
        while let Some(tensor_ptr) = self.in_tensors.pop() {
            if tensor_ptr.is_null() {
                continue;
            }
            let ret = unsafe { ffi::vaccel_torch_tensor_delete(tensor_ptr as *mut _) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete Torch tensor: {}", ret);
            }
        }
    }
}

pub struct InferenceResult {
    out_tensors: Vec<*mut ffi::vaccel_torch_tensor>,
    // TODO: Do we need a torch::status here?
}

impl InferenceResult {
    pub fn new(len: usize) -> Self {
        let out_tensors = vec![std::ptr::null_mut(); len];

        InferenceResult { out_tensors }
    }

    pub fn from_vec(tensors: Vec<*mut ffi::vaccel_torch_tensor>) -> Self {
        InferenceResult {
            out_tensors: tensors,
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

        let tensor: Tensor<T> = unsafe { Tensor::from_ptr(t)? };
        self.out_tensors[id] = std::ptr::null_mut();

        Ok(tensor)
    }

    pub fn to_grpc_output(&self, id: usize) -> Result<TorchTensor> {
        if id >= self.out_tensors.len() {
            return Err(Error::OutOfBounds);
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::EmptyValue);
        }

        unsafe {
            Ok(TorchTensor {
                dims: std::slice::from_raw_parts((*t).dims as *mut _, (*t).nr_dims as usize)
                    .to_owned(),
                type_: TorchDataType::from_i32((*t).data_type as i32)
                    .unwrap()
                    .into(),
                data: std::slice::from_raw_parts((*t).data as *mut u8, (*t).size).to_owned(),
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
            let ret = unsafe { ffi::vaccel_torch_tensor_delete(tensor_ptr as *mut _) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete Torch tensor: {}", ret);
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

impl<'a> ModelLoadUnload<'a> for Model<'a> {
    type LoadUnloadResult = ();

    /// Loads the model.
    ///
    /// The inner `Resource` must be registered to the provided `Session`.
    fn load(&mut self, sess: &mut Session) -> Result<Self::LoadUnloadResult> {
        let result = ();
        match unsafe {
            ffi::vaccel_torch_model_load(sess.as_mut_ptr(), self.inner.as_mut_ptr()) as u32
        } {
            ffi::VACCEL_OK => Ok(result),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Unloads the model.
    ///
    /// Dummy implementation for trait compatibility.
    fn unload(&mut self, _sess: &mut Session) -> Result<Self::LoadUnloadResult> {
        Ok(())
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
    fn run(&mut self, sess: &mut Session, args: &mut InferenceArgs) -> Result<InferenceResult> {
        let mut result = InferenceResult::new(args.in_tensors.len());
        match unsafe {
            ffi::vaccel_torch_model_run(
                sess.as_mut_ptr(),
                self.inner.as_mut_ptr(),
                args.run_options,
                args.in_tensors.as_ptr() as *mut *mut ffi::vaccel_torch_tensor,
                args.in_tensors.len() as i32,
                result.out_tensors.as_mut_ptr(),
                args.nr_outputs,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(result),
            err => Err(Error::Ffi(err)),
        }
    }
}
