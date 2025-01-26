// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, vaccel_error, AgentService};
use log::info;
use vaccel::{File, Resource, VaccelId};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent::EmptyResponse;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent::EmptyResponse;
#[allow(unused_imports)]
use vaccel_rpc_proto::{
    error::VaccelError,
    resource::{RegisterResourceRequest, RegisterResourceResponse, UnregisterResourceRequest},
};

impl AgentService {
    pub(crate) fn do_register_resource(
        &self,
        req: RegisterResourceRequest,
    ) -> ttrpc::Result<RegisterResourceResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        let res_id = VaccelId::from(req.resource_id);
        let mut resp = RegisterResourceResponse::new();
        if !res_id.has_id() {
            // If we got resource id <= 0 we need to create a resource before registering
            info!("Creating new resource");
            let mut res = match req.files.is_empty() {
                false => {
                    let files: Vec<File> = req.files.iter().map(|f| f.into()).collect();

                    match Resource::from_files(&files, req.resource_type) {
                        Ok(r) => r,
                        Err(e) => {
                            resp.set_error(vaccel_error(e));
                            return Ok(resp);
                        }
                    }
                }
                true => {
                    if req.paths.is_empty() {
                        return Err(ttrpc_error(
                            ttrpc::Code::INVALID_ARGUMENT,
                            "No paths or files provided".to_string(),
                        ));
                    }

                    match Resource::new(&req.paths, req.resource_type) {
                        Ok(r) => r,
                        Err(e) => {
                            resp.set_error(vaccel_error(e));
                            return Ok(resp);
                        }
                    }
                }
            };

            info!(
                "Registering resource {} with session {}",
                res.as_ref().id(),
                req.session_id
            );
            match res.as_mut().register(&mut sess) {
                Ok(_) => {
                    resp.set_resource_id(res.as_ref().id().into());
                    let e = self.resources.insert(res.as_ref().id(), res);
                    assert!(e.is_none());
                }
                Err(e) => {
                    resp.set_error(vaccel_error(e));
                }
            }
        } else {
            // If we got resource id > 0 simply register the resource
            let mut res = self.resources.get_mut(&res_id).ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Unknown resource {}", &res_id).to_string(),
                )
            })?;

            info!(
                "Registering resource {} with session {}",
                res.as_ref().id(),
                req.session_id
            );
            match res.as_mut().register(&mut sess) {
                Ok(_) => {
                    resp.set_resource_id(res.as_ref().id().into());
                }
                Err(e) => {
                    resp.set_error(vaccel_error(e));
                }
            }
        }

        Ok(resp)
    }

    pub(crate) fn do_unregister_resource(
        &self,
        req: UnregisterResourceRequest,
    ) -> ttrpc::Result<EmptyResponse> {
        let mut res = self
            .resources
            .get_mut(&req.resource_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Unknown resource {}", &req.resource_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!(
            "Unregistering resource {} from session {}",
            res.as_ref().id(),
            req.session_id
        );
        res.as_mut()
            .unregister(&mut sess)
            .map_err(|e| ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))?;

        // If resource in registered to other sessions do not destroy
        if res
            .as_ref()
            .refcount()
            .map_err(|e| ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))?
            > 0
        {
            return Ok(EmptyResponse::new());
        }

        info!("Destroying resource {}", res.as_ref().id());
        match res.as_mut().release() {
            Ok(()) => {
                drop(res);
                self.resources
                    .remove(&req.resource_id.into())
                    .ok_or_else(|| {
                        ttrpc_error(
                            ttrpc::Code::INVALID_ARGUMENT,
                            format!("Unknown resource {}", &req.resource_id).to_string(),
                        )
                    })?;

                Ok(EmptyResponse::new())
            }
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
        }
    }
}
