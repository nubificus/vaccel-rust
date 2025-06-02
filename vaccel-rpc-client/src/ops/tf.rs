// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use log::error;
use std::ffi::c_int;
use vaccel::{
    c_pointer_to_mut_slice, c_pointer_to_slice, ffi, ops::tensorflow::Status as TfStatus,
};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;
use vaccel_rpc_proto::{
    error::VaccelStatus,
    tensorflow::{
        TFNode, TFTensor, TensorflowModelLoadRequest, TensorflowModelRunRequest,
        TensorflowModelUnloadRequest,
    },
};

impl VaccelRpcClient {
    pub fn tf_model_load(&self, model_id: i64, session_id: i64) -> Result<(Vec<u8>, TfStatus)> {
        let ctx = ttrpc::context::Context::default();
        let req = TensorflowModelLoadRequest {
            session_id,
            model_id,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::tensorflow_model_load, ctx, &req)?;

        let status = resp.status.unwrap_or(VaccelStatus::default());

        Ok((resp.graph_def, status.try_into()?))
    }

    pub fn tf_model_unload(&self, model_id: i64, session_id: i64) -> Result<TfStatus> {
        let ctx = ttrpc::context::Context::default();
        let req = TensorflowModelUnloadRequest {
            session_id,
            model_id,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::tensorflow_model_unload, ctx, &req)?;

        Ok(resp.status.unwrap_or(VaccelStatus::default()).try_into()?)
    }

    pub fn tf_model_run(
        &self,
        model_id: i64,
        session_id: i64,
        run_options: Option<Vec<u8>>,
        in_nodes: Vec<TFNode>,
        in_tensors: Vec<TFTensor>,
        out_nodes: Vec<TFNode>,
    ) -> Result<(Vec<*mut ffi::vaccel_tf_tensor>, TfStatus)> {
        let ctx = ttrpc::context::Context::default();

        let req = TensorflowModelRunRequest {
            model_id,
            session_id,
            run_options,
            in_nodes,
            in_tensors,
            out_nodes,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::tensorflow_model_run, ctx, &req)?;

        let out_tensors: Vec<*mut ffi::vaccel_tf_tensor> = resp
            .out_tensors
            .into_iter()
            .map(|e| unsafe {
                let dims = e.dims;
                let data_type = e.type_.value();
                let data = e.data;

                let mut tensor = std::ptr::null_mut();
                match ffi::vaccel_tf_tensor_allocate(
                    &mut tensor,
                    dims.len() as i32,
                    dims.as_ptr() as *mut i64,
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
            .collect::<vaccel::Result<Vec<*mut ffi::vaccel_tf_tensor>>>()?;
        let status = resp.status.unwrap_or(VaccelStatus::default());

        Ok((out_tensors, status.try_into()?))
    }
}

impl Error {
    fn to_tf_status(&self) -> TfStatus {
        match self {
            Error::HostVaccel(ref e) => match e.get_status() {
                Some(status) => TfStatus::try_from(status).unwrap_or_default(),
                None => TfStatus::new(u8::MAX, "Undefined error").unwrap_or_default(),
            },
            err => TfStatus::new(u8::MAX, &err.to_string()).unwrap_or_default(),
        }
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tf_session_load(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
    status_ptr: *mut ffi::vaccel_tf_status,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    (match client.tf_model_load(model_id, sess_id) {
        Ok((_, status)) => {
            status.populate_ffi(status_ptr);
            ffi::VACCEL_OK
        }
        Err(e) => {
            error!("{}", e);
            e.to_tf_status().populate_ffi(status_ptr);
            e.to_ffi()
        }
    }) as c_int
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tf_session_delete(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
    status_ptr: *mut ffi::vaccel_tf_status,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    (match client.tf_model_unload(model_id, sess_id) {
        Ok(status) => {
            status.populate_ffi(status_ptr);
            ffi::VACCEL_OK
        }
        Err(e) => {
            error!("{}", e);
            e.to_tf_status().populate_ffi(status_ptr);
            e.to_ffi()
        }
    }) as c_int
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `run_options_ptr`, `in_nodes_ptr`, `in_tensors_ptr`, `out_nodes_ptr` and
/// `out_tensors_ptr` are expected to be valid pointers to objects allocated
/// manually or by the respective vAccel functions.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tf_session_run(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
    run_options_ptr: *const ffi::vaccel_tf_buffer,
    in_nodes_ptr: *const ffi::vaccel_tf_node,
    in_tensors_ptr: *const *mut ffi::vaccel_tf_tensor,
    nr_inputs: c_int,
    out_nodes_ptr: *const ffi::vaccel_tf_node,
    out_tensors_ptr: *mut *mut ffi::vaccel_tf_tensor,
    nr_outputs: c_int,
    status_ptr: *mut ffi::vaccel_tf_status,
) -> c_int {
    let run_options = unsafe {
        run_options_ptr.as_ref().map(|opts| {
            c_pointer_to_slice(opts.data as *mut u8, opts.size)
                .unwrap_or(&[])
                .to_owned()
        })
    };

    let in_nodes: Vec<TFNode> =
        match c_pointer_to_slice(in_nodes_ptr, nr_inputs.try_into().unwrap()) {
            Some(slice) => slice.iter().map(|e| e.into()).collect(),
            None => return ffi::VACCEL_EINVAL as c_int,
        };

    let in_tensors: Vec<TFTensor> =
        match c_pointer_to_slice(in_tensors_ptr, nr_inputs.try_into().unwrap()) {
            Some(slice) => slice
                .iter()
                .map(|e| unsafe { e.as_ref().unwrap().into() })
                .collect(),
            None => return ffi::VACCEL_EINVAL as c_int,
        };

    let out_nodes: Vec<TFNode> =
        match c_pointer_to_slice(out_nodes_ptr, nr_outputs.try_into().unwrap()) {
            Some(vec) => vec.iter().map(|e| e.into()).collect(),
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

    (match client.tf_model_run(
        model_id,
        sess_id,
        run_options,
        in_nodes,
        in_tensors,
        out_nodes,
    ) {
        Ok((tensors, status)) => {
            out_tensors.copy_from_slice(&tensors);
            status.populate_ffi(status_ptr);
            ffi::VACCEL_OK
        }
        Err(e) => {
            error!("{}", e);
            e.to_tf_status().populate_ffi(status_ptr);
            e.to_ffi()
        }
    }) as c_int
}
