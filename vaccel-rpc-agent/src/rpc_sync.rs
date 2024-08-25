// SPDX-License-Identifier: Apache-2.0

use crate::VaccelRpcAgent;
#[allow(unused_imports)]
use vaccel_rpc_proto::tensorflow::{
    TensorflowLiteModelLoadRequest, TensorflowLiteModelLoadResponse, TensorflowLiteModelRunRequest,
    TensorflowLiteModelRunResponse, TensorflowLiteModelUnloadRequest,
    TensorflowLiteModelUnloadResponse, TensorflowModelLoadRequest, TensorflowModelLoadResponse,
    TensorflowModelRunRequest, TensorflowModelRunResponse, TensorflowModelUnloadRequest,
    TensorflowModelUnloadResponse,
};
use vaccel_rpc_proto::{
    genop::{GenopRequest, GenopResponse},
    image::{ImageClassificationRequest, ImageClassificationResponse},
    profiling::{ProfilingRequest, ProfilingResponse},
    resources::{
        CreateResourceRequest, CreateResourceResponse, DestroyResourceRequest,
        RegisterResourceRequest, UnregisterResourceRequest,
    },
    session::{
        CreateSessionRequest, CreateSessionResponse, DestroySessionRequest, UpdateSessionRequest,
    },
    sync::{agent::VaccelEmpty, agent_ttrpc::RpcAgent},
    torch::{TorchJitloadForwardRequest, TorchJitloadForwardResponse},
};

impl RpcAgent for VaccelRpcAgent {
    fn create_session(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: CreateSessionRequest,
    ) -> ttrpc::Result<CreateSessionResponse> {
        self.do_create_session(req)
    }

    fn update_session(
        &self,
        _ctx: &::ttrpc::TtrpcContext,
        req: UpdateSessionRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        self.do_update_session(req)
    }

    fn destroy_session(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: DestroySessionRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        self.do_destroy_session(req)
    }

    fn image_classification(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: ImageClassificationRequest,
    ) -> ttrpc::Result<ImageClassificationResponse> {
        self.do_image_classification(req)
    }

    fn create_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: CreateResourceRequest,
    ) -> ttrpc::Result<CreateResourceResponse> {
        self.do_create_resource(req)
    }

    fn destroy_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: DestroyResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        self.do_destroy_resource(req)
    }

    fn register_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: RegisterResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        self.do_register_resource(req)
    }

    fn unregister_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: UnregisterResourceRequest,
    ) -> ttrpc::Result<VaccelEmpty> {
        self.do_unregister_resource(req)
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_load(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowModelLoadRequest,
    ) -> ttrpc::Result<TensorflowModelLoadResponse> {
        self.do_tensorflow_model_load(req)
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_unload(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowModelUnloadRequest,
    ) -> ttrpc::Result<TensorflowModelUnloadResponse> {
        self.do_tensorflow_model_unload(req)
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_run(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowModelRunRequest,
    ) -> ttrpc::Result<TensorflowModelRunResponse> {
        self.do_tensorflow_model_run(req)
    }

    fn tensorflow_lite_model_load(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowLiteModelLoadRequest,
    ) -> ttrpc::Result<TensorflowLiteModelLoadResponse> {
        self.do_tflite_model_load(req)
    }

    fn tensorflow_lite_model_unload(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowLiteModelUnloadRequest,
    ) -> ttrpc::Result<TensorflowLiteModelUnloadResponse> {
        self.do_tflite_model_unload(req)
    }

    fn tensorflow_lite_model_run(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowLiteModelRunRequest,
    ) -> ttrpc::Result<TensorflowLiteModelRunResponse> {
        self.do_tflite_model_run(req)
    }

    fn torch_jitload_forward(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TorchJitloadForwardRequest,
    ) -> ttrpc::Result<TorchJitloadForwardResponse> {
        self.do_torch_jitload_forward(req)
    }

    fn genop(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: GenopRequest,
    ) -> ttrpc::Result<GenopResponse> {
        self.do_genop(req)
    }

    fn get_timers(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: ProfilingRequest,
    ) -> ttrpc::Result<ProfilingResponse> {
        self.do_get_timers(req)
    }
}
