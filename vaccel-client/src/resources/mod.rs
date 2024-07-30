#[cfg(feature = "async")]
use crate::asynchronous::client::VsockClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VsockClient;
use crate::{Error, Result};
#[cfg(feature = "async")]
use protocols::asynchronous::agent_ttrpc::VaccelAgentClient;
use protocols::resources::{
    CreateResourceRequest, DestroyResourceRequest, RegisterResourceRequest,
    UnregisterResourceRequest,
};
#[cfg(not(feature = "async"))]
use protocols::sync::agent_ttrpc::VaccelAgentClient;
use std::ffi::c_void;
use vaccel::{ffi, VaccelId};

pub mod shared_obj;
pub mod single_model;
#[cfg(target_pointer_width = "64")]
pub mod tf_saved_model;

use shared_obj::create_shared_object;
use single_model::create_single_model;
#[cfg(target_pointer_width = "64")]
use tf_saved_model::create_tf_saved_model;

pub trait VaccelResource {
    fn create_resource_request(self) -> Result<CreateResourceRequest>;
}

impl VsockClient {
    pub fn create_resource(&self, resource: impl VaccelResource) -> Result<VaccelId> {
        let ctx = ttrpc::context::Context::default();
        let req = resource.create_resource_request()?;

        let resp = self.execute(VaccelAgentClient::create_resource, ctx, &req)?;

        Ok(resp.resource_id.into())
    }

    pub fn destroy_resource(&self, model_id: i64) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = DestroyResourceRequest::new();
        req.resource_id = model_id;

        let _resp = self.execute(VaccelAgentClient::destroy_resource, ctx, &req)?;

        Ok(())
    }

    pub fn register_resource(&self, model_id: i64, sess_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = RegisterResourceRequest::new();
        req.resource_id = model_id;
        req.session_id = sess_id;

        let _resp = self.execute(VaccelAgentClient::register_resource, ctx, &req)?;

        Ok(())
    }

    pub fn unregister_resource(&self, model_id: i64, sess_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = UnregisterResourceRequest::new();
        req.resource_id = model_id;
        req.session_id = sess_id;

        let _resp = self.execute(VaccelAgentClient::unregister_resource, ctx, &req)?;

        Ok(())
    }
}

#[no_mangle]
pub unsafe extern "C" fn create_resource(
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
        ffi::VACCEL_RES_SHARED_OBJ => {
            let shared_object = data as *mut ffi::vaccel_shared_object;
            let shared_obj = unsafe { shared_object.as_mut().unwrap() };
            create_shared_object(client, shared_obj)
        }
        ffi::VACCEL_RES_SINGLE_MODEL => {
            let model_ptr = data as *mut ffi::vaccel_single_model;
            let model = unsafe { model_ptr.as_mut().unwrap() };
            create_single_model(client, model)
        }
        ffi::VACCEL_RES_TF_SAVED_MODEL => {
            #[cfg(target_pointer_width = "64")]
            {
                let model_ptr = data as *mut ffi::vaccel_tf_saved_model;
                let model = unsafe { model_ptr.as_mut().unwrap() };
                create_tf_saved_model(client, model)
            }
            #[cfg(not(target_pointer_width = "64"))]
            {
                -(ffi::VACCEL_ENOTSUP as i64)
            }
        }
        3_u32..=u32::MAX => {
            todo!()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn destroy_resource(
    client_ptr: *const VsockClient,
    id: ffi::vaccel_id_t,
) -> i32 {
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