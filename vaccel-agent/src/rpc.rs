use chashmap::*;
use std::sync::Arc;

use vaccel_bindings::resource::VaccelResource;
use vaccel_bindings::{vaccel_id_t, vaccel_session, vaccel_tf_model};

use protocols::image::{ImageClassificationRequest, ImageClassificationResponse};
use protocols::resources::{
    CreateResourceRequest, CreateResourceRequest_oneof_model, CreateResourceResponse,
    CreateTensorflowModelRequest, RegisterResourceRequest, UnregisterResourceRequest,
};
use protocols::session::{CreateSessionRequest, CreateSessionResponse, DestroySessionRequest};
use protocols::{agent::VaccelEmpty, resources::DestroyResourceRequest};

fn ttrpc_error(code: ttrpc::Code, msg: String) -> ttrpc::error::Error {
    ttrpc::Error::RpcStatus(ttrpc::error::get_status(code, msg))
}

#[derive(Clone)]
pub struct Agent {
    sessions: Arc<CHashMap<u32, Box<vaccel_session>>>,
    resources: Arc<CHashMap<vaccel_id_t, Box<dyn VaccelResource>>>,
}

unsafe impl Sync for Agent {}
unsafe impl Send for Agent {}

pub fn start(server_address: &str) -> ttrpc::Server {
    let vaccel_agent = Box::new(Agent {
        sessions: Arc::new(CHashMap::new()),
        resources: Arc::new(CHashMap::new()),
    }) as Box<dyn protocols::agent_ttrpc::VaccelAgent + Send + Sync>;

    let agent_worker = Arc::new(vaccel_agent);

    let aservice = protocols::agent_ttrpc::create_vaccel_agent(agent_worker);

    let server = ttrpc::Server::new()
        .bind(server_address)
        .unwrap()
        .register_service(aservice);

    println!("vaccel ttRPC server started. address:{}", server_address);

    server
}

impl protocols::agent_ttrpc::VaccelAgent for Agent {
    fn create_session(
        &self,
        _ctx: &::ttrpc::TtrpcContext,
        req: CreateSessionRequest,
    ) -> ttrpc::Result<CreateSessionResponse> {
        match vaccel_session::new(req.flags) {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(sess) => {
                let mut resp = CreateSessionResponse::new();
                resp.session_id = sess.session_id;

                assert!(!self.sessions.contains_key(&sess.session_id));
                self.sessions.insert_new(sess.session_id, sess);

                println!("Created session {:?}", resp.session_id);
                Ok(resp)
            }
        }
    }

    fn destroy_session(
        &self,
        _ctx: &::ttrpc::TtrpcContext,
        req: DestroySessionRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let sess = self.sessions.remove(&req.session_id).ok_or(ttrpc_error(
            ttrpc::Code::INVALID_ARGUMENT,
            "Unknown session".to_string(),
        ))?;

        println!("Destroying session {:?}", sess.session_id);
        match sess.close() {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(()) => {
                println!("Destroyed session {:?}", req.session_id);
                Ok(VaccelEmpty::new())
            }
        }
    }

    fn image_classification(
        &self,
        _ctx: &::ttrpc::TtrpcContext,
        req: ImageClassificationRequest,
    ) -> ttrpc::Result<ImageClassificationResponse> {
        let mut sess = self.sessions.get_mut(&req.session_id).ok_or(ttrpc_error(
            ttrpc::Code::INVALID_ARGUMENT,
            "Unknown Session".to_string(),
        ))?;

        println!("session:{:?} Image classification", sess.session_id);
        match sess.image_classification(&req.image) {
            Err(e) => {
                println!("Could not perform classification");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
            Ok((tags, _)) => {
                let mut resp = ImageClassificationResponse::new();
                resp.tags = tags;
                Ok(resp)
            }
        }
    }

    fn create_resource(
        &self,
        _ctx: &ttrpc::TtrpcContext,
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
            CreateResourceRequest_oneof_model::tf(req) => self.create_tf_model(req),
            CreateResourceRequest_oneof_model::caffe(_) => Err(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Caffee models not supported yet".to_string(),
            )),
        }
    }

    fn destroy_resource(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: DestroyResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        match self.resources.remove(&req.resource_id) {
            None => Err(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown resource".to_string(),
            )),
            Some(mut model) => {
                model
                    .destroy()
                    .map_err(|e| ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))?;

                Ok(VaccelEmpty::new())
            }
        }
    }

    fn register_resource(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: RegisterResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let mut resource = self.resources.get_mut(&req.resource_id).ok_or(ttrpc_error(
            ttrpc::Code::INVALID_ARGUMENT,
            "Unknown resource".to_string(),
        ))?;

        let mut sess = self.sessions.get_mut(&req.session_id).ok_or(ttrpc_error(
            ttrpc::Code::INVALID_ARGUMENT,
            "Unknown session".to_string(),
        ))?;

        println!(
            "Registering resource {} to session {}",
            req.resource_id, req.session_id
        );

        match sess.register(&mut **resource) {
            Ok(()) => Ok(VaccelEmpty::new()),
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
        }
    }

    fn unregister_resource(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: UnregisterResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let mut resource = self.resources.get_mut(&req.resource_id).ok_or(ttrpc_error(
            ttrpc::Code::INVALID_ARGUMENT,
            "Unknown resource".to_string(),
        ))?;

        let mut sess = self.sessions.get_mut(&req.session_id).ok_or(ttrpc_error(
            ttrpc::Code::INVALID_ARGUMENT,
            "Unknown session".to_string(),
        ))?;

        match sess.unregister(&mut **resource) {
            Ok(()) => Ok(VaccelEmpty::new()),
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
        }
    }
}

impl Agent {
    fn create_tf_model(
        &self,
        req: CreateTensorflowModelRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        println!("Request to create TensorFlow model resource");
        match vaccel_tf_model::from_buffer(req.get_model()) {
            Ok(model) => {
                let mut resp = CreateResourceResponse::new();
                let id = model.id().ok_or(ttrpc_error(
                    ttrpc::Code::INTERNAL,
                    "Could not get model id".to_string(),
                ))?;
                resp.set_resource_id(id);
                self.resources.insert_new(id, model);

                println!("Created new TensorFlow model with id: {}", id);
                Ok(resp)
            }
            Err(e) => {
                println!("Could not register model");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }
}
