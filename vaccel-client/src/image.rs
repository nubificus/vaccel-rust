use crate::client::VsockClient;

use protocols::image::ImageClassificationRequest;
use vaccel_bindings::VACCEL_EIO;

impl VsockClient {
    pub fn image_classify(&self, sess_id: u32, img: Vec<u8>) -> Result<Vec<u8>, u32> {
        let ctx = ttrpc::context::Context::default();
        let mut req = ImageClassificationRequest::default();
        req.set_session_id(sess_id);
        req.set_image(img);

        match self.ttrpc_client.image_classification(ctx, &req) {
            Err(_) => Err(VACCEL_EIO),
            Ok(resp) => Ok(resp.tags),
        }
    }
}
