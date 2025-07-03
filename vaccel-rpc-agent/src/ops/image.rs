// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel_rpc_proto::image::{Request, Response};

impl AgentService {
    pub(crate) fn do_image_classification(&self, req: Request) -> Result<Response> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!("session:{} Image classification", &req.session_id);
        let (tags, _) = sess.image_classification(&req.image)?;

        let mut resp = Response::new();
        resp.tags = tags;

        Ok(resp)
    }
}
