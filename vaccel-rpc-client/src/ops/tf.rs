// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use log::error;
use std::ffi::c_int;
use vaccel::{
    c_pointer_to_mut_slice, c_pointer_to_slice, ffi,
    ops::tf::{DataType, DynTensor, Node, Status as TfStatus},
    Handle,
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

        let out_tensors = resp
            .out_tensors
            .into_iter()
            .map(|e| {
                let tensor = DynTensor::new_unchecked(
                    &e.dims,
                    DataType::from(e.type_.value() as u32),
                    e.data.len(),
                )?
                .with_data(&e.data)?;
                tensor.into_ptr()
            })
            .collect::<vaccel::Result<Vec<*mut ffi::vaccel_tf_tensor>>>()?;
        let status = resp.status.unwrap_or(VaccelStatus::default());

        Ok((out_tensors, status.try_into()?))
    }
}

impl Error {
    fn to_tf_status(&self) -> Result<TfStatus> {
        match self {
            Error::HostVaccel(ref e) => match e.get_status() {
                Some(status) => Ok(TfStatus::try_from(status)?),
                None => Ok(TfStatus::new(u8::MAX, "Undefined error")?),
            },
            err => Ok(TfStatus::new(u8::MAX, &err.to_string())?),
        }
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tf_model_load(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
    status_ptr: *mut ffi::vaccel_tf_status,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let (ret, status) = client.tf_model_load(model_id, sess_id).map_or_else(
        |e| {
            error!("{}", e);
            (e.to_ffi(), e.to_tf_status())
        },
        |(_, status)| (ffi::VACCEL_OK, Ok(status)),
    );

    match status {
        Ok(s) => {
            if let Err(e) = s.populate_ptr(status_ptr) {
                error!("Could not populate status: {}", e);
            }
        }
        Err(e) => error!("Could not create status from error: {}", e),
    };

    ret as c_int
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tf_model_unload(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
    status_ptr: *mut ffi::vaccel_tf_status,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let (ret, status) = client.tf_model_unload(model_id, sess_id).map_or_else(
        |e| {
            error!("{}", e);
            (e.to_ffi(), e.to_tf_status())
        },
        |status| (ffi::VACCEL_OK, Ok(status)),
    );

    match status {
        Ok(s) => {
            if let Err(e) = s.populate_ptr(status_ptr) {
                error!("Could not populate status: {}", e);
            }
        }
        Err(e) => error!("Could not create status from error: {}", e),
    };

    ret as c_int
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `run_options_ptr`, `in_nodes_ptr`, `in_tensors_ptr`, `out_nodes_ptr` and
/// `out_tensors_ptr` are expected to be valid pointers to objects allocated
/// manually or by the respective vAccel functions.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_tf_model_run(
    client_ptr: *const VaccelRpcClient,
    model_id: ffi::vaccel_id_t,
    sess_id: i64,
    run_options_ptr: *const ffi::vaccel_tf_buffer,
    in_nodes_ptr: *const ffi::vaccel_tf_node,
    in_tensors_ptr: *const *mut ffi::vaccel_tf_tensor,
    nr_inputs: usize,
    out_nodes_ptr: *const ffi::vaccel_tf_node,
    out_tensors_ptr: *mut *mut ffi::vaccel_tf_tensor,
    nr_outputs: usize,
    status_ptr: *mut ffi::vaccel_tf_status,
) -> c_int {
    let run_options = unsafe {
        run_options_ptr.as_ref().map(|opts| {
            c_pointer_to_slice(opts.data as *mut u8, opts.size)
                .unwrap_or(&[])
                .to_owned()
        })
    };

    let in_nodes = match c_pointer_to_slice(in_nodes_ptr, nr_inputs) {
        Some(slice) => slice,
        None => return ffi::VACCEL_EINVAL as c_int,
    };
    let proto_in_nodes = match in_nodes
        .iter()
        .map(|ptr| Ok(Node::from_ref(ptr)?.try_into()?))
        .collect::<Result<Vec<TFNode>>>()
    {
        Ok(f) => f,
        Err(e) => {
            error!("{}", e);
            return e.to_ffi() as c_int;
        }
    };

    let in_tensors = match c_pointer_to_slice(in_tensors_ptr, nr_inputs) {
        Some(slice) => slice,
        None => return ffi::VACCEL_EINVAL as c_int,
    };
    let proto_in_tensors: Vec<TFTensor> = match in_tensors
        .iter()
        .map(|ptr| Ok(DynTensor::from_ptr(*ptr as *mut _)?.into()))
        .collect::<Result<Vec<TFTensor>>>()
    {
        Ok(f) => f,
        Err(e) => {
            error!("{}", e);
            return e.to_ffi() as c_int;
        }
    };

    let out_nodes = match c_pointer_to_slice(out_nodes_ptr, nr_outputs) {
        Some(slice) => slice,
        None => return ffi::VACCEL_EINVAL as c_int,
    };
    let proto_out_nodes = match out_nodes
        .iter()
        .map(|ptr| Ok(Node::from_ref(ptr)?.try_into()?))
        .collect::<Result<Vec<TFNode>>>()
    {
        Ok(f) => f,
        Err(e) => {
            error!("{}", e);
            return e.to_ffi() as c_int;
        }
    };

    let out_tensors = match c_pointer_to_mut_slice(out_tensors_ptr, nr_outputs) {
        Some(vec) => vec,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let (ret, status) = client
        .tf_model_run(
            model_id,
            sess_id,
            run_options,
            proto_in_nodes,
            proto_in_tensors,
            proto_out_nodes,
        )
        .map_or_else(
            |e| {
                error!("{}", e);
                (e.to_ffi(), e.to_tf_status())
            },
            |(tensors, status)| {
                out_tensors.copy_from_slice(&tensors);
                (ffi::VACCEL_OK, Ok(status))
            },
        );

    match status {
        Ok(s) => {
            if let Err(e) = s.populate_ptr(status_ptr) {
                error!("Could not populate status: {}", e);
            }
        }
        Err(e) => error!("Could not create status from error: {}", e),
    };

    ret as c_int
}
