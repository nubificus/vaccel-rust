// SPDX-License-Identifier: Apache-2.0

use crate::{agent_service::IntoTtrpcResult, AgentService};
#[allow(unused_imports)]
use vaccel_rpc_proto::tf::{
    ModelLoadRequest as TFModelLoadRequest, ModelLoadResponse as TFModelLoadResponse,
    ModelRunRequest as TFModelRunRequest, ModelRunResponse as TFModelRunResponse,
    ModelUnloadRequest as TFModelUnloadRequest, ModelUnloadResponse as TFModelUnloadResponse,
};
use vaccel_rpc_proto::{
    empty::Empty,
    genop::{Request as GenopRequest, Response as GenopResponse},
    image::{Request as ImageRequest, Response as ImageResponse},
    profiling::{Request as ProfilingRequest, Response as ProfilingResponse},
    resource::{RegisterRequest, RegisterResponse, SyncRequest, SyncResponse, UnregisterRequest},
    session::{CreateRequest, CreateResponse, DestroyRequest, UpdateRequest},
    sync::agent_ttrpc,
    tflite::{
        ModelLoadRequest as TFLiteModelLoadRequest, ModelRunRequest as TFLiteModelRunRequest,
        ModelRunResponse as TFLiteModelRunResponse, ModelUnloadRequest as TFLiteModelUnloadRequest,
    },
    torch::{
        ModelLoadRequest as TorchModelLoadRequest, ModelRunRequest as TorchModelRunRequest,
        ModelRunResponse as TorchModelRunResponse,
    },
};

impl agent_ttrpc::AgentService for AgentService {
    fn create_session(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: CreateRequest,
    ) -> ttrpc::Result<CreateResponse> {
        self.do_create_session(req).into_ttrpc()
    }

    fn update_session(
        &self,
        _ctx: &::ttrpc::TtrpcContext,
        req: UpdateRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_update_session(req).into_ttrpc()
    }

    fn destroy_session(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: DestroyRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_destroy_session(req).into_ttrpc()
    }

    fn register_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: RegisterRequest,
    ) -> ttrpc::Result<RegisterResponse> {
        self.do_register_resource(req).into_ttrpc()
    }

    fn unregister_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: UnregisterRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_unregister_resource(req).into_ttrpc()
    }

    fn sync_resource(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: SyncRequest,
    ) -> ttrpc::Result<SyncResponse> {
        self.do_sync_resource(req).into_ttrpc()
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

    fn image_classification(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: ImageRequest,
    ) -> ttrpc::Result<ImageResponse> {
        self.do_image_classification(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_load(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TFModelLoadRequest,
    ) -> ttrpc::Result<TFModelLoadResponse> {
        self.do_tensorflow_model_load(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_unload(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TFModelUnloadRequest,
    ) -> ttrpc::Result<TFModelUnloadResponse> {
        self.do_tensorflow_model_unload(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    fn tensorflow_model_run(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TFModelRunRequest,
    ) -> ttrpc::Result<TFModelRunResponse> {
        self.do_tensorflow_model_run(req).into_ttrpc()
    }

    fn tensorflow_lite_model_load(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TFLiteModelLoadRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_tflite_model_load(req).into_ttrpc()
    }

    fn tensorflow_lite_model_unload(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TFLiteModelUnloadRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_tflite_model_unload(req).into_ttrpc()
    }

    fn tensorflow_lite_model_run(
        &self,
        _ctx: &::ttrpc::sync::TtrpcContext,
        req: TFLiteModelRunRequest,
    ) -> ttrpc::Result<TFLiteModelRunResponse> {
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
}
