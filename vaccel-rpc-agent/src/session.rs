// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use dashmap::mapref::entry::Entry;
use log::info;
use vaccel_rpc_proto::{
    empty::Empty,
    session::{
        CreateSessionRequest, CreateSessionResponse, DestroySessionRequest, UpdateSessionRequest,
    },
};

impl AgentService {
    pub(crate) fn do_create_session(
        &self,
        req: CreateSessionRequest,
    ) -> Result<CreateSessionResponse> {
        let sess = vaccel::Session::new(req.flags)?;

        let mut resp = CreateSessionResponse::new();
        resp.session_id = sess.id().into();

        let e = self.sessions.insert(sess.id(), Box::new(sess));
        assert!(e.is_none());

        info!("Created session {}", resp.session_id);
        Ok(resp)
    }

    pub(crate) fn do_update_session(&self, req: UpdateSessionRequest) -> Result<Empty> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!("Updating hint {} for session {}", req.flags, req.session_id);

        sess.update(req.flags);
        Ok(Empty::new())
    }

    pub(crate) fn do_destroy_session(&self, req: DestroySessionRequest) -> Result<Empty> {
        let (_, mut sess) = self
            .sessions
            .remove(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        if let Entry::Occupied(t) = self.timers.entry(req.session_id.into()) {
            t.remove_entry();
        }
        sess.release()?;

        info!("Destroyed session {}", req.session_id);
        Ok(Empty::new())
    }
}
