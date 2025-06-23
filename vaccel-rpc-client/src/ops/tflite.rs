// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, IntoFfiResult, Result};
use log::error;
use std::ffi::c_int;
use vaccel::{
    c_pointer_to_mut_slice, c_pointer_to_slice, ffi, ops::tensorflow::lite::Status as TfliteStatus,
};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;
use vaccel_rpc_proto::{
    error::VaccelStatus,
    tensorflow::{
        TFLiteTensor, TensorflowLiteModelLoadRequest, TensorflowLiteModelRunRequest,
        TensorflowLiteModelUnloadRequest,
    },
};

impl VaccelRpcClient {
    pub fn tflite_model_load(&self, model_id: i64, session_id: i64) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = TensorflowLiteModelLoadRequest {
            session_id,
            model_id,
            ..Default::default()
        };

        self.execute(AgentServiceClient::tensorflow_lite_model_load, ctx, &req)?;

        Ok(())
    }

    pub fn tflite_model_unload(&self, model_id: i64, session_id: i64) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = TensorflowLiteModelUnloadRequest {
            session_id,
            model_id,
            ..Default::default()
        };

        self.execute(AgentServiceClient::tensorflow_lite_model_unload, ctx, &req)?;

        Ok(())
    }

    pub fn tflite_model_run(
        &self,
        model_id: i64,
        session_id: i64,
        in_tensors: Vec<TFLiteTensor>,
        nr_outputs: i32,
    ) -> Result<(Vec<*mut ffi::vaccel_tflite_tensor>, TfliteStatus)> {
        let ctx = ttrpc::context::Context::default();

        let req = TensorflowLiteModelRunRequest {
            model_id,
            session_id,
            in_tensors,
            nr_outputs,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::tensorflow_lite_model_run, ctx, &req)?;

        let out_tensors: Vec<*mut ffi::vaccel_tflite_tensor> = resp
            .out_tensors
            .into_iter()
            .map(|e| unsafe {
                let dims = e.dims;
                let data_type = e.type_.value();
                let data = e.data;

                let mut tensor = std::ptr::null_mut();
                match ffi::vaccel_tflite_tensor_allocate(
                    &mut tensor,
                    dims.len() as i32,
                    dims.as_ptr() as *mut _,
                    data_type as u32,
                    data.len(),
                ) as u32
                {
                    ffi::VACCEL_OK => (),
                    err => return Err(vaccel::Error::Ffi(err)),
                }
                assert!(!tensor.is_null());

                std::ptr::copy_nonoverlapping(data.as_ptr(), (*tensor).data as *mut u8, data.len());
                (*tensor).size = data.len();

                Ok(tensor)
            })
            .collect::<vaccel::Result<Vec<*mut ffi::vaccel_tflite_tensor>>>()?;
        let status = resp.status.unwrap_or(VaccelStatus::default());

        Ok((out_tensors, status.try_into()?))
    }
}

impl Error {
    fn to_tflite_status(&self) -> TfliteStatus {
        match self {
            Error::HostVaccel(ref e) => match e.get_status() {
                Some(status) => TfliteStatus::from(status),
                None => TfliteStatus(u8::MAX),
            },
            _ => TfliteStatus(u8::MAX),
        }
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tflite_model_load(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    client.tflite_model_load(model_id, sess_id).into_ffi()
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tflite_model_unload(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    client.tflite_model_unload(model_id, sess_id).into_ffi()
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `in_tensors_ptr` and `out_tensors_ptr` are expected to be valid pointers to
/// objects allocated manually or by the respective vAccel functions.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tflite_model_run(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
    in_tensors_ptr: *const *mut ffi::vaccel_tflite_tensor,
    nr_inputs: c_int,
    out_tensors_ptr: *mut *mut ffi::vaccel_tflite_tensor,
    nr_outputs: c_int,
    status_ptr: *mut u8,
) -> c_int {
    let in_tensors: Vec<TFLiteTensor> =
        match c_pointer_to_slice(in_tensors_ptr, nr_inputs.try_into().unwrap()) {
            Some(slice) => slice
                .iter()
                .map(|e| unsafe { e.as_ref().unwrap().into() })
                .collect(),
            None => return ffi::VACCEL_EINVAL as c_int,
        };

    let out_tensors = match c_pointer_to_mut_slice(out_tensors_ptr, nr_outputs.try_into().unwrap())
    {
        Some(vec) => vec,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let (ret, status) = client
        .tflite_model_run(model_id, sess_id, in_tensors, nr_outputs)
        .map_or_else(
            |e| {
                error!("{}", e);
                (e.to_ffi(), e.to_tflite_status())
            },
            |(tensors, status)| {
                out_tensors.copy_from_slice(&tensors);
                (ffi::VACCEL_OK, status)
            },
        );

    if let Err(e) = status.populate_ptr(status_ptr) {
        error!("Could not populate status: {}", e);
    }

    ret as c_int
}
