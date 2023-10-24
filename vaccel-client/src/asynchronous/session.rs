use crate::{Error, Result};
use super::client::VsockClient;
use protocols::session::{CreateSessionRequest, DestroySessionRequest};
use std::collections::btree_map::Entry;
use vaccel::ffi;
//use tracing::{info, instrument, Instrument};

impl VsockClient {
    pub fn sess_init(&self, flags: u32) -> Result<u32> {
        let ctx = ttrpc::context::Context::default();
        let mut req = CreateSessionRequest::default();
        req.flags = flags;

        let tc = self.ttrpc_client.clone();
        let task = async {
            tokio::spawn(async move {
                tc.create_session(ctx, &req).await
            }).await
        };

        let resp = self.runtime.block_on(task)?;

        Ok(resp?.session_id)
    }

    pub fn sess_free(&self, sess_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = DestroySessionRequest::default();
        req.session_id = sess_id;

        let tc = self.ttrpc_client.clone();
        let task = async {
            tokio::spawn(async move {
                tc.destroy_session(ctx, &req).await
            }).await
        };

        let _resp = self.runtime.block_on(task)?;
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
pub extern "C" fn sess_free(client_ptr: *mut VsockClient, sess_id: u32) -> i32 {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.sess_free(sess_id) {
        Ok(()) => {
            let mut lock = client.timers.lock().unwrap();
            //let timers = lock.entry(sess_id);
            if let Entry::Occupied(t) = lock.entry(sess_id) {
                t.remove_entry();
            }
            ffi::VACCEL_OK as i32
        }
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}

#[no_mangle]
pub async extern "C" fn register_resource(
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
pub async extern "C" fn unregister_resource(
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
