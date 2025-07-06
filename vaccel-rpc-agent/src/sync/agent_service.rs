// SPDX-License-Identifier: Apache-2.0

use crate::{agent_service::IntoTtrpcResult, AgentService};
#[allow(unused_imports)]
use vaccel_rpc_proto::tensorflow::{
    TensorflowLiteModelLoadRequest, TensorflowLiteModelRunRequest, TensorflowLiteModelRunResponse,
    TensorflowLiteModelUnloadRequest, TensorflowModelLoadRequest, TensorflowModelLoadResponse,
    TensorflowModelRunRequest, TensorflowModelRunResponse, TensorflowModelUnloadRequest,
    TensorflowModelUnloadResponse,
};
use vaccel_rpc_proto::{
    empty::Empty,
    genop::{GenopRequest, GenopResponse},
    image::{ImageClassificationRequest, ImageClassificationResponse},
    profiling::{ProfilingRequest, ProfilingResponse},
    resource::{
        RegisterResourceRequest, RegisterResourceResponse, SyncResourceRequest,
        SyncResourceResponse, UnregisterResourceRequest,
    },
    session::{
        CreateSessionRequest, CreateSessionResponse, DestroySessionRequest, UpdateSessionRequest,
    },
    sync::agent_ttrpc,
    torch::{TorchModelLoadRequest, TorchModelRunRequest, TorchModelRunResponse},
};

impl agent_ttrpc::AgentService for AgentService {
    fn create_session(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: CreateSessionRequest,
    ) -> ttrpc::Result<CreateSessionResponse> {
        self.do_create_session(req).into_ttrpc()
    }

    fn update_session(
        &self,
        _ctx: &::ttrpc::TtrpcContext,
        req: UpdateSessionRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_update_session(req).into_ttrpc()
    }

    fn destroy_session(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: DestroySessionRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_destroy_session(req).into_ttrpc()
    }

    fn image_classification(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: ImageClassificationRequest,
    ) -> ttrpc::Result<ImageClassificationResponse> {
        self.do_image_classification(req).into_ttrpc()
    }

    fn register_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: RegisterResourceRequest,
    ) -> ttrpc::Result<RegisterResourceResponse> {
        self.do_register_resource(req).into_ttrpc()
    }

    fn sync_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: SyncResourceRequest,
    ) -> ttrpc::Result<SyncResourceResponse> {
        self.do_sync_resource(req).into_ttrpc()
    }

    fn unregister_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: UnregisterResourceRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_unregister_resource(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_load(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowModelLoadRequest,
    ) -> ttrpc::Result<TensorflowModelLoadResponse> {
        self.do_tensorflow_model_load(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_unload(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowModelUnloadRequest,
    ) -> ttrpc::Result<TensorflowModelUnloadResponse> {
        self.do_tensorflow_model_unload(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_run(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowModelRunRequest,
    ) -> ttrpc::Result<TensorflowModelRunResponse> {
        self.do_tensorflow_model_run(req).into_ttrpc()
    }

    fn tensorflow_lite_model_load(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowLiteModelLoadRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_tflite_model_load(req).into_ttrpc()
    }

    fn tensorflow_lite_model_unload(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowLiteModelUnloadRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_tflite_model_unload(req).into_ttrpc()
    }

    fn tensorflow_lite_model_run(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TensorflowLiteModelRunRequest,
    ) -> ttrpc::Result<TensorflowLiteModelRunResponse> {
        self.do_tflite_model_run(req).into_ttrpc()
    }

    fn torch_model_load(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TorchModelLoadRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_torch_model_load(req).into_ttrpc()
    }

    fn torch_model_run(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TorchModelRunRequest,
    ) -> ttrpc::Result<TorchModelRunResponse> {
        self.do_torch_model_run(req).into_ttrpc()
    }

    fn genop(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: GenopRequest,
    ) -> ttrpc::Result<GenopResponse> {
        self.do_genop(req).into_ttrpc()
    }

    fn get_profiler(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: ProfilingRequest,
    ) -> ttrpc::Result<ProfilingResponse> {
        self.do_get_profiler(req).into_ttrpc()
    }
}
