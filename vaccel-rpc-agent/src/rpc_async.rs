// SPDX-License-Identifier: Apache-2.0

use crate::VaccelRpcAgent;
use async_trait::async_trait;
use std::default::Default;
#[allow(unused_imports)]
use vaccel_rpc_proto::tensorflow::{
    TensorflowLiteModelLoadRequest, TensorflowLiteModelLoadResponse, TensorflowLiteModelRunRequest,
    TensorflowLiteModelRunResponse, TensorflowLiteModelUnloadRequest,
    TensorflowLiteModelUnloadResponse, TensorflowModelLoadRequest, TensorflowModelLoadResponse,
    TensorflowModelRunRequest, TensorflowModelRunResponse, TensorflowModelUnloadRequest,
    TensorflowModelUnloadResponse,
};
use vaccel_rpc_proto::{
    asynchronous::{agent::VaccelEmpty, agent_ttrpc::RpcAgent},
    genop::{GenopArg, GenopRequest, GenopResponse},
    image::{ImageClassificationRequest, ImageClassificationResponse},
    profiling::{ProfilingRequest, ProfilingResponse},
    resource::{RegisterResourceRequest, RegisterResourceResponse, UnregisterResourceRequest},
    session::{
        CreateSessionRequest, CreateSessionResponse, DestroySessionRequest, UpdateSessionRequest,
    },
    torch::{TorchJitloadForwardRequest, TorchJitloadForwardResponse},
};
//use tracing::{info, instrument, Instrument};
use log::debug;

#[async_trait]
impl RpcAgent for VaccelRpcAgent {
    async fn create_session(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: CreateSessionRequest,
    ) -> ttrpc::Result<CreateSessionResponse> {
        self.do_create_session(req)
    }

    async fn update_session(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: UpdateSessionRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        self.do_update_session(req)
    }

    async fn destroy_session(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: DestroySessionRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        self.do_destroy_session(req)
    }

    async fn image_classification(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: ImageClassificationRequest,
    ) -> ttrpc::Result<ImageClassificationResponse> {
        self.do_image_classification(req)
    }

    async fn register_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: RegisterResourceRequest,
    ) -> ttrpc::Result<RegisterResourceResponse> {
        self.do_register_resource(req)
    }

    async fn unregister_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: UnregisterResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        self.do_unregister_resource(req)
    }

    #[cfg(target_pointer_width = "64")]
    async fn tensorflow_model_load(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowModelLoadRequest,
    ) -> ttrpc::Result<TensorflowModelLoadResponse> {
        self.do_tensorflow_model_load(req)
    }

    #[cfg(target_pointer_width = "64")]
    async fn tensorflow_model_unload(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowModelUnloadRequest,
    ) -> ttrpc::Result<TensorflowModelUnloadResponse> {
        self.do_tensorflow_model_unload(req)
    }

    #[cfg(target_pointer_width = "64")]
    async fn tensorflow_model_run(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowModelRunRequest,
    ) -> ttrpc::Result<TensorflowModelRunResponse> {
        self.do_tensorflow_model_run(req)
    }

    async fn tensorflow_lite_model_load(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowLiteModelLoadRequest,
    ) -> ttrpc::Result<TensorflowLiteModelLoadResponse> {
        self.do_tflite_model_load(req)
    }

    async fn tensorflow_lite_model_unload(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowLiteModelUnloadRequest,
    ) -> ttrpc::Result<TensorflowLiteModelUnloadResponse> {
        self.do_tflite_model_unload(req)
    }

    async fn tensorflow_lite_model_run(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TensorflowLiteModelRunRequest,
    ) -> ttrpc::Result<TensorflowLiteModelRunResponse> {
        self.do_tflite_model_run(req)
    }

    async fn torch_jitload_forward(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TorchJitloadForwardRequest,
    ) -> ttrpc::Result<TorchJitloadForwardResponse> {
        self.do_torch_jitload_forward(req)
    }

    async fn genop(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: GenopRequest,
    ) -> ttrpc::Result<GenopResponse> {
        self.do_genop(req)
    }

    async fn genop_stream(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        mut r: ::ttrpc::asynchronous::ServerStreamReceiver<GenopRequest>,
    ) -> ttrpc::Result<GenopResponse> {
        let mut req = GenopRequest::default();
        let mut r_arg = vec![GenopArg::new()];
        let mut w_arg = vec![GenopArg::new()];
        while let Some(mut data) = r.recv().await? {
            req.session_id = data.session_id;
            if data.read_args.len() == 1 && data.read_args[0].parts > 0 {
                if data.read_args[0].part_no < data.read_args[0].parts {
                    r_arg[0].buf.append(&mut data.read_args[0].buf);
                } else {
                    r_arg[0].buf.append(&mut data.read_args[0].buf);
                    r_arg[0].size = data.read_args[0].size;
                    req.read_args.append(&mut r_arg);
                    r_arg = vec![GenopArg::new()];
                }
            } else if data.write_args.len() == 1 && data.write_args[0].parts > 0 {
                if data.write_args[0].part_no < data.write_args[0].parts {
                    w_arg[0].buf.append(&mut data.write_args[0].buf);
                } else {
                    w_arg[0].buf.append(&mut data.write_args[0].buf);
                    w_arg[0].size = data.write_args[0].size;
                    req.write_args.append(&mut w_arg);
                    w_arg = vec![GenopArg::new()];
                }
            } else {
                req.read_args.append(&mut data.read_args);
                req.write_args.append(&mut data.write_args);
            }
        }

        debug!("Genop is streaming");
        self.do_genop(req)
    }

    async fn get_timers(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: ProfilingRequest,
    ) -> ttrpc::Result<ProfilingResponse> {
        self.do_get_timers(req)
    }
}
