#[cfg(not(feature = "async"))]
use crate::sync::client::VsockClient;
#[cfg(feature = "async")]
use crate::asynchronous::client::VsockClient;
use super::{Error, Result};
use protocols::sync::agent_ttrpc::VaccelAgentClient;
use std::{collections::BTreeMap, env};
use vaccel::profiling::ProfRegions;

#[no_mangle]
pub extern "C" fn create_client() -> *mut VsockClient {
    match VsockClient::new() {
        Ok(c) => Box::into_raw(Box::new(c)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn destroy_client(client: *mut VsockClient) {
    if !client.is_null() {
        unsafe { Box::from_raw(client) };
    }
}
