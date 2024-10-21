// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use dashmap::mapref::entry::Entry;
use vaccel::ffi;
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::RpcAgentClient;
use vaccel_rpc_proto::session::{
    CreateSessionRequest, DestroySessionRequest, UpdateSessionRequest,
};
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::RpcAgentClient;
//use tracing::{info, instrument, Instrument};

impl VaccelRpcClient {
    pub fn sess_init(&self, flags: u32) -> Result<u32> {
        let ctx = ttrpc::context::Context::default();
        let req = CreateSessionRequest {
            flags,
            ..Default::default()
        };

        let resp = self.execute(RpcAgentClient::create_session, ctx, &req)?;

        Ok(resp.session_id)
    }

    pub fn sess_update(&self, sess_id: u32, flags: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = UpdateSessionRequest {
            session_id: sess_id,
            flags,
            ..Default::default()
        };

        let _resp = self.execute(RpcAgentClient::update_session, ctx, &req)?;

        Ok(())
    }

    pub fn sess_free(&self, sess_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = DestroySessionRequest {
            session_id: sess_id,
            ..Default::default()
        };

        let _resp = self.execute(RpcAgentClient::destroy_session, ctx, &req)?;

        Ok(())
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn sess_init(client_ptr: *mut VaccelRpcClient, flags: u32) -> i32 {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.sess_init(flags) {
        Ok(ret) => ret as i32,
        Err(_) => -(ffi::VACCEL_EIO as i32),
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn sess_update(
    client_ptr: *const VaccelRpcClient,
    sess_id: u32,
    flags: u32,
) -> i32 {
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

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn sess_free(client_ptr: *mut VaccelRpcClient, sess_id: u32) -> i32 {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.sess_free(sess_id) {
        Ok(()) => {
            //#[cfg(feature = "async")]
            //let mut timers = client.timers.lock().unwrap();
            //#[cfg(not(feature = "async"))]
            let timers = &mut client.timers;
            if let Entry::Occupied(t) = timers.entry(sess_id) {
                t.remove_entry();
            }
            ffi::VACCEL_OK as i32
        }
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}
