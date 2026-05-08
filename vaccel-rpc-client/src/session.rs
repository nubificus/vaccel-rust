// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, IntoFfiResult, Result};
use log::error;
use std::ffi::{c_char, c_int, CStr};
use vaccel::{ffi, VaccelId};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;
use vaccel_rpc_proto::session::{CreateRequest, DestroyRequest, UpdateRequest};
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;
//use tracing::{info, instrument, Instrument};

impl VaccelRpcClient {
    pub fn session_init(&self, flags: u32) -> Result<i64> {
        let ctx = ttrpc::context::Context::default();
        let req = CreateRequest {
            flags,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::create_session, ctx, &req)?;

        Ok(VaccelId::try_from(resp.session_id)?.into())
    }

    pub fn session_init_with_plugin(&self, plugin_spec: &str) -> Result<i64> {
        let ctx = ttrpc::context::Context::default();
        let req = CreateRequest {
            plugin_spec: plugin_spec.to_string(),
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::create_session, ctx, &req)?;

        Ok(VaccelId::try_from(resp.session_id)?.into())
    }

    pub fn session_update(&self, sess_id: i64, flags: u32) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = UpdateRequest {
            session_id: sess_id,
            flags,
            ..Default::default()
        };

        self.execute(AgentServiceClient::update_session, ctx, &req)?;

        Ok(())
    }

    pub fn session_release(&self, sess_id: i64) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let req = DestroyRequest {
            session_id: sess_id,
            ..Default::default()
        };

        self.execute(AgentServiceClient::destroy_session, ctx, &req)?;

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

    client.session_init(flags).into_ffi()
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `plugin_spec` is expected to be a valid pointer to a NUL-terminated C
/// string.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_session_init_with_plugin(
    client_ptr: *mut VaccelRpcClient,
    plugin_spec_ptr: *const c_char,
) -> ffi::vaccel_id_t {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return -(ffi::VACCEL_EINVAL as ffi::vaccel_id_t),
    };

    if plugin_spec_ptr.is_null() {
        return -(ffi::VACCEL_EINVAL as ffi::vaccel_id_t);
    }

    let c_pspec = match CStr::from_ptr(plugin_spec_ptr).to_str().map_err(|e| {
        Error::InvalidArgument(format!(
            "Could not convert `plugin_spec` to `String` [{}]",
            e
        ))
    }) {
        Ok(p) => p,
        Err(e) => {
            error!("{}", e);
            return -(e.to_ffi() as ffi::vaccel_id_t);
        }
    };

    client.session_init_with_plugin(c_pspec).into_ffi()
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

    let sess_vaccel_id = match VaccelId::try_from(sess_id) {
        Ok(id) => id,
        Err(e) => {
            let err = Error::from(e);
            error!("{}", err);
            return err.to_ffi() as c_int;
        }
    };

    client
        .session_update(sess_vaccel_id.into(), flags)
        .into_ffi()
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

    let sess_vaccel_id = match VaccelId::try_from(sess_id) {
        Ok(id) => id,
        Err(e) => {
            let err = Error::from(e);
            error!("{}", err);
            return err.to_ffi() as c_int;
        }
    };

    (match client.session_release(sess_vaccel_id.into()) {
        Ok(()) => {
            client.profiler_manager.remove(sess_vaccel_id);
            ffi::VACCEL_OK
        }
        Err(e) => {
            error!("{}", e);
            e.to_ffi()
        }
    }) as c_int
}
