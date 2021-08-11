use chashmap::*;
use std::default::Default;
use std::sync::Arc;

/*
use vaccel_bindings::resource::VaccelResource;
use vaccel_bindings::{
    vaccel_id_t, vaccel_session, vaccel_tf_buffer, vaccel_tf_model, vaccel_tf_node,
    vaccel_tf_tensor, VACCEL_EINVAL, VACCEL_OK,
};
*/

extern crate vaccel;
use vaccel::tensorflow as tf;

use protocols::error::VaccelError;
use protocols::resources::{
    CreateResourceRequest, CreateResourceRequest_oneof_model, CreateResourceResponse,
    CreateTensorflowSavedModelRequest, RegisterResourceRequest, UnregisterResourceRequest,
};
use protocols::session::{CreateSessionRequest, CreateSessionResponse, DestroySessionRequest};
use protocols::{agent::VaccelEmpty, resources::DestroyResourceRequest};
use protocols::{
    image::{ImageClassificationRequest, ImageClassificationResponse},
    tensorflow::{
        InferenceResult, TFTensor, TensorflowModelLoadRequest, TensorflowModelLoadResponse,
        TensorflowModelRunRequest, TensorflowModelRunResponse,
        TensorflowModelRunResponse_oneof_result,
    },
};

fn ttrpc_error(code: ttrpc::Code, msg: String) -> ttrpc::error::Error {
    ttrpc::Error::RpcStatus(ttrpc::error::get_status(code, msg))
}

fn vaccel_error(err: vaccel::Error) -> VaccelError {
    let mut grpc_error = VaccelError::new();

    match err {
        vaccel::Error::Runtime(e) => grpc_error.set_vaccel_error(e as i64),
        vaccel::Error::InvalidArgument => grpc_error.set_agent_error(1i64),
        vaccel::Error::Uninitialized => grpc_error.set_agent_error(2i64),
        vaccel::Error::TensorFlow(_) => grpc_error.set_agent_error(3i64),
    }

    grpc_error
}

#[derive(Clone)]
pub struct Agent {
    sessions: Arc<CHashMap<vaccel::VaccelId, Box<vaccel::Session>>>,
    resources: Arc<CHashMap<vaccel::VaccelId, Box<dyn vaccel::Resource>>>,
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
        match vaccel::Session::new(req.flags) {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(sess) => {
                let mut resp = CreateSessionResponse::new();
                resp.session_id = sess.id().into();

                assert!(!self.sessions.contains_key(&sess.id()));
                self.sessions.insert_new(sess.id(), Box::new(sess));

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
        let mut sess = self
            .sessions
            .remove(&req.session_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown session".to_string(),
            ))?;

        println!("Destroying session {:?}", sess.id());
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
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown Session".to_string(),
            ))?;

        println!("session:{:?} Image classification", sess.id());
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
            CreateResourceRequest_oneof_model::tf_saved(req) => self.create_tf_model(req),
            CreateResourceRequest_oneof_model::caffe(_) => Err(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Caffee models not supported yet".to_string(),
            )),
            CreateResourceRequest_oneof_model::tf(_) => Err(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Frozen model not supported yet".to_string(),
            )),
        }
    }

    fn destroy_resource(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: DestroyResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        match self.resources.remove(&req.resource_id.into()) {
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
        let mut resource = self
            .resources
            .get_mut(&req.resource_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown resource".to_string(),
            ))?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or(ttrpc_error(
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
        let mut resource = self
            .resources
            .get_mut(&req.resource_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown resource".to_string(),
            ))?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown session".to_string(),
            ))?;

        match sess.unregister(&mut **resource) {
            Ok(()) => Ok(VaccelEmpty::new()),
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
        }
    }

    fn tensorflow_model_load(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        req: TensorflowModelLoadRequest,
    ) -> ttrpc::Result<TensorflowModelLoadResponse> {
        let mut resource = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown TensorFlow model".to_string(),
            ))?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown session".to_string(),
            ))?;

        let model = resource
            .as_mut_any()
            .downcast_mut::<tf::SavedModel>()
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                format!("Resource {} is not a TensorFlow model", req.model_id),
            ))?;

        let mut resp = TensorflowModelLoadResponse::new();
        match model.load_graph(&mut sess) {
            Ok(_) => resp.set_graph_def(Vec::new()),
            Err(e) => resp.set_error(vaccel_error(e)),
        };

        Ok(resp)
    }

    fn tensorflow_model_run(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        mut req: TensorflowModelRunRequest,
    ) -> ttrpc::Result<TensorflowModelRunResponse> {
        let mut resource = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown TensorFlow model".to_string(),
            ))?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                "Unknown session".to_string(),
            ))?;

        let model = resource
            .as_mut_any()
            .downcast_mut::<tf::SavedModel>()
            .ok_or(ttrpc_error(
                ttrpc::Code::INVALID_ARGUMENT,
                format!("Resource {} is not a TensorFlow model", req.model_id),
            ))?;

        let mut sess_args = vaccel::ops::inference::InferenceArgs::new();

        let run_options = tf::Buffer::new(req.mut_run_options().as_slice());
        sess_args.set_run_options(&run_options);

        let in_nodes: Vec<tf::Node> = req.get_in_nodes().iter().map(|e| e.into()).collect();
        let in_tensors = req.get_in_tensors();
        for it in in_nodes.iter().zip(in_tensors.iter()) {
            let (node, tensor) = it;
            sess_args.add_input(node, tensor);
        }

        let out_nodes: Vec<tf::Node> = req.get_out_nodes().iter().map(|e| e.into()).collect();
        let num_outputs = out_nodes.len();
        for output in out_nodes.iter() {
            sess_args.request_output(output);
        }

        let response = match model.inference(&mut sess, &mut sess_args) {
            Ok(result) => {
                let mut inference = InferenceResult::new();
                let mut out_tensors: Vec<TFTensor> = Vec::with_capacity(num_outputs);
                for i in 0..num_outputs {
                    out_tensors.push(result.get_grpc_output(i).unwrap());
                }
                inference.set_out_tensors(out_tensors.into());
                TensorflowModelRunResponse_oneof_result::result(inference)
            }
            Err(e) => TensorflowModelRunResponse_oneof_result::error(vaccel_error(e)),
        };

        Ok(TensorflowModelRunResponse {
            result: Some(response),
            ..Default::default()
        })
    }
}

impl Agent {
    fn create_tf_model(
        &self,
        req: CreateTensorflowSavedModelRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        println!("Request to create TensorFlow model resource");
        match tf::SavedModel::new().from_in_memory(
            req.get_model_pb(),
            req.get_checkpoint(),
            req.get_var_index(),
        ) {
            Ok(model) => {
                println!("Created new TensorFlow model with id: {}", model.id());

                let mut resp = CreateResourceResponse::new();
                resp.set_resource_id(model.id().into());
                self.resources.insert_new(model.id(), Box::new(model));

                Ok(resp)
            }
            Err(e) => {
                println!("Could not register model");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }
}
