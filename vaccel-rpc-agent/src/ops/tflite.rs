// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::{debug, info};
use std::num::TryFromIntError;
use vaccel::ops::{tensorflow::lite as tflite, ModelInitialize, ModelLoadUnload, ModelRun};
use vaccel_rpc_proto::{
    empty::Empty,
    tensorflow::{
        TFLiteTensor, TensorflowLiteModelLoadRequest, TensorflowLiteModelRunRequest,
        TensorflowLiteModelRunResponse, TensorflowLiteModelUnloadRequest,
    },
};

impl AgentService {
    pub(crate) fn do_tflite_model_load(
        &self,
        req: TensorflowLiteModelLoadRequest,
    ) -> Result<Empty> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow Lite model {}", &req.model_id).to_string(),
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

        info!("session:{} TensorFlow Lite model load", sess.id());
        let mut model = tflite::Model::new(res.as_mut());
        model.as_mut().load(&mut sess)?;

        Ok(Empty::new())
    }

    pub(crate) fn do_tflite_model_unload(
        &self,
        req: TensorflowLiteModelUnloadRequest,
    ) -> Result<Empty> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow Lite model {}", &req.model_id).to_string(),
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

        info!("session:{} TensorFlow Lite model unload", sess.id());
        let mut model = tflite::Model::new(res.as_mut());
        model.as_mut().unload(&mut sess)?;

        Ok(Empty::new())
    }

    pub(crate) fn do_tflite_model_run(
        &self,
        req: TensorflowLiteModelRunRequest,
    ) -> Result<TensorflowLiteModelRunResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow Lite model {}", &req.model_id).to_string(),
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

        let mut sess_args = tflite::InferenceArgs::new();

        let in_tensors = req.in_tensors;
        for tensor in in_tensors.iter() {
            debug!("tensor.dim: {:?}", tensor.dims);
            sess_args.add_input(tensor)?;
        }

        sess_args.set_nr_outputs(req.nr_outputs);
        let num_outputs: usize = req.nr_outputs.try_into().map_err(|e: TryFromIntError| {
            AgentServiceError::Internal(
                format!("Could not convert `nr_outputs` to usize: {}", e).to_string(),
            )
        })?;

        info!("session:{} TensorFlow Lite model run", sess.id());
        let mut model = tflite::Model::new(res.as_mut());
        let result = model.as_mut().run(&mut sess, &mut sess_args)?;

        let mut out_tensors: Vec<TFLiteTensor> = Vec::with_capacity(num_outputs);
        for i in 0..num_outputs {
            out_tensors.push(result.to_grpc_output(i)?);
        }

        let mut resp = TensorflowLiteModelRunResponse::new();
        resp.out_tensors = out_tensors;
        resp.status = Some(result.status.into()).into();

        Ok(resp)
    }
}
