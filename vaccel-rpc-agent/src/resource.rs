// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel::{Blob, Resource, ResourceType, VaccelId};
use vaccel_rpc_proto::{
    empty::Empty,
    resource::{
        Blob as ProtoBlob, RegisterRequest, RegisterResponse, SyncRequest, SyncResponse,
        UnregisterRequest,
    },
};

impl AgentService {
    pub(crate) fn do_register_resource(&self, req: RegisterRequest) -> Result<RegisterResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        let proto_res_id = VaccelId::from_ffi(req.resource_id)?;
        let mut resp = RegisterResponse::new();
        if proto_res_id.is_none() {
            // If we got resource id == 0 we need to create a resource before registering
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

            let res_id = res.id().ok_or(AgentServiceError::Internal(
                "Invalid resource ID".to_string(),
            ))?;

            info!(
                "Registering resource {} with session {}",
                res_id, req.session_id
            );
            res.register(&mut sess)?;

            resp.resource_id = res_id.into();

            let e = self.resources.insert(res_id, Box::new(res));
            assert!(e.is_none());
        } else {
            // If we got resource id > 0 simply register the resource
            let mut res = self
                .resources
                .get_mut(&proto_res_id.unwrap())
                .ok_or_else(|| {
                    AgentServiceError::NotFound(
                        format!("Unknown resource {}", &proto_res_id.unwrap()).to_string(),
                    )
                })?;

            let res_id = res.id().ok_or(AgentServiceError::Internal(
                "Invalid resource ID".to_string(),
            ))?;

            info!(
                "Registering resource {} with session {}",
                res_id, req.session_id
            );
            res.register(&mut sess)?;

            resp.resource_id = res_id.into();
        }

        Ok(resp)
    }

    pub(crate) fn do_unregister_resource(&self, req: UnregisterRequest) -> Result<Empty> {
        let mut res = self
            .resources
            .get_mut(&req.resource_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown resource {}", &req.resource_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        let res_id = res.id().ok_or(AgentServiceError::Internal(
            "Invalid resource ID".to_string(),
        ))?;

        info!(
            "Unregistering resource {} from session {}",
            res_id, req.session_id
        );
        res.unregister(&mut sess)?;

        // If resource in registered to other sessions do not destroy
        let refcount = res.refcount()?;
        if refcount > 0 {
            return Ok(Empty::new());
        }

        info!("Destroying resource {}", res_id);
        drop(res);

        self.resources.remove(&res_id).ok_or_else(|| {
            AgentServiceError::NotFound(format!("Unknown resource {}", &res_id).to_string())
        })?;

        Ok(Empty::new())
    }

    pub(crate) fn do_sync_resource(&self, req: SyncRequest) -> Result<SyncResponse> {
        let res = self
            .resources
            .get(&req.resource_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown resource {}", &req.resource_id).to_string(),
                )
            })?;

        info!("Synchronizing resource {}", &req.resource_id);

        let blobs = res
            .blobs()
            .ok_or(AgentServiceError::Internal("No blobs found".to_string()))?;

        let proto_blobs = blobs
            .iter()
            .map(|blob| {
                let data = blob
                    .data()
                    .map(|d| d.to_vec())
                    .ok_or(AgentServiceError::Internal("Blob has no data".to_string()))?;

                Ok(ProtoBlob {
                    data,
                    size: blob.size() as u32,
                    ..Default::default()
                })
            })
            .collect::<Result<Vec<ProtoBlob>>>()?;

        let mut resp = SyncResponse::new();
        resp.blobs = proto_blobs;

        Ok(resp)
    }
}
