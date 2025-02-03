// SPDX-License-Identifier: Apache-2.0

use super::{Buffer, Code, DataType, Tensor, TensorAny, TensorType};
use crate::{
    ffi,
    ops::{ModelInitialize, ModelRun},
    Error, Resource, Result, Session,
};
use protobuf::Enum;
use std::{marker::PhantomPinned, pin::Pin};
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

    pub fn set_run_options(&mut self, run_opts: &Buffer) {
        self.run_options = run_opts.inner();
    }

    // TODO: &TorchTensor -> TensorAny
    pub fn add_input(&mut self, tensor: &dyn TensorAny) {
        self.in_tensors.push(tensor.inner());
    }

    pub fn set_nr_outputs(&mut self, nr_outputs: i32) {
        self.nr_outputs = nr_outputs;
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

    pub fn get_output<T: TensorType>(&self, id: usize) -> Result<Tensor<T>> {
        if id >= self.out_tensors.len() {
            return Err(Error::Torch(Code::OutOfRange));
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::Torch(Code::Unavailable));
        }

        let inner_data_type = unsafe { DataType::from_int((*t).data_type) };
        if inner_data_type != T::data_type() {
            return Err(Error::Torch(Code::InvalidArgument));
        }

        Ok(unsafe { Tensor::from_ffi(t).unwrap() })
    }

    pub fn get_grpc_output(&self, id: usize) -> Result<TorchTensor> {
        if id >= self.out_tensors.len() {
            return Err(Error::Torch(Code::OutOfRange));
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::Torch(Code::Unavailable));
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

    fn run(
        self: Pin<&mut Self>,
        sess: &mut Session,
        args: &mut InferenceArgs,
    ) -> Result<InferenceResult> {
        let mut result = InferenceResult::new(args.in_tensors.len());
        match unsafe {
            ffi::vaccel_torch_jitload_forward(
                sess.inner_mut(),
                self.inner_mut().inner_mut(),
                args.run_options,
                args.in_tensors.as_ptr() as *mut *mut ffi::vaccel_torch_tensor,
                args.in_tensors.len() as i32,
                result.out_tensors.as_mut_ptr(),
                args.nr_outputs,
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
