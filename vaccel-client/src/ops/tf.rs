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
    TFNode, TFTensor, TensorflowModelLoadRequest, TensorflowModelRunRequest,
    TensorflowModelUnloadRequest,
};
use std::os::raw::c_int;
use vaccel::ffi;

impl VsockClient {
    pub fn tf_model_load(&self, model_id: i64, session_id: u32) -> Result<Vec<u8>> {
        let ctx = ttrpc::context::Context::default();
        let req = TensorflowModelLoadRequest {
            session_id,
            model_id,
            ..Default::default()
        };

        let mut resp = self.execute(VaccelAgentClient::tensorflow_model_load, ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        Ok(resp.take_graph_def())
    }

    pub fn tf_model_unload(&self, model_id: i64, session_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = TensorflowModelUnloadRequest {
            session_id,
            model_id,
            ..Default::default()
        };

        let mut resp = self.execute(VaccelAgentClient::tensorflow_model_unload, ctx, &req)?;

        match resp.error.take() {
            None => Ok(()),
            Some(e) => Err(e.into()),
        }
    }

    pub fn tf_model_run(
        &self,
        model_id: i64,
        session_id: u32,
        run_options: Vec<u8>,
        in_nodes: Vec<TFNode>,
        in_tensors: Vec<TFTensor>,
        out_nodes: Vec<TFNode>,
    ) -> Result<Vec<*mut ffi::vaccel_tf_tensor>> {
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

        let mut resp = self.execute(VaccelAgentClient::tensorflow_model_run, ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        let tf_tensors = resp.take_result().out_tensors;
        Ok(tf_tensors
            .into_iter()
            .map(|e| unsafe {
                let dims = e.dims;
                let data_type = e.type_.value();
                let data = e.data;
                let tensor = ffi::vaccel_tf_tensor_new(
                    dims.len() as i32,
                    dims.as_ptr() as *mut i64,
                    data_type as u32,
                );

                ffi::vaccel_tf_tensor_set_data(
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
pub unsafe extern "C" fn tf_session_load(
    client_ptr: *const VsockClient,
    model_id: ffi::vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.tf_model_load(model_id, sess_id) {
        Ok(_) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub unsafe extern "C" fn tf_session_delete(
    client_ptr: *const VsockClient,
    model_id: ffi::vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.tf_model_unload(model_id, sess_id) {
        Ok(_) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub unsafe extern "C" fn tf_session_run(
    client_ptr: *const VsockClient,
    model_id: ffi::vaccel_id_t,
    sess_id: u32,
    run_options_ptr: *mut ffi::vaccel_tf_buffer,
    in_nodes_ptr: *mut ffi::vaccel_tf_node,
    in_tensors_ptr: *const *mut ffi::vaccel_tf_tensor,
    nr_inputs: c_int,
    out_nodes_ptr: *mut ffi::vaccel_tf_node,
    out_tensors_ptr: *mut *mut ffi::vaccel_tf_tensor,
    nr_outputs: c_int,
) -> u32 {
    let run_options = unsafe {
        c_pointer_to_slice((*run_options_ptr).data as *mut u8, (*run_options_ptr).size)
            .unwrap_or(&[])
            .to_owned()
    };

    let in_nodes: Vec<TFNode> =
        match c_pointer_to_slice(in_nodes_ptr, nr_inputs.try_into().unwrap()) {
            Some(slice) => slice.iter().map(|e| e.into()).collect(),
            None => return ffi::VACCEL_EINVAL,
        };

    let in_tensors: Vec<TFTensor> =
        match c_pointer_to_slice(in_tensors_ptr, nr_inputs.try_into().unwrap()) {
            Some(slice) => slice
                .iter()
                .map(|e| unsafe { e.as_ref().unwrap().into() })
                .collect(),
            None => return ffi::VACCEL_EINVAL,
        };

    let out_nodes: Vec<TFNode> =
        match c_pointer_to_slice(out_nodes_ptr, nr_outputs.try_into().unwrap()) {
            Some(vec) => vec.iter().map(|e| e.into()).collect(),
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

    match client.tf_model_run(
        model_id,
        sess_id,
        run_options,
        in_nodes,
        in_tensors,
        out_nodes,
    ) {
        Ok(result) => {
            out_tensors.copy_from_slice(&result);
            ffi::VACCEL_OK
        }
        Err(Error::ClientError(err)) => err,
        Err(_) => ffi::VACCEL_EINVAL,
    }
}
