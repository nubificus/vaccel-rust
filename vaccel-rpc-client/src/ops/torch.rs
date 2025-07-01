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
    ops::torch::{DataType, DynTensor},
    Handle,
};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;
use vaccel_rpc_proto::torch::{TorchModelLoadRequest, TorchModelRunRequest, TorchTensor};

impl VaccelRpcClient {
    pub fn torch_model_load(&self, session_id: i64, model_id: i64) -> Result<()> {
        let ctx = ttrpc::context::Context::default();

        let mut req = TorchModelLoadRequest::new();
        req.session_id = session_id;
        req.model_id = model_id;

        self.execute(AgentServiceClient::torch_model_load, ctx, &req)?;
        Ok(())
    }

    pub fn torch_model_run(
        &self,
        session_id: i64,
        model_id: i64,
        run_options: Option<Vec<u8>>,
        in_tensors: Vec<TorchTensor>,
        nr_out_tensors: u64,
    ) -> Result<Vec<*mut ffi::vaccel_torch_tensor>> {
        let ctx = ttrpc::context::Context::default();

        let req = TorchModelRunRequest {
            session_id,
            model_id,
            run_options,
            in_tensors,
            nr_out_tensors,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::torch_model_run, ctx, &req)?;

        Ok(resp
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
            .collect::<vaccel::Result<Vec<*mut ffi::vaccel_torch_tensor>>>()?)
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_torch_model_load(
    client_ptr: *const VaccelRpcClient,
    sess_id: i64,
    model_id: ffi::vaccel_id_t,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    (match client.torch_model_load(sess_id, model_id) {
        Ok(()) => ffi::VACCEL_OK,
        Err(e) => {
            error!("{}", e);
            e.to_ffi()
        }
    }) as c_int
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `run_options_ptr`, `in_tensors_ptr` and `out_tensors_ptr` are expected to be
/// valid pointers to objects allocated manually or by the respective vAccel
/// functions.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_torch_model_run(
    client_ptr: *const VaccelRpcClient,
    sess_id: i64,
    model_id: ffi::vaccel_id_t,
    run_options_ptr: *const ffi::vaccel_torch_buffer,
    in_tensors_ptr: *const *mut ffi::vaccel_torch_tensor,
    nr_inputs: usize,
    out_tensors_ptr: *mut *mut ffi::vaccel_torch_tensor,
    nr_out_tensors: usize,
) -> c_int {
    let run_options = unsafe {
        run_options_ptr.as_ref().map(|opts| {
            c_pointer_to_slice(opts.data as *mut u8, opts.size)
                .unwrap_or(&[])
                .to_owned()
        })
    };

    let in_tensors = match c_pointer_to_slice(in_tensors_ptr, nr_inputs) {
        Some(slice) => slice,
        None => return ffi::VACCEL_EINVAL as c_int,
    };
    let proto_in_tensors: Vec<TorchTensor> = match in_tensors
        .iter()
        .map(|ptr| Ok(DynTensor::from_ptr(*ptr as *mut _)?.into()))
        .collect::<Result<Vec<TorchTensor>>>()
    {
        Ok(f) => f,
        Err(e) => {
            error!("{}", e);
            return e.to_ffi() as c_int;
        }
    };

    let out_tensors = match c_pointer_to_mut_slice(out_tensors_ptr, nr_out_tensors) {
        Some(vec) => vec,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let proto_nr_out_tensors = match nr_out_tensors.try_into() {
        Ok(num) => num,
        Err(e) => {
            let error = Error::InvalidArgument(format!(
                "Could not convert `nr_out_tensors` to `u64` [{}]",
                e
            ));
            error!("{}", error);
            return error.to_ffi() as c_int;
        }
    };

    (match client.torch_model_run(
        sess_id,
        model_id,
        run_options,
        proto_in_tensors,
        proto_nr_out_tensors,
    ) {
        Ok(results) => {
            out_tensors.copy_from_slice(&results);
            ffi::VACCEL_OK
        }
        Err(e) => {
            error!("{}", e);
            e.to_ffi()
        }
    }) as c_int
}
