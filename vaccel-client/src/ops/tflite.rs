// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VsockClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VsockClient;
use crate::{c_pointer_to_mut_slice, c_pointer_to_slice, Error, Result};
#[cfg(feature = "async")]
use protocols::asynchronous::agent_ttrpc::VaccelAgentClient;
#[cfg(not(feature = "async"))]
use protocols::sync::agent_ttrpc::VaccelAgentClient;
use protocols::tensorflow::{
    TFLiteTensor, TensorflowLiteModelLoadRequest, TensorflowLiteModelRunRequest,
    TensorflowLiteModelUnloadRequest,
};
use std::os::raw::c_int;
use vaccel::ffi;

impl VsockClient {
    pub fn tflite_model_load(&self, model_id: i64, session_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = TensorflowLiteModelLoadRequest {
            session_id,
            model_id,
            ..Default::default()
        };

        let mut resp = self.execute(VaccelAgentClient::tensorflow_lite_model_load, ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        Ok(())
    }

    pub fn tflite_model_unload(&self, model_id: i64, session_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = TensorflowLiteModelUnloadRequest {
            session_id,
            model_id,
            ..Default::default()
        };

        let mut resp = self.execute(VaccelAgentClient::tensorflow_lite_model_unload, ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        Ok(())
    }

    pub fn tflite_model_run(
        &self,
        model_id: i64,
        session_id: u32,
        in_tensors: Vec<TFLiteTensor>,
        nr_outputs: i32,
    ) -> Result<Vec<*mut ffi::vaccel_tflite_tensor>> {
        let ctx = ttrpc::context::Context::default();

        let req = TensorflowLiteModelRunRequest {
            model_id,
            session_id,
            in_tensors,
            nr_outputs,
            ..Default::default()
        };

        let mut resp = self.execute(VaccelAgentClient::tensorflow_lite_model_run, ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        let tflite_tensors = resp.take_result().out_tensors;
        Ok(tflite_tensors
            .into_iter()
            .map(|e| unsafe {
                let dims = e.dims;
                let data_type = e.type_.value();
                let data = e.data;
                let tensor = ffi::vaccel_tflite_tensor_new(
                    dims.len() as i32,
                    dims.as_ptr() as *mut _,
                    data_type as u32,
                );

                ffi::vaccel_tflite_tensor_set_data(
                    tensor,
                    data.as_ptr() as *mut std::ffi::c_void,
                    data.len(),
                );

                std::mem::forget(data);

                tensor
            })
            .collect())
    }
}

#[no_mangle]
pub unsafe extern "C" fn tflite_session_load(
    client_ptr: *const VsockClient,
    model_id: ffi::vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.tflite_model_load(model_id, sess_id) {
        Ok(_) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub unsafe extern "C" fn tflite_session_delete(
    client_ptr: *const VsockClient,
    model_id: ffi::vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.tflite_model_unload(model_id, sess_id) {
        Ok(_) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub unsafe extern "C" fn tflite_session_run(
    client_ptr: *const VsockClient,
    model_id: ffi::vaccel_id_t,
    sess_id: u32,
    in_tensors_ptr: *const *mut ffi::vaccel_tflite_tensor,
    nr_inputs: c_int,
    out_tensors_ptr: *mut *mut ffi::vaccel_tflite_tensor,
    nr_outputs: c_int,
) -> u32 {
    let in_tensors: Vec<TFLiteTensor> =
        match c_pointer_to_slice(in_tensors_ptr, nr_inputs.try_into().unwrap()) {
            Some(slice) => slice
                .iter()
                .map(|e| unsafe { e.as_ref().unwrap().into() })
                .collect(),
            None => return ffi::VACCEL_EINVAL,
        };

    let out_tensors = match c_pointer_to_mut_slice(out_tensors_ptr, nr_outputs.try_into().unwrap())
    {
        Some(vec) => vec,
        None => return ffi::VACCEL_EINVAL,
    };

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL,
    };

    match client.tflite_model_run(model_id, sess_id, in_tensors, nr_outputs) {
        Ok(result) => {
            out_tensors.copy_from_slice(&result);
            ffi::VACCEL_OK
        }
        Err(Error::ClientError(err)) => err,
        Err(_) => ffi::VACCEL_EINVAL,
    }
}
