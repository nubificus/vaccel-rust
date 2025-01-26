// SPDX-License-Identifier: Apache-2.0

use dashmap::DashMap;
use std::{default::Default, pin::Pin, sync::Arc};
use vaccel::{profiling::ProfRegions, Resource, Session, VaccelId};
use vaccel_rpc_proto::profiling::{ProfilingRequest, ProfilingResponse};

#[derive(Clone, Debug)]
pub struct AgentService {
    pub(crate) sessions: Arc<DashMap<VaccelId, Box<Session>>>,
    pub(crate) resources: Arc<DashMap<VaccelId, Pin<Box<Resource>>>>,
    pub(crate) timers: Arc<DashMap<VaccelId, ProfRegions>>,
}

unsafe impl Sync for AgentService {}
unsafe impl Send for AgentService {}

impl AgentService {
    pub(crate) fn new() -> Self {
        AgentService {
            sessions: Arc::new(DashMap::new()),
            resources: Arc::new(DashMap::new()),
            timers: Arc::new(DashMap::new()),
        }
    }

    pub(crate) fn do_get_timers(&self, req: ProfilingRequest) -> ttrpc::Result<ProfilingResponse> {
        let timers = self
            .timers
            .entry(req.session_id.into())
            .or_insert_with(|| ProfRegions::new("vaccel-agent"));

        Ok(ProfilingResponse {
            result: Some(timers.clone().into()).into(),
            ..Default::default()
        })
    }
}
