// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, vaccel_error, AgentService};
use vaccel::ops::{torch, ModelInitialize, ModelRun};
use vaccel_rpc_proto::torch::{
    TorchJitloadForwardRequest, TorchJitloadForwardResponse, TorchJitloadForwardResult, TorchTensor,
};

impl AgentService {
    pub(crate) fn do_torch_jitload_forward(
        &self,
        req: TorchJitloadForwardRequest,
    ) -> ttrpc::Result<TorchJitloadForwardResponse> {
        let mut res = self
            .resources
            .get_mut(&req.model_id.into())
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    "Unknown PyTorch model".to_string(),
                )
            })?;

        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let mut resp = TorchJitloadForwardResponse::new();

        let mut sess_args = torch::InferenceArgs::new();

        let run_options = match torch::Buffer::new(req.run_options.as_slice()) {
            Ok(b) => b,
            Err(e) => {
                resp.set_error(vaccel_error(e));
                return Ok(resp);
            }
        };
        sess_args.set_run_options(&run_options);

        let in_tensors = req.in_tensors;
        for tensor in in_tensors.iter() {
            if let Err(e) = sess_args.add_input(tensor) {
                resp.set_error(vaccel_error(e));
                return Ok(resp);
            };
        }

        sess_args.set_nr_outputs(req.nr_outputs);
        let num_outputs: usize = req.nr_outputs.try_into().unwrap();

        let mut model = torch::Model::new(res.as_mut());
        match model.as_mut().run(&mut sess, &mut sess_args) {
            Ok(result) => {
                let mut jitload_forward = TorchJitloadForwardResult::new();
                let mut out_tensors: Vec<TorchTensor> = Vec::with_capacity(num_outputs);
                for i in 0..num_outputs {
                    out_tensors.push(result.get_grpc_output(i).unwrap());
                }
                jitload_forward.out_tensors = out_tensors;
                resp.set_result(jitload_forward);
            }
            Err(e) => resp.set_error(vaccel_error(e)),
        };

        Ok(resp)
    }
}
