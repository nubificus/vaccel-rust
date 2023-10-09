#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use nix::sys::socket::{connect, socket, AddressFamily, SockFlag, SockType, SockaddrIn};
use std::{os::unix::io::RawFd, str::FromStr};
use vaccel::ffi;

fn client_create_sock_fd(address: &str) -> Result<RawFd, u32> {
    let fd = socket(
        AddressFamily::Inet,
        SockType::Stream,
        SockFlag::SOCK_CLOEXEC,
        None,
    )
    .map_err(|_| ffi::VACCEL_EBACKEND)?;

    let sock_addr = SockaddrIn::from_str(address).map_err(|_| ffi::VACCEL_EINVAL)?;

    connect(fd, &sock_addr).map_err(|_| ffi::VACCEL_EIO)?;

    Ok(fd)
}

pub fn create_ttrpc_client(server_address: &String) -> Result<ttrpc::Client, u32> {
    if server_address == "" {
        return Err(ffi::VACCEL_EINVAL);
    }

    let fields: Vec<&str> = server_address.split("://").collect();

    if fields.len() != 2 {
        return Err(ffi::VACCEL_EINVAL);
    }

    let scheme = fields[0].to_lowercase();

    let client: ttrpc::Client = match scheme.as_str() {
        "vsock" | "unix" => {
            ttrpc::Client::connect(&server_address).map_err(|_| ffi::VACCEL_EINVAL)?
        }
        "tcp" => {
            let fd = client_create_sock_fd(fields[1])?;

            ttrpc::Client::new(fd).map_err(|_| ffi::VACCEL_EBACKEND)?
        }

        _ => return Err(ffi::VACCEL_ENOTSUP),
    };

    Ok(client)
}
