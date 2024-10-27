// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use std::{os::raw::c_uchar, slice};
use vaccel::ffi;
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::RpcAgentClient;
use vaccel_rpc_proto::image::ImageClassificationRequest;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::RpcAgentClient;

impl VaccelRpcClient {
    pub fn image_classify(&self, sess_id: i64, img: Vec<u8>) -> Result<Vec<u8>> {
        let ctx = ttrpc::context::Context::default();
        let req = ImageClassificationRequest {
            session_id: sess_id,
            image: img,
            ..Default::default()
        };

        let resp = self.execute(RpcAgentClient::image_classification, ctx, &req)?;

        Ok(resp.tags)
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `img` and `tags` are expected to be valid pointers to objects allocated
/// manually or by the respective vAccel functions.
#[no_mangle]
pub unsafe extern "C" fn image_classify(
    client_ptr: *const VaccelRpcClient,
    sess_id: i64,
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
            ffi::VACCEL_OK as i32
        }
        Err(Error::ClientError(err)) => err as i32,
        Err(_) => ffi::VACCEL_EIO as i32,
    }
}
