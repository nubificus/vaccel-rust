// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, vaccel_error, VaccelRpcAgent};
use log::info;
use vaccel::{File, Resource};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent::VaccelEmpty;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent::VaccelEmpty;
#[allow(unused_imports)]
use vaccel_rpc_proto::{
    error::VaccelError,
    resource::{RegisterResourceRequest, RegisterResourceResponse, UnregisterResourceRequest},
};

impl VaccelRpcAgent {
    pub(crate) fn do_register_resource(
        &self,
        req: RegisterResourceRequest,
    ) -> ttrpc::Result<RegisterResourceResponse> {
        info!("Creating new resource");

        let res = if !req.files.is_empty() {
            let files: Vec<File> = req.files.iter().map(|f| f.into()).collect();

            Resource::from_files(&files, req.resource_type)
        } else {
            if req.paths.is_empty() {
                return Err(ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "No paths or files provided".to_string(),
                ));
            }

            Resource::new(&req.paths, req.resource_type)
        };

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let mut resp = RegisterResourceResponse::new();
        match res {
            Ok(mut r) => {
                info!(
                    "Registering resource {} with session {}",
                    r.as_ref().id(),
                    req.session_id
                );

                match r.as_mut().register(&mut sess) {
                    Ok(_) => {
                        resp.set_resource_id(r.as_ref().id().into());

                        let e = self.resources.insert(r.as_ref().id(), r);
                        assert!(e.is_none());

                        Ok(resp)
                    }
                    Err(e) => {
                        resp.set_error(vaccel_error(e));
                        Ok(resp)
                    }
                }
            }
            Err(e) => {
                resp.set_error(vaccel_error(e));
                Ok(resp)
            }
        }
    }

    pub(crate) fn do_unregister_resource(
        &self,
        req: UnregisterResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let mut res = self
            .resources
            .get_mut(&req.resource_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown resource".to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        info!(
            "Unregistering resource {} from session {}",
            res.as_ref().id(),
            req.session_id
        );

        res.as_mut()
            .unregister(&mut sess)
            .map_err(|e| ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))?;

        info!("Destroying resource {}", res.as_ref().id());

        match res.as_mut().release() {
            Ok(()) => {
                drop(res);
                self.resources
                    .remove(&req.resource_id.into())
                    .ok_or_else(|| {
                        ttrpc_error(
                            ttrpc::Code::INVALID_ARGUMENT,
                            "Unknown resource".to_string(),
                        )
                    })?;

                Ok(VaccelEmpty::new())
            }
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
        }
    }
}
