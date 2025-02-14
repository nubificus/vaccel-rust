// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, vaccel_error, AgentService};
use log::debug;
use vaccel::ops::{tensorflow as tf, ModelInitialize, ModelLoadUnload, ModelRun};
use vaccel_rpc_proto::tensorflow::{
    InferenceResult as ProtoInferenceResult, TFTensor, TensorflowModelLoadRequest,
    TensorflowModelLoadResponse, TensorflowModelRunRequest, TensorflowModelRunResponse,
    TensorflowModelUnloadRequest, TensorflowModelUnloadResponse,
};

impl AgentService {
    pub(crate) fn do_tensorflow_model_load(
        &self,
        req: TensorflowModelLoadRequest,
    ) -> ttrpc::Result<TensorflowModelLoadResponse> {
        let mut res = self
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

        let mut resp = TensorflowModelLoadResponse::new();
        let mut model = tf::Model::new(res.as_mut());
        match model.as_mut().load(&mut sess) {
            Ok(_) => resp.set_graph_def(Vec::new()),
            Err(e) => resp.set_error(vaccel_error(e)),
        };

        Ok(resp)
    }

    pub(crate) fn do_tensorflow_model_unload(
        &self,
        req: TensorflowModelUnloadRequest,
    ) -> ttrpc::Result<TensorflowModelUnloadResponse> {
        let mut res = self
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

        let mut resp = TensorflowModelUnloadResponse::new();
        let mut model = tf::Model::new(res.as_mut());
        match model.as_mut().unload(&mut sess) {
            Ok(_) => resp.success = true,
            Err(e) => resp.error = Some(vaccel_error(e)).into(),
        };

        Ok(resp)
    }

    pub(crate) fn do_tensorflow_model_run(
        &self,
        req: TensorflowModelRunRequest,
    ) -> ttrpc::Result<TensorflowModelRunResponse> {
        let mut res = self
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

        let mut resp = TensorflowModelRunResponse::new();

        let mut sess_args = tf::InferenceArgs::new();

        let run_options = match tf::Buffer::new(req.run_options.as_slice()) {
            Ok(b) => b,
            Err(e) => {
                resp.set_error(vaccel_error(e));
                return Ok(resp);
            }
        };
        sess_args.set_run_options(&run_options);

        let in_nodes: Vec<tf::Node> = match req.in_nodes.iter().map(|e| e.try_into()).collect() {
            Ok(n) => n,
            Err(e) => {
                resp.set_error(vaccel_error(e));
                return Ok(resp);
            }
        };
        let in_tensors = req.in_tensors;
        for it in in_nodes.iter().zip(in_tensors.iter()) {
            let (node, tensor) = it;
            debug!("tensor.dim: {:?}", tensor.dims);
            if let Err(e) = sess_args.add_input(node, tensor) {
                resp.set_error(vaccel_error(e));
                return Ok(resp);
            };
        }

        let out_nodes: Vec<tf::Node> = match req.out_nodes.iter().map(|e| e.try_into()).collect() {
            Ok(n) => n,
            Err(e) => {
                resp.set_error(vaccel_error(e));
                return Ok(resp);
            }
        };
        let num_outputs = out_nodes.len();
        for output in out_nodes.iter() {
            sess_args.request_output(output);
        }

        let mut model = tf::Model::new(res.as_mut());
        match model.as_mut().run(&mut sess, &mut sess_args) {
            Ok(result) => {
                let mut inference = ProtoInferenceResult::new();
                let mut out_tensors: Vec<TFTensor> = Vec::with_capacity(num_outputs);
                for i in 0..num_outputs {
                    out_tensors.push(result.get_grpc_output(i).unwrap());
                }
                inference.out_tensors = out_tensors;
                resp.set_result(inference);
            }
            Err(e) => resp.set_error(vaccel_error(e)),
        };

        Ok(resp)
    }
}
