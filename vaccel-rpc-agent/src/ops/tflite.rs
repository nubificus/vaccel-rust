// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, vaccel_error, VaccelRpcAgent};
use log::debug;
use vaccel::ops::{tensorflow::lite as tflite, ModelInitialize, ModelLoadUnload, ModelRun};
use vaccel_rpc_proto::tensorflow::{
    InferenceLiteResult, TFLiteTensor, TensorflowLiteModelLoadRequest,
    TensorflowLiteModelLoadResponse, TensorflowLiteModelRunRequest, TensorflowLiteModelRunResponse,
    TensorflowLiteModelUnloadRequest, TensorflowLiteModelUnloadResponse,
};

impl VaccelRpcAgent {
    pub(crate) fn do_tflite_model_load(
        &self,
        req: TensorflowLiteModelLoadRequest,
    ) -> ttrpc::Result<TensorflowLiteModelLoadResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown TensorFlow Lite model".to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let mut resp = TensorflowLiteModelLoadResponse::new();
        let mut model = tflite::Model::new(res.as_mut());
        if let Err(e) = model.as_mut().load(&mut sess) {
            resp.set_error(vaccel_error(e));
        };

        Ok(resp)
    }

    pub(crate) fn do_tflite_model_unload(
        &self,
        req: TensorflowLiteModelUnloadRequest,
    ) -> ttrpc::Result<TensorflowLiteModelUnloadResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown TensorFlow Lite model".to_string(),
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

        let mut resp = TensorflowLiteModelUnloadResponse::new();
        let mut model = tflite::Model::new(res.as_mut());
        if let Err(e) = model.as_mut().unload(&mut sess) {
            resp.set_error(vaccel_error(e));
        };

        Ok(resp)
    }

    pub(crate) fn do_tflite_model_run(
        &self,
        req: TensorflowLiteModelRunRequest,
    ) -> ttrpc::Result<TensorflowLiteModelRunResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown TensorFlow Lite model".to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let mut sess_args = tflite::InferenceArgs::new();

        let in_tensors = req.in_tensors;
        for tensor in in_tensors.iter() {
            debug!("tensor.dim: {:?}", tensor.dims);
            sess_args.add_input(tensor);
        }

        sess_args.set_nr_outputs(req.nr_outputs);
        let num_outputs: usize = req.nr_outputs.try_into().unwrap();

        let mut model = tflite::Model::new(res.as_mut());
        let response = match model.as_mut().run(&mut sess, &mut sess_args) {
            Ok(result) => {
                let mut inference = InferenceLiteResult::new();
                let mut out_tensors: Vec<TFLiteTensor> = Vec::with_capacity(num_outputs);
                for i in 0..num_outputs {
                    out_tensors.push(result.get_grpc_output(i).unwrap());
                }
                inference.out_tensors = out_tensors;
                let mut resp = TensorflowLiteModelRunResponse::new();
                resp.set_result(inference);
                resp
            }
            Err(e) => {
                let mut resp = TensorflowLiteModelRunResponse::new();
                resp.set_error(vaccel_error(e));
                resp
            }
        };

        Ok(response)
    }
}
