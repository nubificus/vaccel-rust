// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, Agent};
use log::{error, info};
use protocols::image::{ImageClassificationRequest, ImageClassificationResponse};

impl Agent {
    pub(crate) fn do_image_classification(
        &self,
        req: ImageClassificationRequest,
    ) -> ttrpc::Result<ImageClassificationResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown Session".to_string())
            })?;

        info!("session:{:?} Image classification", sess.id());
        match sess.image_classification(&req.image) {
            Err(e) => {
                error!("Could not perform classification");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
            Ok((tags, _)) => {
                let mut resp = ImageClassificationResponse::new();
                resp.tags = tags;
                Ok(resp)
            }
        }
    }
}
