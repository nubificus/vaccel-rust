// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel::{Blob, Resource, ResourceType, VaccelId};
#[allow(unused_imports)]
use vaccel_rpc_proto::{
    empty::Empty,
    error::VaccelError,
    resource::{RegisterResourceRequest, RegisterResourceResponse, UnregisterResourceRequest},
};

impl AgentService {
    pub(crate) fn do_register_resource(
        &self,
        req: RegisterResourceRequest,
    ) -> Result<RegisterResourceResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        let res_id = VaccelId::from(req.resource_id);
        let mut resp = RegisterResourceResponse::new();
        if !res_id.has_id() {
            // If we got resource id <= 0 we need to create a resource before registering
            info!("Creating new resource");
            let res_type = ResourceType::from(req.resource_type.value() as u32);
            let mut res = match req.blobs.is_empty() {
                false => {
                    let blobs = req
                        .blobs
                        .into_iter()
                        .map(|f| Ok(f.try_into()?))
                        .collect::<Result<Vec<Blob>>>()?;

                    Resource::from_blobs(blobs, res_type)?
                }
                true => {
                    if req.paths.is_empty() {
                        return Err(AgentServiceError::InvalidArgument(
                            "No paths or blobs provided".to_string(),
                        ));
                    }

                    Resource::new(&req.paths, res_type)?
                }
            };

            info!(
                "Registering resource {} with session {}",
                res.id(),
                req.session_id
            );
            res.register(&mut sess)?;

            resp.resource_id = res.id().into();

            let e = self.resources.insert(res.id(), Box::new(res));
            assert!(e.is_none());
        } else {
            // If we got resource id > 0 simply register the resource
            let mut res = self.resources.get_mut(&res_id).ok_or_else(|| {
                AgentServiceError::NotFound(format!("Unknown resource {}", &res_id).to_string())
            })?;

            info!(
                "Registering resource {} with session {}",
                res.id(),
                req.session_id
            );
            res.register(&mut sess)?;

            resp.resource_id = res.id().into();
        }

        Ok(resp)
    }

    pub(crate) fn do_unregister_resource(&self, req: UnregisterResourceRequest) -> Result<Empty> {
        let mut res = self
            .resources
            .get_mut(&req.resource_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown resource {}", &req.resource_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!(
            "Unregistering resource {} from session {}",
            res.id(),
            req.session_id
        );
        res.unregister(&mut sess)?;

        // If resource in registered to other sessions do not destroy
        let refcount = res.refcount()?;
        if refcount > 0 {
            return Ok(Empty::new());
        }

        info!("Destroying resource {}", res.id());
        drop(res);

        self.resources
            .remove(&req.resource_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown resource {}", &req.resource_id).to_string(),
                )
            })?;

        Ok(Empty::new())
    }
}
