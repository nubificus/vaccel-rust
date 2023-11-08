use super::client::VsockClient;
use crate::{Error, Result};
use protocols::session::{CreateSessionRequest, DestroySessionRequest, UpdateSessionRequest};
use std::collections::btree_map::Entry;
use vaccel::ffi;

impl VsockClient {
    pub fn sess_init(&self, flags: u32) -> Result<u32> {
        let ctx = ttrpc::context::Context::default();
        let mut req = CreateSessionRequest::default();
        req.flags = flags;

        let resp = self.ttrpc_client.create_session(ctx, &req)?;

        Ok(resp.session_id)
    }

    pub fn sess_update(&self, sess_id: u32, flags: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = UpdateSessionRequest::default();
        req.session_id = sess_id;
        req.flags = flags;

        self.ttrpc_client.update_session(ctx, &req)?;
        Ok(())
    }

    pub fn sess_free(&self, sess_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = DestroySessionRequest::default();
        req.session_id = sess_id;

        self.ttrpc_client.destroy_session(ctx, &req)?;
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn sess_init(client_ptr: *mut VsockClient, flags: u32) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.sess_init(flags) {
        Ok(ret) => ret as i32,
        Err(_) => -(ffi::VACCEL_EIO as i32),
    }
}

#[no_mangle]
pub extern "C" fn sess_update(client_ptr: *const VsockClient, sess_id: u32, flags: u32) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.sess_update(sess_id, flags) {
        Ok(()) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub extern "C" fn sess_free(client_ptr: *mut VsockClient, sess_id: u32) -> i32 {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.sess_free(sess_id) {
        Ok(()) => {
            if let Entry::Occupied(t) = client.timers.entry(sess_id) {
                t.remove_entry();
            }
            ffi::VACCEL_OK as i32
        }
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub extern "C" fn register_resource(
    client_ptr: *const VsockClient,
    res_id: ffi::vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.register_resource(res_id, sess_id) {
        Ok(()) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub extern "C" fn unregister_resource(
    client_ptr: *const VsockClient,
    res_id: ffi::vaccel_id_t,
    sess_id: u32,
) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.unregister_resource(res_id, sess_id) {
        Ok(()) => ffi::VACCEL_OK as i32,
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}
