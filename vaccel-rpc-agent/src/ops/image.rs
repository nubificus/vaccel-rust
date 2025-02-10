// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel_rpc_proto::image::{ImageClassificationRequest, ImageClassificationResponse};

impl AgentService {
    pub(crate) fn do_image_classification(
        &self,
        req: ImageClassificationRequest,
    ) -> Result<ImageClassificationResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!("session:{} Image classification", sess.id());
        let (tags, _) = sess.image_classification(&req.image)?;

        let mut resp = ImageClassificationResponse::new();
        resp.tags = tags;

        Ok(resp)
    }
}
