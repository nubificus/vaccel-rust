use crate::{util::create_ttrpc_client, Error, Result};
use std::{collections::BTreeMap, env, sync::{Arc, Mutex}};
use vaccel::profiling::ProfRegions;
use tokio::runtime::Runtime;
use protocols::asynchronous::agent_ttrpc::VaccelAgentClient;

#[repr(C)]
pub struct VsockClient {
    pub ttrpc_client: VaccelAgentClient,
    pub timers: Arc<Mutex<BTreeMap<u32, ProfRegions>>>,
    pub runtime: Arc<Runtime>,
}

impl VsockClient {
    pub fn new() -> Result<Self> {
        let r = Runtime::new().unwrap();
        let server_address = match env::var("VACCEL_VSOCK") {
            Ok(addr) => addr,
            Err(_) => "vsock://2:2048".to_string(),
        };

        let _guard = r.enter();
        let ttrpc_client = tokio::task::block_in_place(|| {
            create_ttrpc_client(&server_address).map_err(|e| Error::ClientError(e)).unwrap()
        });

        Ok(VsockClient {
            ttrpc_client: VaccelAgentClient::new(ttrpc_client),
            timers: Arc::new(Mutex::new(BTreeMap::new())),
            runtime: Arc::new(r),
        })
    }
}
