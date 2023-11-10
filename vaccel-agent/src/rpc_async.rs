use chashmap::*;
// use::std::time::Instant
use protocols::{
    asynchronous::agent::VaccelEmpty,
    error::VaccelError,
    genop::{GenopArg, GenopRequest, GenopResponse, GenopResult},
    image::{ImageClassificationRequest, ImageClassificationResponse},
    profiling::{ProfilingRequest, ProfilingResponse},
    resources::{
        create_resource_request::Model as CreateResourceRequestModel, CreateResourceRequest,
        CreateResourceResponse, CreateSharedObjRequest, CreateTensorflowSavedModelRequest,
        CreateTorchSavedModelRequest, DestroyResourceRequest, RegisterResourceRequest,
        UnregisterResourceRequest,
    },
    session::{CreateSessionRequest, CreateSessionResponse, DestroySessionRequest},
    tensorflow::{
        InferenceResult, TFTensor, TensorflowModelLoadRequest, TensorflowModelLoadResponse,
        TensorflowModelRunRequest, TensorflowModelRunResponse, TensorflowModelUnloadRequest,
        TensorflowModelUnloadResponse,
    },
    torch::{
        TorchJitloadForwardRequest, TorchJitloadForwardResponse, TorchJitloadForwardResult,
        TorchTensor,
    },
};
use std::{
    collections::btree_map::Entry,
    collections::BTreeMap,
    default::Default,
    error::Error,
    sync::{Arc, Mutex},
};
extern crate vaccel;
use async_trait::async_trait;
use ttrpc::asynchronous::Server;
use vaccel::{ops::genop, profiling::ProfRegions, shared_obj as so, tensorflow as tf, torch};
//use tracing::{info, instrument, Instrument};
use log::{debug, error, info};

fn ttrpc_error(code: ttrpc::Code, msg: String) -> ttrpc::Error {
    ttrpc::Error::RpcStatus(ttrpc::error::get_status(code, msg))
}

fn vaccel_error(err: vaccel::Error) -> VaccelError {
    let mut grpc_error = VaccelError::new();

    match err {
        vaccel::Error::Runtime(e) => grpc_error.set_vaccel_error(e as i64),
        vaccel::Error::InvalidArgument => grpc_error.set_agent_error(1i64),
        vaccel::Error::Uninitialized => grpc_error.set_agent_error(2i64),
        vaccel::Error::TensorFlow(_) => grpc_error.set_agent_error(3i64),
        vaccel::Error::Torch(_) => grpc_error.set_agent_error(4i64),
    }

    grpc_error
}

#[derive(Clone)]
pub struct Agent {
    sessions: Arc<CHashMap<vaccel::VaccelId, Box<vaccel::Session>>>,
    resources: Arc<CHashMap<vaccel::VaccelId, Box<dyn vaccel::Resource>>>,
    timers: Arc<Mutex<BTreeMap<u32, ProfRegions>>>,
}

unsafe impl Sync for Agent {}
unsafe impl Send for Agent {}

pub fn new(server_address: &str) -> Result<Server, Box<dyn Error>> {
    let vaccel_agent = Box::new(Agent {
        sessions: Arc::new(CHashMap::new()),
        resources: Arc::new(CHashMap::new()),
        timers: Arc::new(Mutex::new(BTreeMap::new())),
    })
        as Box<dyn protocols::asynchronous::agent_ttrpc::VaccelAgent + Send + Sync>;

    let agent_worker = Arc::new(vaccel_agent);
    let aservice = protocols::asynchronous::agent_ttrpc::create_vaccel_agent(agent_worker);

    if server_address.is_empty() {
        return Err("Server address cannot be empty".into());
    }

    let fields: Vec<&str> = server_address.split("://").collect();
    if fields.len() != 2 {
        return Err("Invalid address".into());
    }

    let scheme = fields[0].to_lowercase();
    let server: Server = match scheme.as_str() {
        "vsock" | "unix" | "tcp" => Server::new()
            .bind(server_address)?
            .register_service(aservice),
        _ => return Err("Unsupported protocol".into()),
    };

    Ok(server)
}

#[async_trait]
impl protocols::asynchronous::agent_ttrpc::VaccelAgent for Agent {
    async fn create_session(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: CreateSessionRequest,
    ) -> ttrpc::Result<CreateSessionResponse> {
        match vaccel::Session::new(req.flags) {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(sess) => {
                let mut resp = CreateSessionResponse::new();
                resp.session_id = sess.id().into();

                assert!(!self.sessions.contains_key(&sess.id()));
                self.sessions.insert_new(sess.id(), Box::new(sess));

                info!("Created session {}", resp.session_id);
                Ok(resp)
            }
        }
    }

    async fn destroy_session(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: DestroySessionRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        let mut sess = self
            .sessions
            .remove(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let mut timers_lock = self.timers.lock().unwrap();
        if let Entry::Occupied(t) = timers_lock.entry(req.session_id) {
            t.remove_entry();
        }
        match sess.close() {
            Err(e) => Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string())),
            Ok(()) => {
                info!("Destroyed session {}", req.session_id);
                Ok(VaccelEmpty::new())
            }
        }
    }

    async fn image_classification(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: ImageClassificationRequest,
    ) -> ttrpc::Result<ImageClassificationResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown Session".to_string())
            })?;

        info!("session:{:?} Image classification", sess.id());
        match sess.image_classification(&req.image) {
            Err(e) => {
                error!("Could not perform classification");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
            Ok((tags, _)) => {
                let mut resp = ImageClassificationResponse::new();
                resp.tags = tags;
                Ok(resp)
            }
        }
    }

    async fn create_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
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
            _ => {
                return Err(ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Invalid model".to_string(),
                ))
            }
        }
    }

    async fn destroy_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
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

    async fn register_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
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

    async fn unregister_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
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

    async fn tensorflow_model_load(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowModelLoadRequest,
    ) -> ttrpc::Result<TensorflowModelLoadResponse> {
        let mut resource = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown TensorFlow model".to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let model = resource
            .as_mut_any()
            .downcast_mut::<tf::SavedModel>()
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Resource {} is not a TensorFlow model", req.model_id),
                )
            })?;

        let mut resp = TensorflowModelLoadResponse::new();
        match model.session_load(&mut sess) {
            Ok(_) => resp.set_graph_def(Vec::new()),
            Err(e) => resp.set_error(vaccel_error(e)),
        };

        Ok(resp)
    }

    async fn tensorflow_model_unload(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowModelUnloadRequest,
    ) -> ttrpc::Result<TensorflowModelUnloadResponse> {
        let mut resource = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown TensorFlow model".to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown vAccel session".to_string(),
                )
            })?;

        let model = resource
            .as_mut_any()
            .downcast_mut::<tf::SavedModel>()
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Resource {} is not a TensorFlow model", req.model_id),
                )
            })?;

        let mut resp = TensorflowModelUnloadResponse::new();
        match model.session_delete(&mut sess) {
            Ok(_) => resp.success = true,
            Err(e) => resp.error = Some(vaccel_error(e)).into(),
        };

        Ok(resp)
    }

    async fn tensorflow_model_run(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowModelRunRequest,
    ) -> ttrpc::Result<TensorflowModelRunResponse> {
        let mut resource = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown TensorFlow model".to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let model = resource
            .as_mut_any()
            .downcast_mut::<tf::SavedModel>()
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Resource {} is not a TensorFlow model", req.model_id),
                )
            })?;

        let mut sess_args = vaccel::ops::inference::InferenceArgs::new();

        let run_options = tf::Buffer::new(req.run_options.as_slice());
        sess_args.set_run_options(&run_options);

        let in_nodes: Vec<tf::Node> = req.in_nodes.iter().map(|e| e.into()).collect();
        let in_tensors = req.in_tensors;
        for it in in_nodes.iter().zip(in_tensors.iter()) {
            let (node, tensor) = it;
            debug!("tensor.dim: {:?}", tensor.dims);
            sess_args.add_input(node, tensor);
        }

        let out_nodes: Vec<tf::Node> = req.out_nodes.iter().map(|e| e.into()).collect();
        let num_outputs = out_nodes.len();
        for output in out_nodes.iter() {
            sess_args.request_output(output);
        }

        let response = match model.session_run(&mut sess, &mut sess_args) {
            Ok(result) => {
                let mut inference = InferenceResult::new();
                let mut out_tensors: Vec<TFTensor> = Vec::with_capacity(num_outputs);
                for i in 0..num_outputs {
                    out_tensors.push(result.get_grpc_output(i).unwrap());
                }
                inference.out_tensors = out_tensors;
                let mut resp = TensorflowModelRunResponse::new();
                resp.set_result(inference);
                resp
            }
            Err(e) => {
                let mut resp = TensorflowModelRunResponse::new();
                resp.set_error(vaccel_error(e));
                resp
            }
        };

        Ok(response)
    }

    async fn torch_jitload_forward(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TorchJitloadForwardRequest,
    ) -> ttrpc::Result<TorchJitloadForwardResponse> {
        let mut resource = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown PyTorch model".to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let model = resource
            .as_mut_any()
            .downcast_mut::<torch::SavedModel>()
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Resource {} is not a pytorch model", req.model_id),
                )
            })?;

        // origin: vaccel::ops::inference...
        let mut sess_args = torch::TorchArgs::new();
        let mut jitload = torch::TorchJitLoadForward::new();

        let run_options = torch::Buffer::new(req.run_options.as_slice());
        sess_args.set_run_options(&run_options);

        let in_tensors = req.in_tensors;
        for tensor in in_tensors.iter() {
            sess_args.add_input(tensor);
        }

        // TODO: bindings examples
        /*
        let response = jitload.jitload_forward(&mut sess, &mut sess_args, &mut model)?;
        match response.get_output::<f32>(0) {
            Ok(result) => {
                println!("Success");
                println!(
                    "Output tensor => type:{:?} nr_dims:{}",
                    result.data_type(),
                    result.nr_dims()
                );
                for i in 0..result.nr_dims() {
                    println!("dim[{}]: {}", i, result.dim(i as usize).unwrap());
                }
            }
            // Err(e) => println!("Torch JitLoadForward failed: '{}'", e),
        }
        Ok(TorchJitloadForwardResponse {
            result: Some(response),
           ..Default::default()
        })
        */

        // TODO
        // let num_outputs = in_tensors.len();
        let num_outputs: usize = 1;

        //println!("NUM of output: {}, Type: {}", num_outputs, type_of(&num_outputs));
        let response = match jitload.jitload_forward(&mut sess, &mut sess_args, model) {
            Ok(result) => {
                let mut jitload_forward = TorchJitloadForwardResult::new();
                let mut out_tensors: Vec<TorchTensor> = Vec::with_capacity(num_outputs);
                for i in 0..num_outputs {
                    out_tensors.push(result.get_grpc_output(i).unwrap());
                }
                jitload_forward.out_tensors = out_tensors;
                let mut resp = TorchJitloadForwardResponse::new();
                resp.set_result(jitload_forward);
                resp
            }
            Err(e) => {
                let mut resp = TorchJitloadForwardResponse::new();
                resp.set_error(vaccel_error(e));
                resp
            }
        };

        Ok(response)
    }

    async fn genop(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        mut req: GenopRequest,
    ) -> ttrpc::Result<GenopResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        // FIXME: This will lock until the function finishes
        let mut timers_lock = self.timers.lock().unwrap();
        let timers = timers_lock
            .entry(req.session_id)
            .or_insert_with(|| ProfRegions::new("vaccel-agent"));
        timers.start("genop > read_args");
        let mut read_args: Vec<genop::GenopArg> =
            req.read_args.iter_mut().map(|e| e.into()).collect();
        timers.stop("genop > read_args");

        timers.start("genop > write_args");
        let mut write_args: Vec<genop::GenopArg> =
            req.write_args.iter_mut().map(|e| e.into()).collect();
        timers.stop("genop > write_args");

        info!("Genop session {}", sess.id());
        timers.start("genop > sess.genop");
        let response = match sess.genop(read_args.as_mut_slice(), write_args.as_mut_slice(), timers)
        {
            Ok(_) => {
                let mut res = GenopResult::new();
                res.write_args = write_args.iter().map(|e| e.into()).collect();
                let mut resp = GenopResponse::new();
                resp.set_result(res);
                resp
            }
            Err(e) => {
                let mut resp = GenopResponse::new();
                resp.set_error(vaccel_error(e));
                resp
            }
        };
        timers.stop("genop > sess.genop");

        //timers.print();
        //timers.print_total();

        Ok(response)
    }

    async fn genop_stream(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        mut r: ::ttrpc::asynchronous::ServerStreamReceiver<GenopRequest>,
    ) -> ttrpc::Result<GenopResponse> {
        let mut req = GenopRequest::default();
        let mut r_arg = vec![GenopArg::new()];
        let mut w_arg = vec![GenopArg::new()];
        while let Some(mut data) = r.recv().await? {
            req.session_id = data.session_id;
            if data.read_args.len() == 1 && data.read_args[0].parts > 0 {
                if data.read_args[0].part_no < data.read_args[0].parts {
                    r_arg[0].buf.append(&mut data.read_args[0].buf);
                } else {
                    //println!("ASSEMBLED: {:?}", req.read_args);
                    r_arg[0].buf.append(&mut data.read_args[0].buf);
                    r_arg[0].size = data.read_args[0].size;
                    //println!("APPEND IN");
                    req.read_args.append(&mut r_arg);
                    r_arg = vec![GenopArg::new()];
                }
                if data.write_args.len() == 1 && data.write_args[0].parts > 0 {
                    if data.write_args[0].part_no < data.write_args[0].parts {
                        w_arg[0].buf.append(&mut data.write_args[0].buf);
                    } else {
                        //println!("ASSEMBLED: {:?}", req.read_args);
                        w_arg[0].buf.append(&mut data.write_args[0].buf);
                        w_arg[0].size = data.write_args[0].size;
                        //println!("APPEND IN");
                        req.write_args.append(&mut w_arg);
                        w_arg = vec![GenopArg::new()];
                    }
                }
            } else {
                //println!("APPEND");
                req.read_args.append(&mut data.read_args);
                req.write_args.append(&mut data.write_args);
            }
        }
        //println!("LEN: {:?}", &req.read_args.len());
        //for r in &req.read_args {
        //    println!("LEN: {:?}", r.buf.len());
        //}
        //println!("BYTES: {:?}", bytes);
        //req.merge_from_bytes(&bytes).unwrap();
        //let mut req = GenopStreamRequestRaw::from_bytes(&bytes);

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        // TODO: This will lock until the function finishes
        let mut timers_lock = self.timers.lock().unwrap();
        let timers = timers_lock
            .entry(req.session_id)
            .or_insert_with(|| ProfRegions::new("vaccel-agent"));
        timers.start("genop > read_args");
        let mut read_args: Vec<genop::GenopArg> =
            req.read_args.iter_mut().map(|e| e.into()).collect();
        timers.stop("genop > read_args");

        timers.start("genop > write_args");
        let mut write_args: Vec<genop::GenopArg> =
            req.write_args.iter_mut().map(|e| e.into()).collect();
        timers.stop("genop > write_args");

        info!("Genop stream session {}", sess.id());
        timers.start("genop > sess.genop");
        let response = match sess.genop(read_args.as_mut_slice(), write_args.as_mut_slice(), timers)
        {
            Ok(_) => {
                let mut res = GenopResult::new();
                res.write_args = write_args.iter().map(|e| e.into()).collect();
                let mut resp = GenopResponse::new();
                resp.set_result(res);
                resp
            }
            Err(e) => {
                let mut resp = GenopResponse::new();
                resp.set_error(vaccel_error(e));
                resp
            }
        };
        timers.stop("genop > sess.genop");

        //timers.print();
        //timers.print_total();

        Ok(response)
    }

    async fn get_timers(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: ProfilingRequest,
    ) -> ttrpc::Result<ProfilingResponse> {
        let mut timers_lock = self.timers.lock().unwrap();
        let timers = timers_lock
            .entry(req.session_id)
            .or_insert_with(|| ProfRegions::new("vaccel-agent"));

        Ok(ProfilingResponse {
            result: Some(timers.clone().into()).into(),
            ..Default::default()
        })
    }
}

impl Agent {
    fn create_tf_model(
        &self,
        req: CreateTensorflowSavedModelRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        info!("Request to create TensorFlow model resource");
        match tf::SavedModel::new().from_in_memory(&req.model_pb, &req.checkpoint, &req.var_index) {
            Ok(model) => {
                info!("Created new TensorFlow model with id: {}", model.id());

                let mut resp = CreateResourceResponse::new();
                resp.resource_id = model.id().into();
                self.resources.insert_new(model.id(), Box::new(model));

                Ok(resp)
            }
            Err(e) => {
                error!("Could not register model");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }

    fn create_shared_object(
        &self,
        req: CreateSharedObjRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        info!("Request to create SharedObject resource");
        match so::SharedObject::from_in_memory(&req.shared_obj) {
            Ok(shared_obj) => {
                info!("Created new Shared Object with id: {}", shared_obj.id());

                let mut resp = CreateResourceResponse::new();
                resp.resource_id = shared_obj.id().into();
                self.resources
                    .insert_new(shared_obj.id(), Box::new(shared_obj));
                Ok(resp)
            }
            Err(e) => {
                error!("Could not register model");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }
    fn create_torch_model(
        &self,
        req: CreateTorchSavedModelRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        info!("Request to create PyTorch model resource");
        match torch::SavedModel::new().from_in_memory(&req.model) {
            Ok(model) => {
                info!("Created new Torch model with id: {}", model.id());

                let mut resp = CreateResourceResponse::new();
                resp.resource_id = model.id().into();
                self.resources.insert_new(model.id(), Box::new(model));

                Ok(resp)
            }
            Err(e) => {
                error!("Could not register model");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }
}
