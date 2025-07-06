// SPDX-License-Identifier: Apache-2.0

use crate::Result;
use log::debug;
use ttrpc::context::Context;
use vaccel::profiling::ProfilerManager;
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;

#[repr(C)]
pub struct VaccelRpcClient {
    pub ttrpc_client: AgentServiceClient,
    pub profiler_manager: ProfilerManager,
}

impl VaccelRpcClient {
    pub fn new() -> Result<Self> {
        debug!("Client is sync");

        let server_address = Self::get_env_address();
        let ttrpc_client = Self::create_ttrpc_client(&server_address)?;

        Ok(VaccelRpcClient {
            ttrpc_client: AgentServiceClient::new(ttrpc_client),
            profiler_manager: ProfilerManager::new(Self::TIMERS_PREFIX),
        })
    }

    pub fn execute<'a, 'b, F, A, R>(&'a self, func: F, ctx: Context, req: &'b A) -> R
    where
        F: Fn(&'a AgentServiceClient, Context, &'b A) -> R,
    {
        func(&self.ttrpc_client, ctx, req)
    }
}
