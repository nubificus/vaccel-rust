// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, vaccel_error, VaccelRpcAgent};
use log::debug;
use vaccel::{
    ops::{tensorflow as tf, tensorflow::InferenceArgs, InferenceModel},
    resources::TFSavedModel,
};
use vaccel_rpc_proto::tensorflow::{
    InferenceResult, TFTensor, TensorflowModelLoadRequest, TensorflowModelLoadResponse,
    TensorflowModelRunRequest, TensorflowModelRunResponse, TensorflowModelUnloadRequest,
    TensorflowModelUnloadResponse,
};

impl VaccelRpcAgent {
    pub(crate) fn do_tensorflow_model_load(
        &self,
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
            .downcast_mut::<TFSavedModel>()
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Resource {} is not a TensorFlow model", req.model_id),
                )
            })?;

        let mut resp = TensorflowModelLoadResponse::new();
        match model.load(&mut sess) {
            Ok(_) => resp.set_graph_def(Vec::new()),
            Err(e) => resp.set_error(vaccel_error(e)),
        };

        Ok(resp)
    }

    pub(crate) fn do_tensorflow_model_unload(
        &self,
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
            .downcast_mut::<TFSavedModel>()
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Resource {} is not a TensorFlow model", req.model_id),
                )
            })?;

        let mut resp = TensorflowModelUnloadResponse::new();
        match model.unload(&mut sess) {
            Ok(_) => resp.success = true,
            Err(e) => resp.error = Some(vaccel_error(e)).into(),
        };

        Ok(resp)
    }

    pub(crate) fn do_tensorflow_model_run(
        &self,
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
            .downcast_mut::<TFSavedModel>()
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Resource {} is not a TensorFlow model", req.model_id),
                )
            })?;

        let mut sess_args = InferenceArgs::new();

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

        let response = match model.run(&mut sess, &mut sess_args) {
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
}
