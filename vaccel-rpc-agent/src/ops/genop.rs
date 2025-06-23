// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel::{profiling::ProfRegions, Arg};
use vaccel_rpc_proto::genop::{GenopRequest, GenopResponse};

impl AgentService {
    pub(crate) fn do_genop(&self, req: GenopRequest) -> Result<GenopResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        let mut timers = self
            .timers
            .entry(req.session_id.into())
            .or_insert_with(|| ProfRegions::new("vaccel-agent"));
        timers.start("genop > read_args");
        let mut read_args = req
            .read_args
            .into_iter()
            .map(|a| Ok(a.try_into()?))
            .collect::<Result<Vec<Arg>>>()?;
        timers.stop("genop > read_args");

        timers.start("genop > write_args");
        let mut write_args = req
            .write_args
            .into_iter()
            .map(|a| Ok(a.try_into()?))
            .collect::<Result<Vec<Arg>>>()?;
        timers.stop("genop > write_args");

        info!("session:{} Genop", sess.id());
        timers.start("genop > sess.genop");
        sess.genop(
            read_args.as_mut_slice(),
            write_args.as_mut_slice(),
            &mut timers,
        )?;

        let mut resp = GenopResponse::new();
        resp.write_args = write_args.into_iter().map(|e| e.into()).collect();
        timers.stop("genop > sess.genop");

        Ok(resp)
    }
}
