// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, Agent};
use log::{error, info};
#[cfg(feature = "async")]
use protocols::asynchronous::agent::VaccelEmpty;
#[allow(unused_imports)]
use protocols::resources::{
    create_resource_request::Resource, CreateResourceRequest, CreateResourceResponse,
    CreateSharedObjRequest, CreateSingleModelRequest, CreateTensorflowSavedModelRequest,
    DestroyResourceRequest, RegisterResourceRequest, UnregisterResourceRequest,
};
#[cfg(not(feature = "async"))]
use protocols::sync::agent::VaccelEmpty;
#[cfg(target_pointer_width = "64")]
use vaccel::resources::TFSavedModel;
use vaccel::resources::{SharedObject, SingleModel};

impl Agent {
    pub(crate) fn do_create_resource(
        &self,
        req: CreateResourceRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        let resource = match req.resource {
            None => {
                return Err(ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Invalid model".to_string(),
                ))
            }
            Some(resource) => resource,
        };

        match resource {
            Resource::SharedObj(req) => self.create_shared_object(req),
            Resource::SingleModel(req) => self.create_single_model(req),
            #[cfg(target_pointer_width = "64")]
            Resource::TfSavedModel(req) => self.create_tf_saved_model(req),
            Resource::CaffeModel(_) => Err(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Caffee models not supported yet".to_string(),
            )),
            _ => Err(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Invalid model".to_string(),
            )),
        }
    }

    pub(crate) fn do_destroy_resource(
        &self,
        req: DestroyResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let (_, mut model) = self
            .resources
            .remove(&req.resource_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown resource".to_string(),
                )
            })?;
        model
            .destroy()
            .map_err(|e| ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))?;

        Ok(VaccelEmpty::new())
    }

    pub(crate) fn do_register_resource(
        &self,
        req: RegisterResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let mut resource = self
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
            "Registering resource {} to session {}",
            req.resource_id, req.session_id
        );

        match sess.register(&mut **resource) {
            Ok(()) => Ok(VaccelEmpty::new()),
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
        }
    }

    pub(crate) fn do_unregister_resource(
        &self,
        req: UnregisterResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let mut resource = self
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

        match sess.unregister(&mut **resource) {
            Ok(()) => Ok(VaccelEmpty::new()),
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
        }
    }

    fn create_shared_object(
        &self,
        req: CreateSharedObjRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        info!("Request to create SharedObject resource");
        match SharedObject::from_in_memory(&req.shared_obj) {
            Ok(shared_obj) => {
                info!("Created new Shared Object with id: {}", shared_obj.id());

                let mut resp = CreateResourceResponse::new();
                resp.resource_id = shared_obj.id().into();
                let e = self.resources.insert(shared_obj.id(), Box::new(shared_obj));
                assert!(e.is_none());
                Ok(resp)
            }
            Err(e) => {
                error!("Could not register shared object");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }

    fn create_single_model(
        &self,
        req: CreateSingleModelRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        info!("Request to create SingleModel resource");
        match SingleModel::new().from_in_memory(&req.file) {
            Ok(model) => {
                info!("Created new SingleModel with id: {}", model.id());

                let mut resp = CreateResourceResponse::new();
                resp.resource_id = model.id().into();
                let e = self.resources.insert(model.id(), Box::new(model));
                assert!(e.is_none());

                Ok(resp)
            }
            Err(e) => {
                error!("Could not register model");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }

    #[cfg(target_pointer_width = "64")]
    fn create_tf_saved_model(
        &self,
        req: CreateTensorflowSavedModelRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        info!("Request to create TensorFlow model resource");
        match TFSavedModel::new().from_in_memory(&req.model_pb, &req.checkpoint, &req.var_index) {
            Ok(model) => {
                info!("Created new TensorFlow model with id: {}", model.id());

                let mut resp = CreateResourceResponse::new();
                resp.resource_id = model.id().into();
                let e = self.resources.insert(model.id(), Box::new(model));
                assert!(e.is_none());

                Ok(resp)
            }
            Err(e) => {
                error!("Could not register model");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }
}
