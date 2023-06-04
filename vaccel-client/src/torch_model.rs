use crate::client::VsockClient;
use crate::resources::VaccelResource;
use crate::{Error, Result};

use vaccel::ffi;
use vaccel::torch::SavedModel;

use std::slice;

use protobuf::{ProtobufEnum, RepeatedField};

use protocols::{
    resources::{CreateResourceRequest, CreateTorchSavedModelRequest},
    torch::{TorchTensor, TorchJitloadForwardRequest},
};

impl VaccelResource for SavedModel {
    fn create_resource_request(self) -> Result<CreateResourceRequest> {
        let mut model = CreateTorchSavedModelRequest::new();
        model.set_model(
            self.get_protobuf()
                .ok_or(Error::InvalidArgument)?
                .to_owned(),
        );

        let mut req = CreateResourceRequest::new();
        req.set_torch_saved(model);

        Ok(req)
    }
}

impl VsockClient {
    pub fn torch_jitload_forward(
        &self,
        session_id: u32,
        model_id: i64,
        run_options: Vec<u8>,
        in_tensors: Vec<TorchTensor>,
    ) -> Result<Vec<*mut ffi::vaccel_torch_tensor>> {
        let ctx = ttrpc::context::Context::default();

        let req = TorchJitloadForwardRequest {
            session_id,
            model_id,
            run_options,
            in_tensors: RepeatedField::from_vec(in_tensors),
            ..Default::default()
        };

        let mut resp = self.ttrpc_client.torch_jitload_forward(ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        let torch_tensors = resp.take_result().take_out_tensors();
        Ok(torch_tensors
            .into_iter()
            .map(|mut e| unsafe {
                let dims = e.take_dims();
                let data_type = e.get_field_type().value();
                let data = e.take_data();
                let tensor = ffi::vaccel_torch_tensor_new(
                    dims.len() as i32,
                    dims.as_ptr() as *mut i64,
                    data_type as u32,
                );

                ffi::vaccel_torch_tensor_set_data(
                    tensor,
                    data.as_ptr() as *mut std::ffi::c_void,
                    data.len() as usize,
                );

                std::mem::forget(data);

                tensor
            })
            .collect())
    }
}

pub(crate) fn create_torch_model(
    client: &VsockClient,
    model_ptr: *mut ffi::vaccel_torch_saved_model,
) -> ffi::vaccel_id_t {
    let model = SavedModel::from_vaccel(model_ptr);
    match client.create_resource(model) {
        Ok(id) => id.into(),
        Err(Error::ClientError(err)) => -(err as ffi::vaccel_id_t),
        Err(_) => -(ffi::VACCEL_EIO as ffi::vaccel_id_t),
    }
}

fn c_pointer_to_vec<T>(buf: *mut T, len: usize, capacity: usize) -> Option<Vec<T>> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { Vec::from_raw_parts(buf, len, capacity) })
    }
}

fn c_pointer_to_slice<'a, T>(buf: *const T, len: usize) -> Option<&'a [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(buf, len) })
    }
}

fn c_pointer_to_mut_slice<'a, T>(buf: *mut T, len: usize) -> Option<&'a mut [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts_mut(buf, len) })
    }
}

#[no_mangle]
pub extern "C" fn torch_jitload_forward(
    client_ptr: *const VsockClient,
    sess_id: u32,
    model_id: ffi::vaccel_id_t,
    run_options_ptr: *mut ffi::vaccel_torch_buffer,
    in_tensors_ptr: *const *mut ffi::vaccel_torch_tensor,
    nr_inputs: usize,
    out_tensors_ptr: *mut *mut ffi::vaccel_torch_tensor,
    nr_outputs: usize,
) -> u32 {
    let run_options = unsafe {
        c_pointer_to_slice(
            (*run_options_ptr).data as *mut u8,
            (*run_options_ptr).size as usize,
        )
        .unwrap_or(&[])
        .to_owned()
    };

    let in_tensors: Vec<TorchTensor> = match c_pointer_to_slice(in_tensors_ptr, nr_inputs) {
        Some(slice) => slice
            .into_iter()
            .map(|e| unsafe { e.as_ref().unwrap().into() })
            .collect(),
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

    let ret = match client.torch_jitload_forward(
        sess_id,
        model_id,
        run_options,
        in_tensors,
    ) {
        Ok(results) => {
            out_tensors.copy_from_slice(&results);
            ffi::VACCEL_OK
        }
        Err(Error::ClientError(err)) => err,
        Err(_) => ffi::VACCEL_EINVAL,
    };

    ret
}
