// SPDX-License-Identifier: Apache-2.0

use crate::{
    util::{create_ttrpc_client, get_env_address},
    Error, Result,
};
use dashmap::DashMap;
use log::debug;
use std::{future::Future, sync::Arc};
use tokio::runtime::Runtime;
use ttrpc::context::Context;
use vaccel::profiling::ProfRegions;
use vaccel_rpc_proto::asynchronous::agent_ttrpc::RpcAgentClient;

#[repr(C)]
pub struct VaccelRpcClient {
    pub ttrpc_client: RpcAgentClient,
    pub timers: Arc<DashMap<u32, ProfRegions>>,
    pub runtime: Arc<Runtime>,
}

impl VaccelRpcClient {
    pub fn new() -> Result<Self> {
        debug!("Client is async");

        let r = Runtime::new().unwrap();
        let server_address = get_env_address();

        let _guard = r.enter();
        let ttrpc_client = create_ttrpc_client(&server_address).map_err(Error::ClientError)?;

        Ok(VaccelRpcClient {
            ttrpc_client: RpcAgentClient::new(ttrpc_client),
            timers: Arc::new(DashMap::new()),
            runtime: Arc::new(r),
        })
    }

    pub fn execute<'a, 'b, F, A, R>(&'a self, func: F, ctx: Context, req: &'b A) -> R::Output
    where
        F: Fn(&'a RpcAgentClient, Context, &'b A) -> R,
        R: Future,
    {
        self.runtime
            .block_on(async { func(&self.ttrpc_client, ctx, req).await })
    }
}
