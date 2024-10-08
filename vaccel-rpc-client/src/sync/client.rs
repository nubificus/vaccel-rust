// SPDX-License-Identifier: Apache-2.0

use crate::{
    util::{create_ttrpc_client, get_env_address},
    Error, Result,
};
use dashmap::DashMap;
use log::debug;
use std::sync::Arc;
use ttrpc::context::Context;
use vaccel::profiling::ProfRegions;
use vaccel_rpc_proto::sync::agent_ttrpc::RpcAgentClient;

#[repr(C)]
pub struct VaccelRpcClient {
    pub ttrpc_client: RpcAgentClient,
    pub timers: Arc<DashMap<u32, ProfRegions>>,
}

impl VaccelRpcClient {
    pub fn new() -> Result<Self> {
        debug!("Client is sync");

        let server_address = get_env_address();

        let ttrpc_client = create_ttrpc_client(&server_address).map_err(Error::ClientError)?;

        Ok(VaccelRpcClient {
            ttrpc_client: RpcAgentClient::new(ttrpc_client),
            timers: Arc::new(DashMap::new()),
        })
    }

    pub fn execute<'a, 'b, F, A, R>(&'a self, func: F, ctx: Context, req: &'b A) -> R
    where
        F: Fn(&'a RpcAgentClient, Context, &'b A) -> R,
    {
        func(&self.ttrpc_client, ctx, req)
    }
}
