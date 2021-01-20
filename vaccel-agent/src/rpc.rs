use std::sync::{Arc};

use ttrpc::{self, error::get_rpc_status as ttrpc_error};

use protocols::agent::{CreateSessionResponse, VaccelEmpty};


#[derive(Clone)]
pub struct Agent {
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
        _ctx: &ttrpc::TtrpcContext,
        req: protocols::agent::CreateSessionRequest
    ) -> ttrpc::Result<CreateSessionResponse> {
        let mut sess: vaccel_bindings::vaccel_session = Default::default();
        match vaccel_bindings::new_session(&mut sess, req.flags) {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(()) => {
                let mut resp = CreateSessionResponse::new();
                resp.session_id = sess.session_id;

                Ok(resp)
            }
        }
    }

    fn destroy_session(
        &self,
        _ctx: &ttrpc::TtrpcContext,
        _req: protocols::agent::DestroySessionRequest
    ) -> ttrpc::Result<VaccelEmpty> {
        Ok(VaccelEmpty::new())
    }
}

