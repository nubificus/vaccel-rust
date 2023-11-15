use crate::{ttrpc_error, vaccel_error, Agent};
use log::{error, info};
use protocols::{
    resources::{CreateResourceResponse, CreateTorchSavedModelRequest},
    torch::{
        TorchJitloadForwardRequest, TorchJitloadForwardResponse, TorchJitloadForwardResult,
        TorchTensor,
    },
};
use vaccel::torch;

impl Agent {
    pub(crate) fn do_torch_jitload_forward(
        &self,
        req: TorchJitloadForwardRequest,
    ) -> ttrpc::Result<TorchJitloadForwardResponse> {
        let mut resource = self
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

        let model = resource
            .as_mut_any()
            .downcast_mut::<torch::SavedModel>()
            .ok_or_else(|| {
                ttrpc_error(
                    ttrpc::Code::INVALID_ARGUMENT,
                    format!("Resource {} is not a pytorch model", req.model_id),
                )
            })?;

        // origin: vaccel::ops::inference...
        let mut sess_args = torch::TorchArgs::new();
        let mut jitload = torch::TorchJitLoadForward::new();

        let run_options = torch::Buffer::new(req.run_options.as_slice());
        sess_args.set_run_options(&run_options);

        let in_tensors = req.in_tensors;
        for tensor in in_tensors.iter() {
            sess_args.add_input(tensor);
        }

        // TODO: bindings examples
        /*
           let response = jitload.jitload_forward(&mut sess, &mut sess_args, &mut model)?;
           match response.get_output::<f32>(0) {
           Ok(result) => {
           println!("Success");
           println!(
           "Output tensor => type:{:?} nr_dims:{}",
           result.data_type(),
           result.nr_dims()
           );
           for i in 0..result.nr_dims() {
           println!("dim[{}]: {}", i, result.dim(i as usize).unwrap());
           }
           }
        // Err(e) => println!("Torch JitLoadForward failed: '{}'", e),
        }
        Ok(TorchJitloadForwardResponse {
        result: Some(response),
        ..Default::default()
        })
        */

        // TODO
        // let num_outputs = in_tensors.len();
        let num_outputs: usize = 1;

        //println!("NUM of output: {}, Type: {}", num_outputs, type_of(&num_outputs));
        let response = match jitload.jitload_forward(&mut sess, &mut sess_args, model) {
            Ok(result) => {
                let mut jitload_forward = TorchJitloadForwardResult::new();
                let mut out_tensors: Vec<TorchTensor> = Vec::with_capacity(num_outputs);
                for i in 0..num_outputs {
                    out_tensors.push(result.get_grpc_output(i).unwrap());
                }
                jitload_forward.out_tensors = out_tensors;
                let mut resp = TorchJitloadForwardResponse::new();
                resp.set_result(jitload_forward);
                resp
            }
            Err(e) => {
                let mut resp = TorchJitloadForwardResponse::new();
                resp.set_error(vaccel_error(e));
                resp
            }
        };

        Ok(response)
    }

    pub(crate) fn create_torch_model(
        &self,
        req: CreateTorchSavedModelRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        info!("Request to create PyTorch model resource");
        match torch::SavedModel::new().from_in_memory(&req.model) {
            Ok(model) => {
                info!("Created new Torch model with id: {}", model.id());

                let mut resp = CreateResourceResponse::new();
                resp.resource_id = model.id().into();
                let e = self.resources.insert(model.id(), Box::new(model));
                assert!(e.is_none());

                Ok(resp)
            }
            Err(e) => {
                error!("Could not register model");
                Err(ttrpc_error(ttrpc::Code::INTERNAL, e.to_string()))
            }
        }
    }
}
