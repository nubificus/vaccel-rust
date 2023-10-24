use crate::{util::create_ttrpc_client, Error, Result};
use protocols::sync::agent_ttrpc::VaccelAgentClient;
use std::{collections::BTreeMap, env};
use vaccel::profiling::ProfRegions;

#[repr(C)]
pub struct VsockClient {
    pub ttrpc_client: VaccelAgentClient,
    pub timers: BTreeMap<u32, ProfRegions>,
}

impl VsockClient {
    pub fn new() -> Result<Self> {
        let server_address = match env::var("VACCEL_VSOCK") {
            Ok(addr) => addr,
            Err(_) => "vsock://2:2048".to_string(),
        };

        let ttrpc_client =
            create_ttrpc_client(&server_address).map_err(|e| Error::ClientError(e))?;

        Ok(VsockClient {
            ttrpc_client: VaccelAgentClient::new(ttrpc_client),
            timers: BTreeMap::new(),
        })
    }
}
