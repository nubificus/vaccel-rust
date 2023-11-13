use crate::{util::create_ttrpc_client, Error, Result};
use dashmap::DashMap;
use log::debug;
use protocols::sync::agent_ttrpc::VaccelAgentClient;
use std::{env, sync::Arc};
use ttrpc::context::Context;
use vaccel::profiling::ProfRegions;

#[repr(C)]
pub struct VsockClient {
    pub ttrpc_client: VaccelAgentClient,
    pub timers: Arc<DashMap<u32, ProfRegions>>,
}

impl VsockClient {
    pub fn new() -> Result<Self> {
        debug!("Client is sync");

        let server_address = match env::var("VACCEL_VSOCK") {
            Ok(addr) => addr,
            Err(_) => "vsock://2:2048".to_string(),
        };

        let ttrpc_client = create_ttrpc_client(&server_address).map_err(Error::ClientError)?;

        Ok(VsockClient {
            ttrpc_client: VaccelAgentClient::new(ttrpc_client),
            timers: Arc::new(DashMap::new()),
        })
    }

    pub fn execute<'a, 'b, F, A, R>(&'a self, func: F, ctx: Context, req: &'b A) -> R
    where
        F: Fn(&'a VaccelAgentClient, Context, &'b A) -> R,
    {
        func(&self.ttrpc_client, ctx, req)
    }
}
