#[cfg(feature = "async")]
use crate::asynchronous::client::VsockClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VsockClient;
use crate::{c_pointer_to_mut_slice, c_pointer_to_slice, resources::VaccelResource, Error, Result};
#[cfg(feature = "async")]
use protocols::asynchronous::agent_ttrpc::VaccelAgentClient;
#[cfg(not(feature = "async"))]
use protocols::sync::agent_ttrpc::VaccelAgentClient;
use protocols::{
    resources::{CreateResourceRequest, CreateTensorflowSavedModelRequest},
    tensorflow::{
        TFNode, TFTensor, TensorflowModelLoadRequest, TensorflowModelRunRequest,
        TensorflowModelUnloadRequest,
    },
};
use vaccel::{ffi, tensorflow::SavedModel};

impl VaccelResource for SavedModel {
    fn create_resource_request(self) -> Result<CreateResourceRequest> {
        let mut model = CreateTensorflowSavedModelRequest::new();
        model.model_pb = self
            .get_protobuf()
            .ok_or(Error::InvalidArgument)?
            .to_owned();

        model.checkpoint = self
            .get_checkpoint()
            .ok_or(Error::InvalidArgument)?
            .to_owned();

        model.var_index = self
            .get_var_index()
            .ok_or(Error::InvalidArgument)?
            .to_owned();

        let mut req = CreateResourceRequest::new();
        req.set_tf_saved(model);

        Ok(req)
    }
}

impl VsockClient {
    pub fn tensorflow_load_graph(&self, model_id: i64, session_id: u32) -> Result<Vec<u8>> {
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

    pub fn tensorflow_session_delete(&self, model_id: i64, session_id: u32) -> Result<()> {
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

    pub fn tensorflow_inference(
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
                    dims.as_ptr() as *mut u32,
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

pub(crate) fn create_tf_model(
    client: &VsockClient,
    model_ptr: *mut ffi::vaccel_tf_saved_model,
) -> ffi::vaccel_id_t {
    let model = SavedModel::from_vaccel(model_ptr);
    match client.create_resource(model) {
        Ok(id) => id.into(),
        Err(Error::ClientError(err)) => -(err as ffi::vaccel_id_t),
        Err(_) => -(ffi::VACCEL_EIO as ffi::vaccel_id_t),
    }
}

#[no_mangle]
pub unsafe extern "C" fn tf_model_load(
    client_ptr: *const VsockClient,
    model_id: ffi::vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.tensorflow_load_graph(model_id, sess_id) {
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

    match client.tensorflow_session_delete(model_id, sess_id) {
        Ok(_) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub unsafe extern "C" fn tf_model_run(
    client_ptr: *const VsockClient,
    model_id: ffi::vaccel_id_t,
    sess_id: u32,
    run_options_ptr: *mut ffi::vaccel_tf_buffer,
    in_nodes_ptr: *mut ffi::vaccel_tf_node,
    in_tensors_ptr: *const *mut ffi::vaccel_tf_tensor,
    nr_inputs: usize,
    out_nodes_ptr: *mut ffi::vaccel_tf_node,
    out_tensors_ptr: *mut *mut ffi::vaccel_tf_tensor,
    nr_outputs: usize,
) -> u32 {
    let run_options = unsafe {
        c_pointer_to_slice((*run_options_ptr).data as *mut u8, (*run_options_ptr).size)
            .unwrap_or(&[])
            .to_owned()
    };

    let in_nodes: Vec<TFNode> = match c_pointer_to_slice(in_nodes_ptr, nr_inputs) {
        Some(slice) => slice.iter().map(|e| e.into()).collect(),
        None => return ffi::VACCEL_EINVAL,
    };

    let in_tensors: Vec<TFTensor> = match c_pointer_to_slice(in_tensors_ptr, nr_inputs) {
        Some(slice) => slice
            .iter()
            .map(|e| unsafe { e.as_ref().unwrap().into() })
            .collect(),
        None => return ffi::VACCEL_EINVAL,
    };

    let out_nodes: Vec<TFNode> = match c_pointer_to_slice(out_nodes_ptr, nr_outputs) {
        Some(vec) => vec.iter().map(|e| e.into()).collect(),
        None => return ffi::VACCEL_EINVAL,
    };

    let out_tensors = match c_pointer_to_mut_slice(out_tensors_ptr, nr_outputs) {
        Some(vec) => vec,
        None => return ffi::VACCEL_EINVAL,
    };

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL,
    };

    match client.tensorflow_inference(
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
