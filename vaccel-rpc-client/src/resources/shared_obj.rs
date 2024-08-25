// SPDX-License-Identifier: Apache-2.0

use super::VaccelResource;
#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use vaccel::{ffi, resources::SharedObject};
use vaccel_rpc_proto::resources::{CreateResourceRequest, CreateSharedObjRequest};

impl VaccelResource for SharedObject {
    fn create_resource_request(self) -> Result<CreateResourceRequest> {
        let mut sharedobjreq = CreateSharedObjRequest::new();
        let vbytes = self.get_bytes();
        sharedobjreq.shared_obj = vbytes.ok_or(Error::InvalidArgument)?.to_owned();

        let mut req = CreateResourceRequest::new();
        req.set_shared_obj(sharedobjreq);

        Ok(req)
    }
}

impl VaccelRpcClient {}

pub(crate) fn create_shared_object(
    client: &VaccelRpcClient,
    shared_object: *mut ffi::vaccel_shared_object,
) -> ffi::vaccel_id_t {
    let shared_obj = SharedObject::from_vaccel(shared_object);
    match client.create_resource(shared_obj) {
        Ok(id) => id.into(),
        Err(Error::ClientError(err)) => -(err as ffi::vaccel_id_t),
        Err(_) => -(ffi::VACCEL_EIO as ffi::vaccel_id_t),
    }
}
