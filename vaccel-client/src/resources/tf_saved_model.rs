// SPDX-License-Identifier: Apache-2.0

use super::VaccelResource;
#[cfg(feature = "async")]
use crate::asynchronous::client::VsockClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VsockClient;
use crate::{Error, Result};
use protocols::resources::{CreateResourceRequest, CreateTensorflowSavedModelRequest};
use vaccel::{ffi, resources::TFSavedModel};

impl VaccelResource for TFSavedModel {
    fn create_resource_request(self) -> Result<CreateResourceRequest> {
        let mut model = CreateTensorflowSavedModelRequest::new();
        model.model_pb = self
            .get_protobuf()
            .ok_or(Error::InvalidArgument)?
            .to_owned();

        model.checkpoint = self
            .get_checkpoint()
            .ok_or(Error::InvalidArgument)?
            .to_owned();

        model.var_index = self
            .get_var_index()
            .ok_or(Error::InvalidArgument)?
            .to_owned();

        let mut req = CreateResourceRequest::new();
        req.set_tf_saved_model(model);

        Ok(req)
    }
}

impl VsockClient {}

pub(crate) fn create_tf_saved_model(
    client: &VsockClient,
    model_ptr: *mut ffi::vaccel_tf_saved_model,
) -> ffi::vaccel_id_t {
    let model = TFSavedModel::from_vaccel(model_ptr);
    match client.create_resource(model) {
        Ok(id) => id.into(),
        Err(Error::ClientError(err)) => -(err as ffi::vaccel_id_t),
        Err(_) => -(ffi::VACCEL_EIO as ffi::vaccel_id_t),
    }
}
