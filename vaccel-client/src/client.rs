// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VsockClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VsockClient;
use env_logger::Env;

#[no_mangle]
pub extern "C" fn create_client() -> *mut VsockClient {
    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("info")).try_init();

    match VsockClient::new() {
        Ok(c) => Box::into_raw(Box::new(c)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn destroy_client(client: *mut VsockClient) {
    if !client.is_null() {
        unsafe { drop(Box::from_raw(client)) };
    }
}
