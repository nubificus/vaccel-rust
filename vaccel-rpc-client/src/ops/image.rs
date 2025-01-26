// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use log::error;
use std::{
    ffi::{c_int, c_uchar},
    slice,
};
use vaccel::ffi;
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;
use vaccel_rpc_proto::image::ImageClassificationRequest;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;

impl VaccelRpcClient {
    pub fn image_classify(&self, sess_id: i64, img: Vec<u8>) -> Result<Vec<u8>> {
        let ctx = ttrpc::context::Context::default();
        let req = ImageClassificationRequest {
            session_id: sess_id,
            image: img,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::image_classification, ctx, &req)?;

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
pub unsafe extern "C" fn vaccel_rpc_client_image_classify(
    client_ptr: *const VaccelRpcClient,
    sess_id: i64,
    img: *const c_uchar,
    img_len: usize,
    tags: *mut c_uchar,
    tags_len: usize,
) -> c_int {
    let img = unsafe { slice::from_raw_parts(img, img_len) };
    let tags_slice = unsafe { slice::from_raw_parts_mut(tags, tags_len) };

    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    match client.image_classify(sess_id, img.to_vec()) {
        Ok(ret) => {
            tags_slice.copy_from_slice(&ret[..tags_slice.len()]);
            ffi::VACCEL_OK as c_int
        }
        Err(e) => {
            error!("{}", e);
            match e {
                Error::ClientError(_) => ffi::VACCEL_EBACKEND as c_int,
                _ => ffi::VACCEL_EIO as c_int,
            }
        }
    }
}
