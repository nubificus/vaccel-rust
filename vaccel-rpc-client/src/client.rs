// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use env_logger::Env;
use log::error;
use std::{env, net::ToSocketAddrs};
#[cfg(feature = "async")]
use ttrpc::asynchronous::Client as TtrpcClient;
#[cfg(not(feature = "async"))]
use ttrpc::Client as TtrpcClient;

impl VaccelRpcClient {
    pub fn get_env_address() -> String {
        match env::var("VACCEL_RPC_ADDRESS") {
            Ok(addr) => addr,
            Err(_) => match env::var("VACCEL_RPC_ADDR") {
                Ok(addr) => addr,
                Err(_) => "tcp://127.0.0.1:65500".to_string(),
            },
        }
    }

    pub(crate) fn resolve_uri(uri: &str) -> Result<String> {
        let parts: Vec<&str> = uri.split("://").collect();
        if parts.len() != 2 {
            return Err(Error::InvalidArgument("Invalid server address uri".into()));
        }

        let scheme = parts[0].to_lowercase();
        match scheme.as_str() {
            "vsock" | "unix" => Ok(uri.to_string()),
            "tcp" => {
                let address = parts[1].to_lowercase();
                let mut resolved = address.to_socket_addrs()?;
                let resolved_address = match resolved.next() {
                    Some(a) => a.to_string(),
                    None => {
                        return Err(Error::Other("Could not resolve TCP server address".into()))
                    }
                };

                Ok(format!("{}://{}", scheme, resolved_address.as_str()))
            }
            _ => Err(Error::Unsupported("Unsupported protocol".into())),
        }
    }

    pub(crate) fn create_ttrpc_client(server_address: &str) -> Result<TtrpcClient> {
        if server_address.is_empty() {
            return Err(Error::InvalidArgument(
                "Server address cannot be empty".into(),
            ));
        }

        let resolved_uri = Self::resolve_uri(server_address)?;

        Ok(TtrpcClient::connect(&resolved_uri)?)
    }
}

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
