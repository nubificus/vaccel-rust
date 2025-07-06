// SPDX-License-Identifier: Apache-2.0

use crate::Result;
use log::debug;
use std::{future::Future, sync::Arc};
use tokio::runtime::Runtime;
use ttrpc::context::Context;
use vaccel::profiling::ProfilerManager;
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;

#[repr(C)]
pub struct VaccelRpcClient {
    pub ttrpc_client: AgentServiceClient,
    pub profiler_manager: ProfilerManager,
    pub runtime: Arc<Runtime>,
}

impl VaccelRpcClient {
    pub fn new() -> Result<Self> {
        debug!("Client is async");

        let r = Runtime::new().unwrap();
        let server_address = Self::get_env_address();

        let _guard = r.enter();
        let ttrpc_client = Self::create_ttrpc_client(&server_address)?;

        Ok(VaccelRpcClient {
            ttrpc_client: AgentServiceClient::new(ttrpc_client),
            profiler_manager: ProfilerManager::new(Self::TIMERS_PREFIX),
            runtime: Arc::new(r),
        })
    }

    pub fn execute<'a, 'b, F, A, R>(&'a self, func: F, ctx: Context, req: &'b A) -> R::Output
    where
        F: Fn(&'a AgentServiceClient, Context, &'b A) -> R,
        R: Future,
    {
        self.runtime
            .block_on(async { func(&self.ttrpc_client, ctx, req).await })
    }
}
