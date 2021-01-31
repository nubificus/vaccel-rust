use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use vaccel_bindings::vaccel_session;

use protocols::agent::{
    CreateSessionRequest, CreateSessionResponse,
    DestroySessionRequest, VaccelEmpty,
    ImageClassificationRequest, ImageClassificationResponse
};


fn ttrpc_error(code: ttrpc::Code, msg: String) -> ttrpc::error::Error {
    ttrpc::Error::RpcStatus(ttrpc::error::get_status(code, msg))
}

#[derive(Clone)]
pub struct Agent {
    sessions: Arc<Mutex<HashMap<u32, vaccel_session>>>,
}

unsafe impl Sync for Agent {}
unsafe impl Send for Agent {}

pub fn start(server_address: &str) -> ttrpc::Server {
    let vaccel_agent =
        Box::new(
            Agent {
                sessions: Arc::new(Mutex::new(HashMap::new()))
            }
        ) as Box<dyn protocols::agent_ttrpc::VaccelAgent + Send + Sync>;

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
        match vaccel_session::new(req.flags) {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(sess) => {
                let mut resp = CreateSessionResponse::new();
                resp.session_id = sess.session_id;

                let mut s = self.sessions.lock().unwrap();
                s.insert(sess.session_id, sess);

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
        let mut s = self.sessions.lock().unwrap();
        let sess = match s.get_mut(&req.session_id) {
            Some(sess) => sess,
            None => return Err(ttrpc_error(ttrpc::Code::UNAVAILABLE, "Unknown session".to_string())),
        };

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
        req: ImageClassificationRequest
    ) -> ttrpc::Result<ImageClassificationResponse> {
        let mut s = self.sessions.lock().unwrap();
        let sess = match s.get_mut(&req.session_id) {
            Some(sess) => sess,
            None => return Err(ttrpc_error(ttrpc::Code::UNAVAILABLE, "Unknown session".to_string())),
        };

        println!("session:{:?} Image classification", sess.session_id);
        match sess.image_classification(&req.image) {
            Err(e) => {
                println!("Could not perform classification");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            },
            Ok((tags, _)) => {
                let mut resp = ImageClassificationResponse::new();
                resp.tags = tags;
                Ok(resp)
            }
        }
    }
}

