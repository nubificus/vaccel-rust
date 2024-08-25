// SPDX-License-Identifier: Apache-2.0

use super::VaccelResource;
#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use vaccel::{ffi, resources::SingleModel};
use vaccel_rpc_proto::resources::{CreateResourceRequest, CreateSingleModelRequest};

impl VaccelResource for SingleModel {
    fn create_resource_request(self) -> Result<CreateResourceRequest> {
        let mut model = CreateSingleModelRequest::new();
        model.file = self.get_file().ok_or(Error::InvalidArgument)?.to_owned();

        let mut req = CreateResourceRequest::new();
        req.set_single_model(model);

        Ok(req)
    }
}

impl VaccelRpcClient {}

pub(crate) fn create_single_model(
    client: &VaccelRpcClient,
    model_ptr: *mut ffi::vaccel_single_model,
) -> ffi::vaccel_id_t {
    let model = SingleModel::from_vaccel(model_ptr);
    match client.create_resource(model) {
        Ok(id) => id.into(),
        Err(Error::ClientError(err)) => -(err as ffi::vaccel_id_t),
        Err(_) => -(ffi::VACCEL_EIO as ffi::vaccel_id_t),
    }
}
