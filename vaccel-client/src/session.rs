use crate::client::VsockClient;

use protocols::session::{CreateSessionRequest, DestroySessionRequest};
use vaccel_bindings::vaccel_id_t;
use vaccel_bindings::{VACCEL_EINVAL, VACCEL_EIO, VACCEL_OK};

impl VsockClient {
    pub fn sess_init(&self, flags: u32) -> Result<u32, u32> {
        let ctx = ttrpc::context::Context::default();
        let mut req = CreateSessionRequest::default();
        req.flags = flags;

        match self.ttrpc_client.create_session(ctx, &req) {
            Err(_) => Err(VACCEL_EIO),
            Ok(resp) => Ok(resp.session_id),
        }
    }

    pub fn sess_free(&self, sess_id: u32) -> Result<(), u32> {
        let ctx = ttrpc::context::Context::default();
        let mut req = DestroySessionRequest::default();
        req.session_id = sess_id;

        match self.ttrpc_client.destroy_session(ctx, &req) {
            Err(_) => Err(VACCEL_EIO),
            Ok(_) => Ok(()),
        }
    }
}

#[no_mangle]
pub extern "C" fn sess_init(client_ptr: *mut VsockClient, flags: u32) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return VACCEL_EINVAL as i32,
    };

    match client.sess_init(flags) {
        Ok(ret) => ret as i32,
        Err(err) => -(err as i32),
    }
}

#[no_mangle]
pub extern "C" fn sess_free(client_ptr: *const VsockClient, sess_id: u32) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return VACCEL_EINVAL as i32,
    };

    match client.sess_free(sess_id) {
        Ok(()) => VACCEL_OK as i32,
        Err(ret) => ret as i32,
    }
}

#[no_mangle]
pub extern "C" fn register_resource(
    client_ptr: *const VsockClient,
    res_id: vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return VACCEL_EINVAL as i32,
    };

    match client.register_resource(res_id, sess_id) {
        Err(ret) => ret as i32,
        Ok(()) => VACCEL_OK as i32,
    }
}

#[no_mangle]
pub extern "C" fn unregister_resource(
    client_ptr: *const VsockClient,
    res_id: vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return VACCEL_EINVAL as i32,
    };

    match client.unregister_resource(res_id, sess_id) {
        Err(ret) => ret as i32,
        Ok(()) => VACCEL_OK as i32,
    }
}
