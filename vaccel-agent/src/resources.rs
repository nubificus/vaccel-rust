use crate::{ttrpc_error, Agent};
use log::{error, info};
#[cfg(feature = "async")]
use protocols::asynchronous::agent::VaccelEmpty;
use protocols::resources::{
    create_resource_request::Model as CreateResourceRequestModel, CreateResourceRequest,
    CreateResourceResponse, CreateSharedObjRequest, DestroyResourceRequest,
    RegisterResourceRequest, UnregisterResourceRequest,
};
#[cfg(not(feature = "async"))]
use protocols::sync::agent::VaccelEmpty;
use vaccel::shared_obj as so;

impl Agent {
    pub(crate) fn do_create_resource(
        &self,
        req: CreateResourceRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        let model = match req.model {
            None => {
                return Err(ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Invalid model".to_string(),
                ))
            }
            Some(model) => model,
        };

        match model {
            CreateResourceRequestModel::SharedObj(req) => self.create_shared_object(req),
            CreateResourceRequestModel::TfSaved(req) => self.create_tf_model(req),
            CreateResourceRequestModel::Caffe(_) => Err(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Caffee models not supported yet".to_string(),
            )),
            CreateResourceRequestModel::Tf(_) => Err(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Frozen model not supported yet".to_string(),
            )),
            CreateResourceRequestModel::TorchSaved(req) => self.create_torch_model(req),
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

    pub(crate) fn create_shared_object(
        &self,
        req: CreateSharedObjRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        info!("Request to create SharedObject resource");
        match so::SharedObject::from_in_memory(&req.shared_obj) {
            Ok(shared_obj) => {
                info!("Created new Shared Object with id: {}", shared_obj.id());

                let mut resp = CreateResourceResponse::new();
                resp.resource_id = shared_obj.id().into();
                let e = self.resources.insert(shared_obj.id(), Box::new(shared_obj));
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
