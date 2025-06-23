// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use std::num::TryFromIntError;
use vaccel::ops::ModelLoadUnload;
use vaccel::ops::{torch, ModelInitialize, ModelRun};
use vaccel_rpc_proto::{
    empty::Empty,
    torch::{TorchModelLoadRequest, TorchModelRunRequest, TorchModelRunResponse, TorchTensor},
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
        let mut model = torch::Model::new(res.as_mut());
        model.load(&mut sess)?;

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

        let mut sess_args = torch::InferenceArgs::new();

        let run_options = req.run_options.map(torch::Buffer::new).transpose()?;
        sess_args.set_run_options(run_options.as_ref());

        let in_tensors = req.in_tensors;
        for tensor in in_tensors.iter() {
            sess_args.add_input(tensor)?;
        }

        sess_args.set_nr_outputs(req.nr_outputs);
        let num_outputs: usize = req.nr_outputs.try_into().map_err(|e: TryFromIntError| {
            AgentServiceError::Internal(
                format!("Could not convert `nr_outputs` to usize: {}", e).to_string(),
            )
        })?;

        info!("session:{} PyTorch model run", sess.id());
        let mut model = torch::Model::new(res.as_mut());
        let result = model.run(&mut sess, &mut sess_args)?;

        let mut out_tensors: Vec<TorchTensor> = Vec::with_capacity(num_outputs);
        for i in 0..num_outputs {
            out_tensors.push(result.to_grpc_output(i)?);
        }

        let mut resp = TorchModelRunResponse::new();
        resp.out_tensors = out_tensors;

        Ok(resp)
    }
}
