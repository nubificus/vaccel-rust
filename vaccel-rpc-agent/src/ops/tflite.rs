// SPDX-License-Identifier: Apache-2.0

use crate::agent_service::{AgentService, AgentServiceError, Result};
use log::info;
use std::num::TryFromIntError;
use vaccel::ops::tf::lite as tflite;
use vaccel_rpc_proto::{
    empty::Empty,
    tensorflow::{
        TensorflowLiteModelLoadRequest, TensorflowLiteModelRunRequest,
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

        info!("session:{} TensorFlow Lite model load", &req.session_id);
        sess.tflite_model_load(&mut res)?;

        Ok(Empty::new())
    }

    pub(crate) fn do_tflite_model_unload(
        &self,
        req: TensorflowLiteModelUnloadRequest,
    ) -> Result<Empty> {
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

        info!("session:{} TensorFlow Lite model unload", &req.session_id);
        sess.tflite_model_unload(&mut res)?;

        Ok(Empty::new())
    }

    pub(crate) fn do_tflite_model_run(
        &self,
        req: TensorflowLiteModelRunRequest,
    ) -> Result<TensorflowLiteModelRunResponse> {
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

        let in_tensors = req
            .in_tensors
            .into_iter()
            .map(|e| e.try_into())
            .collect::<vaccel::Result<Vec<tflite::DynTensor>>>()?;

        let nr_out_tensors = req
            .nr_out_tensors
            .try_into()
            .map_err(|e: TryFromIntError| {
                AgentServiceError::Internal(
                    format!("Could not convert `nr_out_tensors` to `usize`: {}", e).to_string(),
                )
            })?;

        info!("session:{} TensorFlow Lite model run", &req.session_id);
        let (out_tensors, status) = sess.tflite_model_run(&mut res, &in_tensors, nr_out_tensors)?;

        let mut resp = TensorflowLiteModelRunResponse::new();
        resp.out_tensors = out_tensors.into_iter().map(Into::into).collect();
        resp.status = Some(status.into()).into();

        Ok(resp)
    }
}
