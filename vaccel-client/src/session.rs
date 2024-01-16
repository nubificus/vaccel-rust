#[cfg(feature = "async")]
use crate::asynchronous::client::VsockClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VsockClient;
use crate::{Error, Result};
use dashmap::mapref::entry::Entry;
#[cfg(feature = "async")]
use protocols::asynchronous::agent_ttrpc::VaccelAgentClient;
use protocols::session::{CreateSessionRequest, DestroySessionRequest, UpdateSessionRequest};
#[cfg(not(feature = "async"))]
use protocols::sync::agent_ttrpc::VaccelAgentClient;
use vaccel::ffi;
//use tracing::{info, instrument, Instrument};

impl VsockClient {
    pub fn sess_init(&self, flags: u32) -> Result<u32> {
        let ctx = ttrpc::context::Context::default();
        let req = CreateSessionRequest {
            flags,
            ..Default::default()
        };

        let resp = self.execute(VaccelAgentClient::create_session, ctx, &req)?;

        Ok(resp.session_id)
    }

    pub fn sess_update(&self, sess_id: u32, flags: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = UpdateSessionRequest {
            session_id: sess_id,
            flags,
            ..Default::default()
        };

        let _resp = self.execute(VaccelAgentClient::update_session, ctx, &req)?;

        Ok(())
    }

    pub fn sess_free(&self, sess_id: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = DestroySessionRequest {
            session_id: sess_id,
            ..Default::default()
        };

        let _resp = self.execute(VaccelAgentClient::destroy_session, ctx, &req)?;

        Ok(())
    }
}

#[no_mangle]
pub unsafe extern "C" fn sess_init(client_ptr: *mut VsockClient, flags: u32) -> i32 {
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
pub unsafe extern "C" fn sess_update(
    client_ptr: *const VsockClient,
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

#[no_mangle]
pub unsafe extern "C" fn sess_free(client_ptr: *mut VsockClient, sess_id: u32) -> i32 {
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

#[no_mangle]
pub unsafe extern "C" fn register_resource(
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
pub unsafe extern "C" fn unregister_resource(
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
