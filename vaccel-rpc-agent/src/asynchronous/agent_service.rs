// SPDX-License-Identifier: Apache-2.0

use crate::{agent_service::IntoTtrpcResult, AgentService};
use async_trait::async_trait;
use log::debug;
use std::default::Default;
#[allow(unused_imports)]
use vaccel_rpc_proto::tf::{
    ModelLoadRequest as TFModelLoadRequest, ModelLoadResponse as TFModelLoadResponse,
    ModelRunRequest as TFModelRunRequest, ModelRunResponse as TFModelRunResponse,
    ModelUnloadRequest as TFModelUnloadRequest, ModelUnloadResponse as TFModelUnloadResponse,
};
use vaccel_rpc_proto::{
    asynchronous::agent_ttrpc,
    empty::Empty,
    genop::{Arg, Request as GenopRequest, Response as GenopResponse},
    image::{Request as ImageRequest, Response as ImageResponse},
    profiling::{Request as ProfilingRequest, Response as ProfilingResponse},
    resource::{RegisterRequest, RegisterResponse, SyncRequest, SyncResponse, UnregisterRequest},
    session::{CreateRequest, CreateResponse, DestroyRequest, UpdateRequest},
    tflite::{
        ModelLoadRequest as TFLiteModelLoadRequest, ModelRunRequest as TFLiteModelRunRequest,
        ModelRunResponse as TFLiteModelRunResponse, ModelUnloadRequest as TFLiteModelUnloadRequest,
    },
    torch::{
        ModelLoadRequest as TorchModelLoadRequest, ModelRunRequest as TorchModelRunRequest,
        ModelRunResponse as TorchModelRunResponse,
    },
};
//use tracing::{info, instrument, Instrument};

#[async_trait]
impl agent_ttrpc::AgentService for AgentService {
    async fn create_session(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: CreateRequest,
    ) -> ttrpc::Result<CreateResponse> {
        self.do_create_session(req).into_ttrpc()
    }

    async fn update_session(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: UpdateRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_update_session(req).into_ttrpc()
    }

    async fn destroy_session(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: DestroyRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_destroy_session(req).into_ttrpc()
    }

    async fn register_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: RegisterRequest,
    ) -> ttrpc::Result<RegisterResponse> {
        self.do_register_resource(req).into_ttrpc()
    }

    async fn unregister_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: UnregisterRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_unregister_resource(req).into_ttrpc()
    }

    async fn sync_resource(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: SyncRequest,
    ) -> ttrpc::Result<SyncResponse> {
        self.do_sync_resource(req).into_ttrpc()
    }

    async fn genop(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: GenopRequest,
    ) -> ttrpc::Result<GenopResponse> {
        self.do_genop(req).into_ttrpc()
    }

    async fn genop_stream(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        mut r: ::ttrpc::asynchronous::ServerStreamReceiver<GenopRequest>,
    ) -> ttrpc::Result<GenopResponse> {
        let mut req = GenopRequest::default();
        let mut r_arg = vec![Arg::new()];
        let mut w_arg = vec![Arg::new()];
        while let Some(mut data) = r.recv().await? {
            req.session_id = data.session_id;
            if data.read_args.len() == 1 && data.read_args[0].parts > 0 {
                if data.read_args[0].part_no < data.read_args[0].parts {
                    r_arg[0].buf.append(&mut data.read_args[0].buf);
                } else {
                    r_arg[0].buf.append(&mut data.read_args[0].buf);
                    r_arg[0].size = data.read_args[0].size;
                    r_arg[0].arg_type = data.read_args[0].arg_type;
                    r_arg[0].custom_type_id = data.read_args[0].custom_type_id;
                    req.read_args.append(&mut r_arg);
                    r_arg = vec![Arg::new()];
                }
            } else if data.write_args.len() == 1 && data.write_args[0].parts > 0 {
                if data.write_args[0].part_no < data.write_args[0].parts {
                    w_arg[0].buf.append(&mut data.write_args[0].buf);
                } else {
                    w_arg[0].buf.append(&mut data.write_args[0].buf);
                    w_arg[0].size = data.write_args[0].size;
                    w_arg[0].arg_type = data.write_args[0].arg_type;
                    w_arg[0].custom_type_id = data.write_args[0].custom_type_id;
                    req.write_args.append(&mut w_arg);
                    w_arg = vec![Arg::new()];
                }
            } else {
                req.read_args.append(&mut data.read_args);
                req.write_args.append(&mut data.write_args);
            }
        }

        debug!("Genop is streaming");
        self.do_genop(req).into_ttrpc()
    }

    async fn get_profiler(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: ProfilingRequest,
    ) -> ttrpc::Result<ProfilingResponse> {
        self.do_get_profiler(req).into_ttrpc()
    }

    async fn image_classification(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: ImageRequest,
    ) -> ttrpc::Result<ImageResponse> {
        self.do_image_classification(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    async fn tensorflow_model_load(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TFModelLoadRequest,
    ) -> ttrpc::Result<TFModelLoadResponse> {
        self.do_tensorflow_model_load(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    async fn tensorflow_model_unload(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TFModelUnloadRequest,
    ) -> ttrpc::Result<TFModelUnloadResponse> {
        self.do_tensorflow_model_unload(req).into_ttrpc()
    }

    #[cfg(target_pointer_width = "64")]
    async fn tensorflow_model_run(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TFModelRunRequest,
    ) -> ttrpc::Result<TFModelRunResponse> {
        self.do_tensorflow_model_run(req).into_ttrpc()
    }

    async fn tensorflow_lite_model_load(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TFLiteModelLoadRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_tflite_model_load(req).into_ttrpc()
    }

    async fn tensorflow_lite_model_unload(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TFLiteModelUnloadRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_tflite_model_unload(req).into_ttrpc()
    }

    async fn tensorflow_lite_model_run(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TFLiteModelRunRequest,
    ) -> ttrpc::Result<TFLiteModelRunResponse> {
        self.do_tflite_model_run(req).into_ttrpc()
    }

    async fn torch_model_load(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TorchModelLoadRequest,
    ) -> ttrpc::Result<Empty> {
        self.do_torch_model_load(req).into_ttrpc()
    }

    async fn torch_model_run(
        &self,
        _ctx: &::ttrpc::asynchronous::TtrpcContext,
        req: TorchModelRunRequest,
    ) -> ttrpc::Result<TorchModelRunResponse> {
        self.do_torch_model_run(req).into_ttrpc()
    }
}
