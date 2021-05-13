use crate::client::VsockClient;
use crate::tf_model::create_tf_model;
use protocols::resources::{
    CreateResourceRequest, DestroyResourceRequest, RegisterResourceRequest,
    UnregisterResourceRequest,
};

use vaccel_bindings::{vaccel_id_t, vaccel_resource_t, vaccel_tf_model};
use vaccel_bindings::{VACCEL_EINVAL, VACCEL_EIO, VACCEL_OK};

use std::ffi::c_void;

pub trait VaccelResource {
    fn create_resource_request(self) -> Result<CreateResourceRequest, u32>;
}

impl VsockClient {
    pub fn create_resource(&self, resource: impl VaccelResource) -> Result<vaccel_id_t, u32> {
        let ctx = ttrpc::context::Context::default();
        let req = resource.create_resource_request()?;

        match self.ttrpc_client.create_resource(ctx, &req) {
            Err(_) => Err(VACCEL_EIO),
            Ok(resp) => Ok(resp.get_resource_id()),
        }
    }

    pub fn destroy_resource(&self, model_id: i64) -> Result<(), u32> {
        let ctx = ttrpc::context::Context::default();
        let mut req = DestroyResourceRequest::new();
        req.set_resource_id(model_id);

        self.ttrpc_client
            .destroy_resource(ctx, &req)
            .map_err(|_| VACCEL_EIO)?;

        Ok(())
    }

    pub fn register_resource(&self, model_id: i64, sess_id: u32) -> Result<(), u32> {
        let ctx = ttrpc::context::Context::default();
        let mut req = RegisterResourceRequest::new();
        req.set_resource_id(model_id);
        req.set_session_id(sess_id);

        self.ttrpc_client
            .register_resource(ctx, &req)
            .map_err(|_| VACCEL_EIO)?;

        Ok(())
    }

    pub fn unregister_resource(&self, model_id: i64, sess_id: u32) -> Result<(), u32> {
        let ctx = ttrpc::context::Context::default();
        let mut req = UnregisterResourceRequest::new();
        req.set_resource_id(model_id);
        req.set_session_id(sess_id);

        self.ttrpc_client
            .unregister_resource(ctx, &req)
            .map_err(|_| VACCEL_EIO)?;

        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn create_resource(
    client_ptr: *const VsockClient,
    res_type: vaccel_resource_t,
    data: *mut c_void,
) -> vaccel_id_t {
    if data.is_null() {
        return VACCEL_EINVAL as i64;
    }

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return VACCEL_EINVAL as i64,
    };

    match res_type {
        _vaccel_resource_t_VACCEL_RES_TF_MODEL => {
            let model_ptr = data as *mut vaccel_tf_model;
            let model = unsafe { model_ptr.as_ref().unwrap() };
            create_tf_model(client, model)
        }
    }
}

#[no_mangle]
pub extern "C" fn destroy_resource(client_ptr: *const VsockClient, id: vaccel_id_t) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return VACCEL_EINVAL as i32,
    };

    match client.destroy_resource(id) {
        Err(ret) => ret as i32,
        Ok(()) => VACCEL_OK as i32,
    }
}
