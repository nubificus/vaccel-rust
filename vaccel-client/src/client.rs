use crate::util::create_ttrpc_client;
use protocols::agent_ttrpc::VaccelAgentClient;
use std::env;

#[repr(C)]
pub struct VsockClient {
    pub ttrpc_client: VaccelAgentClient,
}

impl VsockClient {
    pub fn new() -> Result<Self, u32> {
        let server_address = match env::var("VACCEL_VSOCK") {
            Ok(addr) => addr,
            Err(_) => "vsock://1:2048".to_string(),
        };

        let ttrpc_client = create_ttrpc_client(&server_address)?;

        Ok(VsockClient {
            ttrpc_client: VaccelAgentClient::new(ttrpc_client),
        })
    }
}

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
