// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel::{profiling::SessionProfiler, Arg};
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

        let mut read_args = self.profile_fn(sess.id(), "genop > read_args", || {
            req.read_args
                .into_iter()
                .map(|a| Ok(a.try_into()?))
                .collect::<Result<Vec<Arg>>>()
        })?;

        let mut write_args = self.profile_fn(sess.id(), "genop > write_args", || {
            req.write_args
                .into_iter()
                .map(|a| Ok(a.try_into()?))
                .collect::<Result<Vec<Arg>>>()
        })?;

        info!("session:{} Genop", sess.id());
        self.profile_fn(sess.id(), "genop > sess.genop", || {
            sess.genop(read_args.as_mut_slice(), write_args.as_mut_slice())
        })?;

        let mut resp = GenopResponse::new();
        resp.write_args = self.profile_fn(sess.id(), "genop > resp_write_args", || {
            write_args.into_iter().map(|e| e.into()).collect()
        });

        Ok(resp)
    }
}
