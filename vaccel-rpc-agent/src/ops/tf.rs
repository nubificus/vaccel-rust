// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::{debug, info};
use vaccel::ops::{tensorflow as tf, ModelInitialize, ModelLoadUnload, ModelRun};
use vaccel_rpc_proto::tensorflow::{
    TFTensor, TensorflowModelLoadRequest, TensorflowModelLoadResponse, TensorflowModelRunRequest,
    TensorflowModelRunResponse, TensorflowModelUnloadRequest, TensorflowModelUnloadResponse,
};

impl AgentService {
    pub(crate) fn do_tensorflow_model_load(
        &self,
        req: TensorflowModelLoadRequest,
    ) -> Result<TensorflowModelLoadResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow model {}", &req.model_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!("session:{} TensorFlow model load", sess.id());
        let mut model = tf::Model::new(res.as_mut());
        let status = model.as_mut().load(&mut sess)?;

        let mut resp = TensorflowModelLoadResponse::new();
        // FIXME: Either remove this or properly return graph_def
        resp.graph_def = Vec::new();
        resp.status = Some(status.into()).into();

        Ok(resp)
    }

    pub(crate) fn do_tensorflow_model_unload(
        &self,
        req: TensorflowModelUnloadRequest,
    ) -> Result<TensorflowModelUnloadResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow model {}", &req.model_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!("session:{} TensorFlow model unload", sess.id());
        let mut model = tf::Model::new(res.as_mut());
        let status = model.as_mut().unload(&mut sess)?;

        let mut resp = TensorflowModelUnloadResponse::new();
        resp.status = Some(status.into()).into();

        Ok(resp)
    }

    pub(crate) fn do_tensorflow_model_run(
        &self,
        req: TensorflowModelRunRequest,
    ) -> Result<TensorflowModelRunResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow model {}", &req.model_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        let mut sess_args = tf::InferenceArgs::new();

        let run_options = tf::Buffer::new(req.run_options.as_slice())?;
        sess_args.set_run_options(&run_options);

        let in_nodes: Vec<tf::Node> = req
            .in_nodes
            .iter()
            .map(|e| e.try_into())
            .collect::<vaccel::Result<Vec<tf::Node>>>()?;
        let in_tensors = req.in_tensors;
        for it in in_nodes.iter().zip(in_tensors.iter()) {
            let (node, tensor) = it;
            debug!("tensor.dim: {:?}", tensor.dims);
            sess_args.add_input(node, tensor)?;
        }

        let out_nodes: Vec<tf::Node> = req
            .out_nodes
            .iter()
            .map(|e| e.try_into())
            .collect::<vaccel::Result<Vec<tf::Node>>>()?;
        let num_outputs = out_nodes.len();
        for output in out_nodes.iter() {
            sess_args.request_output(output);
        }

        info!("session:{} TensorFlow model run", sess.id());
        let mut model = tf::Model::new(res.as_mut());
        let result = model.as_mut().run(&mut sess, &mut sess_args)?;

        let mut out_tensors: Vec<TFTensor> = Vec::with_capacity(num_outputs);
        for i in 0..num_outputs {
            out_tensors.push(result.get_grpc_output(i)?);
        }

        let mut resp = TensorflowModelRunResponse::new();
        resp.out_tensors = out_tensors;
        resp.status = Some(result.status.into()).into();

        Ok(resp)
    }
}
