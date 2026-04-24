// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use std::num::TryFromIntError;
use vaccel::{ops::tf::lite::DynTensor, profiling::SessionProfiler};
use vaccel_rpc_proto::{
    empty::Empty,
    tflite::{ModelLoadRequest, ModelRunRequest, ModelRunResponse, ModelUnloadRequest},
};

impl AgentService {
    pub(crate) fn do_tflite_model_load(&self, req: ModelLoadRequest) -> Result<Empty> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow Lite model {}", &req.model_id).to_string(),
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
        let sess_id = sess.id().ok_or(AgentServiceError::Internal(
            "Invalid session ID".to_string(),
        ))?;

        info!("session:{} TensorFlow Lite model load", sess_id);
        self.profile_fn(
            sess_id,
            "tflite_model_load > sess.tflite_model_load",
            || sess.tflite_model_load(&mut res),
        )?;

        Ok(Empty::new())
    }

    pub(crate) fn do_tflite_model_unload(&self, req: ModelUnloadRequest) -> Result<Empty> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow Lite model {}", &req.model_id).to_string(),
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
        let sess_id = sess.id().ok_or(AgentServiceError::Internal(
            "Invalid session ID".to_string(),
        ))?;

        info!("session:{} TensorFlow Lite model unload", sess_id);
        self.profile_fn(
            sess_id,
            "tflite_model_unload > sess.tflite_model_unload",
            || sess.tflite_model_unload(&mut res),
        )?;

        Ok(Empty::new())
    }

    pub(crate) fn do_tflite_model_run(&self, req: ModelRunRequest) -> Result<ModelRunResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.try_into()?)
            .ok_or_else(|| {
                AgentServiceError::NotFound(
                    format!("Unknown TensorFlow Lite model {}", &req.model_id).to_string(),
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
        let sess_id = sess.id().ok_or(AgentServiceError::Internal(
            "Invalid session ID".to_string(),
        ))?;

        let in_tensors = self.profile_fn(sess_id, "tflite_model_run > in_tensors", || {
            req.in_tensors
                .into_iter()
                .map(|e| e.try_into())
                .collect::<vaccel::Result<Vec<DynTensor>>>()
        })?;

        let nr_out_tensors = req
            .nr_out_tensors
            .try_into()
            .map_err(|e: TryFromIntError| {
                AgentServiceError::Internal(
                    format!("Could not convert `nr_out_tensors` to `usize`: {}", e).to_string(),
                )
            })?;

        info!("session:{} TensorFlow Lite model run", sess_id);
        let (out_tensors, status) =
            self.profile_fn(sess_id, "tflite_model_run > sess.tflite_model_run", || {
                sess.tflite_model_run(&mut res, &in_tensors, nr_out_tensors)
            })?;

        let mut resp = ModelRunResponse::new();
        resp.out_tensors = self.profile_fn(sess_id, "tflite_model_run > resp_out_tensors", || {
            out_tensors.into_iter().map(Into::into).collect()
        });
        resp.status = Some(status.into()).into();

        Ok(resp)
    }
}
