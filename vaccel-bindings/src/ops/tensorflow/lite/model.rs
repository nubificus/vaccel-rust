// SPDX-License-Identifier: Apache-2.0

use super::{DataType, Status, Tensor, TensorAny, TensorType};
use crate::{
    ffi,
    ops::{ModelInitialize, ModelLoadUnload, ModelRun},
    Error, Resource, Result, Session,
};
use log::warn;
use protobuf::Enum;
use std::{marker::PhantomPinned, pin::Pin};
use vaccel_rpc_proto::tensorflow::{TFLiteTensor, TFLiteType};

pub struct InferenceArgs {
    in_tensors: Vec<*const ffi::vaccel_tflite_tensor>,
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
            in_tensors: vec![],
            nr_outputs: 0,
        }
    }

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
            let ret = unsafe { ffi::vaccel_tflite_tensor_delete(tensor_ptr as *mut _) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete TFLite tensor: {}", ret);
            }
        }
    }
}

pub struct InferenceResult {
    out_tensors: Vec<*mut ffi::vaccel_tflite_tensor>,
    pub status: Status,
}

impl InferenceResult {
    pub fn new(len: usize) -> Self {
        let out_tensors = vec![std::ptr::null_mut(); len];

        InferenceResult {
            out_tensors,
            status: Status(0),
        }
    }

    pub fn from_vec(tensors: Vec<*mut ffi::vaccel_tflite_tensor>) -> Self {
        InferenceResult {
            out_tensors: tensors,
            status: Status(0),
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

    pub fn to_grpc_output(&self, id: usize) -> Result<TFLiteTensor> {
        if id >= self.out_tensors.len() {
            return Err(Error::OutOfBounds);
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::EmptyValue);
        }

        unsafe {
            Ok(TFLiteTensor {
                dims: std::slice::from_raw_parts((*t).dims, (*t).nr_dims as usize).to_vec(),
                type_: TFLiteType::from_i32((*t).data_type as i32).unwrap().into(),
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
            let ret = unsafe { ffi::vaccel_tflite_tensor_delete(tensor_ptr as *mut _) } as u32;
            if ret != ffi::VACCEL_OK {
                warn!("Could not delete TFLite tensor: {}", ret);
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

    fn run(
        self: Pin<&mut Self>,
        sess: &mut Session,
        args: &mut Self::RunArgs,
    ) -> Result<Self::RunResult> {
        let mut result = InferenceResult::new(args.in_tensors.len());
        match unsafe {
            ffi::vaccel_tflite_session_run(
                sess.inner_mut(),
                self.inner_mut().inner_mut(),
                args.in_tensors.as_ptr() as *const *mut ffi::vaccel_tflite_tensor,
                args.in_tensors.len() as i32,
                result.out_tensors.as_mut_ptr(),
                args.nr_outputs,
                &mut result.status.0 as *mut _,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(result),
            error => Err(Error::FfiWithStatus {
                error,
                status: result.status.into(),
            }),
        }
    }

    fn inner_mut(self: Pin<&mut Self>) -> Pin<&mut Resource> {
        unsafe { self.get_unchecked_mut().inner.as_mut() }
    }
}

impl<'a> ModelLoadUnload<'a> for Model<'a> {
    type LoadUnloadResult = ();

    fn load(self: Pin<&mut Self>, sess: &mut Session) -> Result<Self::LoadUnloadResult> {
        match unsafe {
            ffi::vaccel_tflite_session_load(sess.inner_mut(), self.inner_mut().inner_mut()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }

    fn unload(self: Pin<&mut Self>, sess: &mut Session) -> Result<Self::LoadUnloadResult> {
        match unsafe {
            ffi::vaccel_tflite_session_delete(sess.inner_mut(), self.inner_mut().inner_mut()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }
}
