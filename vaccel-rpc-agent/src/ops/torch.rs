// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use std::num::TryFromIntError;
use vaccel::ops::torch;
use vaccel_rpc_proto::{
    empty::Empty,
    torch::{TorchModelLoadRequest, TorchModelRunRequest, TorchModelRunResponse},
};

impl AgentService {
    pub(crate) fn do_torch_model_load(&self, req: TorchModelLoadRequest) -> Result<Empty> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown PyTorch model {}", &req.model_id).to_string(),
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

        info!("session:{} PyTorch model load", sess.id());
        sess.torch_model_load(&mut res)?;

        Ok(Empty::new())
    }

    pub(crate) fn do_torch_model_run(
        &self,
        req: TorchModelRunRequest,
    ) -> Result<TorchModelRunResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown PyTorch model {}", &req.model_id).to_string(),
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

        let run_options = req.run_options.map(torch::Buffer::new).transpose()?;

        let in_tensors = req
            .in_tensors
            .into_iter()
            .map(|e| e.try_into())
            .collect::<vaccel::Result<Vec<torch::DynTensor>>>()?;

        let nr_out_tensors = req
            .nr_out_tensors
            .try_into()
            .map_err(|e: TryFromIntError| {
                AgentServiceError::Internal(
                    format!("Could not convert `nr_out_tensors` to `usize`: {}", e).to_string(),
                )
            })?;

        info!("session:{} PyTorch model run", sess.id());
        let out_tensors =
            sess.torch_model_run(&mut res, run_options.as_ref(), &in_tensors, nr_out_tensors)?;

        let mut resp = TorchModelRunResponse::new();
        resp.out_tensors = out_tensors.into_iter().map(Into::into).collect();

        Ok(resp)
    }
}
