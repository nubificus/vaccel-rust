// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use vaccel::ops::tf;
use vaccel_rpc_proto::tensorflow::{
    TensorflowModelLoadRequest, TensorflowModelLoadResponse, TensorflowModelRunRequest,
    TensorflowModelRunResponse, TensorflowModelUnloadRequest, TensorflowModelUnloadResponse,
};

impl AgentService {
    pub(crate) fn do_tensorflow_model_load(
        &self,
        req: TensorflowModelLoadRequest,
    ) -> Result<TensorflowModelLoadResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow model {}", &req.model_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!("session:{} TensorFlow model load", &req.session_id);
        let status = sess.tf_model_load(&mut res)?;

        let mut resp = TensorflowModelLoadResponse::new();
        // FIXME: Either remove this or properly return graph_def
        resp.graph_def = Vec::new();
        resp.status = Some(status.try_into()?).into();

        Ok(resp)
    }

    pub(crate) fn do_tensorflow_model_unload(
        &self,
        req: TensorflowModelUnloadRequest,
    ) -> Result<TensorflowModelUnloadResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow model {}", &req.model_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        info!("session:{} TensorFlow model unload", &req.session_id);
        let status = sess.tf_model_unload(&mut res)?;

        let mut resp = TensorflowModelUnloadResponse::new();
        resp.status = Some(status.try_into()?).into();

        Ok(resp)
    }

    pub(crate) fn do_tensorflow_model_run(
        &self,
        req: TensorflowModelRunRequest,
    ) -> Result<TensorflowModelRunResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow model {}", &req.model_id).to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown session {}", &req.session_id).to_string(),
                )
            })?;

        let run_options = req.run_options.map(tf::Buffer::new).transpose()?;

        let in_nodes = req
            .in_nodes
            .into_iter()
            .map(|e| e.try_into())
            .collect::<vaccel::Result<Vec<tf::Node>>>()?;
        let out_nodes = req
            .out_nodes
            .into_iter()
            .map(|e| e.try_into())
            .collect::<vaccel::Result<Vec<tf::Node>>>()?;

        let in_tensors = req
            .in_tensors
            .into_iter()
            .map(|e| e.try_into())
            .collect::<vaccel::Result<Vec<tf::DynTensor>>>()?;

        info!("session:{} TensorFlow model run", &req.session_id);
        let (out_tensors, status) = sess.tf_model_run(
            &mut res,
            run_options.as_ref(),
            &in_nodes,
            &in_tensors,
            &out_nodes,
        )?;

        let mut resp = TensorflowModelRunResponse::new();
        resp.out_tensors = out_tensors.into_iter().map(Into::into).collect();
        resp.status = Some(status.try_into()?).into();

        Ok(resp)
    }
}
