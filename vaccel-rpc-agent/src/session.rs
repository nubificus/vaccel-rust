// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel::Session;
use vaccel_rpc_proto::{
    empty::Empty,
    session::{CreateRequest, CreateResponse, DestroyRequest, UpdateRequest},
};

impl AgentService {
    pub(crate) fn do_create_session(&self, req: CreateRequest) -> Result<CreateResponse> {
        let sess = Session::with_flags(req.flags)?;
        let sess_id = sess.id().ok_or(AgentServiceError::Internal(
            "Invalid session ID".to_string(),
        ))?;

        let mut resp = CreateResponse::new();
        resp.session_id = sess_id.into();

        let e = self.sessions.insert(sess_id, Box::new(sess));
        assert!(e.is_none());

        info!("Created session {}", resp.session_id);
        Ok(resp)
    }

    pub(crate) fn do_update_session(&self, req: UpdateRequest) -> Result<Empty> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!("Updating hint {} for session {}", req.flags, req.session_id);

        sess.update(req.flags);
        Ok(Empty::new())
    }

    pub(crate) fn do_destroy_session(&self, req: DestroyRequest) -> Result<Empty> {
        let (_, sess) = self
            .sessions
            .remove(&req.session_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        self.profiler_manager
            .remove(sess.id().ok_or(AgentServiceError::Internal(
                "Invalid session ID".to_string(),
            ))?);
        drop(sess);

        info!("Destroyed session {}", req.session_id);
        Ok(Empty::new())
    }
}
