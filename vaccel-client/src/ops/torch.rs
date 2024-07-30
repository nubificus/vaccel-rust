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
use protocols::torch::{TorchJitloadForwardRequest, TorchTensor};
use std::os::raw::c_int;
use vaccel::ffi;

impl VsockClient {
    pub fn torch_jitload_forward(
        &self,
        session_id: u32,
        model_id: i64,
        run_options: Vec<u8>,
        in_tensors: Vec<TorchTensor>,
        nr_outputs: i32,
    ) -> Result<Vec<*mut ffi::vaccel_torch_tensor>> {
        let ctx = ttrpc::context::Context::default();

        let req = TorchJitloadForwardRequest {
            session_id,
            model_id,
            run_options,
            in_tensors,
            nr_outputs,
            ..Default::default()
        };

        let mut resp = self.execute(VaccelAgentClient::torch_jitload_forward, ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        let torch_tensors = resp.take_result().out_tensors;
        Ok(torch_tensors
            .into_iter()
            .map(|e| unsafe {
                let dims = e.dims;
                let data_type = e.type_.value();
                let data = e.data;
                let tensor = ffi::vaccel_torch_tensor_new(
                    dims.len() as i32,
                    dims.as_ptr() as *mut i64,
                    data_type as u32,
                );

                ffi::vaccel_torch_tensor_set_data(
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
pub unsafe extern "C" fn torch_jitload_forward(
    client_ptr: *const VsockClient,
    sess_id: u32,
    model_id: ffi::vaccel_id_t,
    run_options_ptr: *mut ffi::vaccel_torch_buffer,
    in_tensors_ptr: *const *mut ffi::vaccel_torch_tensor,
    nr_inputs: c_int,
    out_tensors_ptr: *mut *mut ffi::vaccel_torch_tensor,
    nr_outputs: c_int,
) -> u32 {
    let run_options = unsafe {
        c_pointer_to_slice((*run_options_ptr).data as *mut u8, (*run_options_ptr).size)
            .unwrap_or(&[])
            .to_owned()
    };

    let in_tensors: Vec<TorchTensor> =
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

    match client.torch_jitload_forward(sess_id, model_id, run_options, in_tensors, nr_outputs) {
        Ok(results) => {
            out_tensors.copy_from_slice(&results);
            ffi::VACCEL_OK
        }
        Err(Error::ClientError(err)) => err,
        Err(_) => ffi::VACCEL_EINVAL,
    }
}
