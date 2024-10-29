// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use dashmap::mapref::entry::Entry;
use log::error;
use std::ffi::c_int;
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
    pub fn session_init(&self, flags: u32) -> Result<i64> {
        let ctx = ttrpc::context::Context::default();
        let req = CreateSessionRequest {
            flags,
            ..Default::default()
        };

        let resp = self.execute(RpcAgentClient::create_session, ctx, &req)?;

        Ok(resp.session_id)
    }

    pub fn session_update(&self, sess_id: i64, flags: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = UpdateSessionRequest {
            session_id: sess_id,
            flags,
            ..Default::default()
        };

        let _resp = self.execute(RpcAgentClient::update_session, ctx, &req)?;

        Ok(())
    }

    pub fn session_release(&self, sess_id: i64) -> Result<()> {
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
pub unsafe extern "C" fn vaccel_rpc_client_session_init(
    client_ptr: *mut VaccelRpcClient,
    flags: u32,
) -> ffi::vaccel_id_t {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return -(ffi::VACCEL_EINVAL as ffi::vaccel_id_t),
    };

    match client.session_init(flags) {
        Ok(id) => id,
        Err(e) => {
            error!("{}", e);
            match e {
                Error::ClientError(_) => -(ffi::VACCEL_EBACKEND as ffi::vaccel_id_t),
                _ => -(ffi::VACCEL_EIO as ffi::vaccel_id_t),
            }
        }
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_session_update(
    client_ptr: *const VaccelRpcClient,
    sess_id: ffi::vaccel_id_t,
    flags: u32,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    match client.session_update(sess_id, flags) {
        Ok(()) => ffi::VACCEL_OK as c_int,
        Err(e) => {
            error!("{}", e);
            match e {
                Error::ClientError(_) => ffi::VACCEL_EBACKEND as c_int,
                _ => ffi::VACCEL_EIO as c_int,
            }
        }
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_session_release(
    client_ptr: *mut VaccelRpcClient,
    sess_id: ffi::vaccel_id_t,
) -> c_int {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    match client.session_release(sess_id) {
        Ok(()) => {
            //#[cfg(feature = "async")]
            //let mut timers = client.timers.lock().unwrap();
            //#[cfg(not(feature = "async"))]
            let timers = &mut client.timers;
            if let Entry::Occupied(t) = timers.entry(sess_id) {
                t.remove_entry();
            }
            ffi::VACCEL_OK as c_int
        }
        Err(e) => {
            error!("{}", e);
            match e {
                Error::ClientError(_) => ffi::VACCEL_EBACKEND as c_int,
                _ => ffi::VACCEL_EIO as c_int,
            }
        }
    }
}
