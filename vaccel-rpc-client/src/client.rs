// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use env_logger::Env;
use log::error;

#[no_mangle]
pub extern "C" fn vaccel_rpc_client_create() -> *mut VaccelRpcClient {
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("info")).try_init();

    match VaccelRpcClient::new() {
        Ok(c) => Box::into_raw(Box::new(c)),
        Err(e) => {
            error!("{}", e);
            std::ptr::null_mut()
        }
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_destroy(client: *mut VaccelRpcClient) {
    if !client.is_null() {
        unsafe { drop(Box::from_raw(client)) };
    }
}
