// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel::{File, Resource, VaccelId};
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
            let mut res = match req.files.is_empty() {
                false => {
                    let files: Vec<File> = req.files.iter().map(|f| f.into()).collect();

                    Resource::from_files(&files, req.resource_type)?
                }
                true => {
                    if req.paths.is_empty() {
                        return Err(AgentServiceError::InvalidArgument(
                            "No paths or files provided".to_string(),
                        ));
                    }

                    Resource::new(&req.paths, req.resource_type)?
                }
            };

            info!(
                "Registering resource {} with session {}",
                res.as_ref().id(),
                req.session_id
            );
            res.as_mut().register(&mut sess)?;

	    let id = res.as_ref().id();
            info!("Final resource ID: {}", id);
            resp.resource_id = id.into();

            //resp.resource_id = res.as_ref().id().into();

            let e = self.resources.insert(id, res);
            assert!(e.is_none());
        } else {
            // If we got resource id > 0 simply register the resource
            let mut res = self.resources.get_mut(&res_id).ok_or_else(|| {
                AgentServiceError::NotFound(format!("Unknown resource {}", &res_id).to_string())
            })?;

            info!(
                "Registering resource {} with session {}",
                res.as_ref().id(),
                req.session_id
            );
            res.as_mut().register(&mut sess)?;
	    let id = res.as_ref().id();
            info!("Final resource ID: {}", id);
            resp.resource_id = id.into();

            //resp.resource_id = res.as_ref().id().into();

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
            res.as_ref().id(),
            req.session_id
        );
        res.as_mut().unregister(&mut sess)?;

        // If resource in registered to other sessions do not destroy
        let refcount = res.as_ref().refcount()?;
        if refcount > 0 {
            return Ok(Empty::new());
        }

        info!("Destroying resource {}", res.as_ref().id());
        res.as_mut().release()?;

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
