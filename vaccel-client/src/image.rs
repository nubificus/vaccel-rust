use crate::client::VsockClient;
use crate::{Error, Result};

use protocols::image::ImageClassificationRequest;
use vaccel::ffi;

use std::os::raw::c_uchar;
use std::slice;

impl VsockClient {
    pub fn image_classify(&self, sess_id: u32, img: Vec<u8>) -> Result<Vec<u8>> {
        let ctx = ttrpc::context::Context::default();
        let mut req = ImageClassificationRequest::default();
        req.set_session_id(sess_id);
        req.set_image(img);

        let resp = self.ttrpc_client.image_classification(ctx, &req)?;
        Ok(resp.tags)
    }
}

#[no_mangle]
pub extern "C" fn image_classify(
    client_ptr: *const VsockClient,
    sess_id: u32,
    img: *const c_uchar,
    img_len: usize,
    tags: *mut c_uchar,
    tags_len: usize,
) -> i32 {
    let img = unsafe { slice::from_raw_parts(img, img_len) };
    let tags_slice = unsafe { slice::from_raw_parts_mut(tags, tags_len) };

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as i32,
    };

    match client.image_classify(sess_id, img.to_vec()) {
        Ok(ret) => {
            tags_slice.copy_from_slice(&ret[..tags_slice.len()]);
            std::mem::forget(tags_slice);

            ffi::VACCEL_OK as i32
        }
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}
