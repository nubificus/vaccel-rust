use crate::client::VsockClient;
use crate::tf_model::create_tf_model;
use crate::{Error, Result};

use protocols::resources::{
    CreateResourceRequest, DestroyResourceRequest, RegisterResourceRequest,
    UnregisterResourceRequest,
};
use vaccel::ffi;
use vaccel::VaccelId;

use std::ffi::c_void;

pub trait VaccelResource {
    fn create_resource_request(self) -> Result<CreateResourceRequest>;
}

impl VsockClient {
    pub fn create_resource(&self, resource: impl VaccelResource) -> Result<VaccelId> {
        let ctx = ttrpc::context::Context::default();
        let req = resource.create_resource_request()?;

        let resp = self.ttrpc_client.create_resource(ctx, &req)?;

        Ok(resp.get_resource_id().into())
    }

    pub fn destroy_resource(&self, model_id: i64) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = DestroyResourceRequest::new();
        req.set_resource_id(model_id);

        self.ttrpc_client.destroy_resource(ctx, &req)?;

        Ok(())
    }

    pub fn register_resource(&self, model_id: i64, sess_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = RegisterResourceRequest::new();
        req.set_resource_id(model_id);
        req.set_session_id(sess_id);

        self.ttrpc_client.register_resource(ctx, &req)?;

        Ok(())
    }

    pub fn unregister_resource(&self, model_id: i64, sess_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = UnregisterResourceRequest::new();
        req.set_resource_id(model_id);
        req.set_session_id(sess_id);

        self.ttrpc_client.unregister_resource(ctx, &req)?;

        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn create_resource(
    client_ptr: *const VsockClient,
    res_type: ffi::vaccel_resource_t,
    data: *mut c_void,
) -> ffi::vaccel_id_t {
    if data.is_null() {
        return ffi::VACCEL_EINVAL as i64;
    }

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => {
            return ffi::VACCEL_EINVAL as i64;
        }
    };

    match res_type {
        _vaccel_resource_t_VACCEL_RES_TF_SAVED_MODEL => {
            let model_ptr = data as *mut ffi::vaccel_tf_saved_model;
            let model = unsafe { model_ptr.as_mut().unwrap() };
            create_tf_model(client, model)
        }
    }
}

#[no_mangle]
pub extern "C" fn destroy_resource(client_ptr: *const VsockClient, id: ffi::vaccel_id_t) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.destroy_resource(id) {
        Ok(()) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}
