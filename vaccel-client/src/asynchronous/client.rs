use crate::{util::create_ttrpc_client, Error, Result};
use dashmap::DashMap;
use log::debug;
use protocols::asynchronous::agent_ttrpc::VaccelAgentClient;
use std::{env, future::Future, sync::Arc};
use tokio::runtime::Runtime;
use ttrpc::context::Context;
use vaccel::profiling::ProfRegions;

#[repr(C)]
pub struct VsockClient {
    pub ttrpc_client: VaccelAgentClient,
    pub timers: Arc<DashMap<u32, ProfRegions>>,
    pub runtime: Arc<Runtime>,
}

impl VsockClient {
    pub fn new() -> Result<Self> {
        debug!("Client is async");

        let r = Runtime::new().unwrap();
        let server_address = match env::var("VACCEL_VSOCK") {
            Ok(addr) => addr,
            Err(_) => "vsock://2:2048".to_string(),
        };

        let _guard = r.enter();
        let ttrpc_client = create_ttrpc_client(&server_address).map_err(Error::ClientError)?;

        Ok(VsockClient {
            ttrpc_client: VaccelAgentClient::new(ttrpc_client),
            timers: Arc::new(DashMap::new()),
            runtime: Arc::new(r),
        })
    }

    pub fn execute<'a, 'b, F, A, R>(&'a self, func: F, ctx: Context, req: &'b A) -> R::Output
    where
        F: Fn(&'a VaccelAgentClient, Context, &'b A) -> R,
        R: Future,
    {
        self.runtime
            .block_on(async { func(&self.ttrpc_client, ctx, req).await })
    }
}
