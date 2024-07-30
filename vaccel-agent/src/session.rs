// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, Agent};
use dashmap::mapref::entry::Entry;
use log::info;
#[cfg(feature = "async")]
use protocols::asynchronous::agent::VaccelEmpty;
use protocols::session::{
    CreateSessionRequest, CreateSessionResponse, DestroySessionRequest, UpdateSessionRequest,
};
#[cfg(not(feature = "async"))]
use protocols::sync::agent::VaccelEmpty;

impl Agent {
    pub(crate) fn do_create_session(
        &self,
        req: CreateSessionRequest,
    ) -> ttrpc::Result<CreateSessionResponse> {
        match vaccel::Session::new(req.flags) {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(sess) => {
                let mut resp = CreateSessionResponse::new();
                resp.session_id = sess.id().into();

                let e = self.sessions.insert(sess.id(), Box::new(sess));
                assert!(e.is_none());

                info!("Created session {}", resp.session_id);
                Ok(resp)
            }
        }
    }

    pub(crate) fn do_update_session(
        &self,
        req: UpdateSessionRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown session".to_string(),
            ))?;

        info!("Updating hint {} for session {}", req.flags, req.session_id);

        vaccel::Session::update(&mut sess, req.flags);
        Ok(VaccelEmpty::new())
    }

    pub(crate) fn do_destroy_session(
        &self,
        req: DestroySessionRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let (_, mut sess) = self
            .sessions
            .remove(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        if let Entry::Occupied(t) = self.timers.entry(req.session_id) {
            t.remove_entry();
        }
        match sess.close() {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(()) => {
                info!("Destroyed session {}", req.session_id);
                Ok(VaccelEmpty::new())
            }
        }
    }
}
