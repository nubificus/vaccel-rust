use std::sync::{Arc};

use protocols::agent::{
    CreateSessionRequest, CreateSessionResponse,
    DestroySessionRequest, VaccelEmpty,
    ImageClassificationRequest, ImageClassificationResponse
};

#[derive(Clone)]
pub struct Agent {
}

fn ttrpc_error(code: ttrpc::Code, msg: String) -> ttrpc::error::Error {
    ttrpc::Error::RpcStatus(ttrpc::error::get_status(code, msg))
}

pub fn start(server_address: &str) -> ttrpc::Server {
    let vaccel_agent = Box::new(Agent{})
        as Box<dyn protocols::agent_ttrpc::VaccelAgent + Send + Sync>;

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
        req: CreateSessionRequest
    ) -> ttrpc::Result<CreateSessionResponse> {
        let mut sess: vaccel_bindings::vaccel_session = Default::default();
        match vaccel_bindings::new_session(&mut sess, req.flags) {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(()) => {
                let mut resp = CreateSessionResponse::new();
                resp.session_id = sess.session_id;

                println!("Created session {:?}", sess.session_id);

                Ok(resp)
            }
        }
    }

    fn destroy_session(
        &self,
        _ctx: &::ttrpc::TtrpcContext,
        req: DestroySessionRequest
    ) -> ttrpc::Result<VaccelEmpty> {
        let mut sess: vaccel_bindings::vaccel_session = Default::default();
        sess.session_id = req.session_id;
        println!("Destroying session {:?}", sess.session_id);
        match vaccel_bindings::close_session(&mut sess) {
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
        req: ImageClassificationRequest
    ) -> ttrpc::Result<ImageClassificationResponse> {
        let mut sess: vaccel_bindings::vaccel_session = Default::default();
        let mut tags = vec![0; 1024];
        let mut image_path = vec![0; 1024];

        sess.session_id = req.session_id;
        println!("session:{:?} Image classification", sess.session_id);
        match vaccel_bindings::image_classification(
        &mut sess, &req.image, &mut tags, &mut image_path) {
            Err(e) => {
                println!("Could not perform classification");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            },
            Ok(()) => {
                let mut resp = ImageClassificationResponse::new();
                resp.tags.append(&mut tags);
                Ok(resp)
            }
        }
    }
}

