// SPDX-License-Identifier: Apache-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::env;
#[cfg(feature = "async")]
use ttrpc::asynchronous::Client as TtrpcClient;
#[cfg(not(feature = "async"))]
use ttrpc::Client as TtrpcClient;
use vaccel::ffi;

pub fn create_ttrpc_client(server_address: &str) -> Result<TtrpcClient, u32> {
    if server_address.is_empty() {
        return Err(ffi::VACCEL_EINVAL);
    }

    let fields: Vec<&str> = server_address.split("://").collect();

    if fields.len() != 2 {
        return Err(ffi::VACCEL_EINVAL);
    }

    let scheme = fields[0].to_lowercase();

    let client: TtrpcClient = match scheme.as_str() {
        "vsock" | "unix" | "tcp" => {
            TtrpcClient::connect(server_address).map_err(|_| ffi::VACCEL_EINVAL)?
        }
        _ => return Err(ffi::VACCEL_ENOTSUP),
    };

    Ok(client)
}

pub(crate) fn get_env_address() -> String {
    match env::var("VACCEL_RPC_ADDRESS") {
        Ok(addr) => addr,
        Err(_) => match env::var("VACCEL_RPC_ADDR") {
            Ok(addr) => addr,
            Err(_) => "tcp://127.0.0.1:65500".to_string(),
        },
    }
}
