// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use log::error;
use std::ffi::c_int;
use vaccel::{c_pointer_to_mut_slice, c_pointer_to_slice, ffi};
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
        nr_outputs: i32,
    ) -> Result<Vec<*mut ffi::vaccel_torch_tensor>> {
        let ctx = ttrpc::context::Context::default();

        let req = TorchModelRunRequest {
            session_id,
            model_id,
            run_options,
            in_tensors,
            nr_outputs,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::torch_model_run, ctx, &req)?;

        resp.out_tensors
            .into_iter()
            .map(|e| unsafe {
                let dims = e.dims;
                let data_type = e.type_.value();
                let data = e.data;

                let mut tensor = std::ptr::null_mut();
                match ffi::vaccel_torch_tensor_allocate(
                    &mut tensor,
                    dims.len() as i64,
                    dims.as_ptr() as *mut i64,
                    data_type as u32,
                    data.len(),
                ) as u32
                {
                    ffi::VACCEL_OK => (),
                    err => return Err(vaccel::Error::Ffi(err).into()),
                }
                assert!(!tensor.is_null());

                std::ptr::copy_nonoverlapping(data.as_ptr(), (*tensor).data as *mut u8, data.len());
                (*tensor).size = data.len();

                Ok(tensor)
            })
            .collect()
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
    nr_inputs: c_int,
    out_tensors_ptr: *mut *mut ffi::vaccel_torch_tensor,
    nr_outputs: c_int,
) -> c_int {
    let run_options = unsafe {
        run_options_ptr.as_ref().map(|opts| {
            c_pointer_to_slice(opts.data as *mut u8, opts.size)
                .unwrap_or(&[])
                .to_owned()
        })
    };

    let in_tensors: Vec<TorchTensor> =
        match c_pointer_to_slice(in_tensors_ptr, nr_inputs.try_into().unwrap()) {
            Some(slice) => {
                let res = slice
                    .iter()
                    .map(|e| unsafe {
                        e.as_ref()
                            .ok_or(Error::InvalidArgument("".to_string()))?
                            .try_into()
                            .map_err(|e: vaccel::Error| e.into())
                    })
                    .collect();
                match res {
                    Ok(s) => s,
                    Err(e) => {
                        return match e {
                            Error::InvalidArgument(_) => ffi::VACCEL_EINVAL,
                            _ => {
                                error!("{}", e);
                                ffi::VACCEL_EBACKEND
                            }
                        } as c_int
                    }
                }
            }
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

    (match client.torch_model_run(sess_id, model_id, run_options, in_tensors, nr_outputs) {
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
