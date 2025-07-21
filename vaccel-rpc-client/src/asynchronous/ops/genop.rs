// SPDX-License-Identifier: Apache-2.0

use super::client::VaccelRpcClient;
use crate::Result;
use protobuf::Message;
use ttrpc::asynchronous::ClientStreamSender;
use vaccel::{profiling::SessionProfiler, VaccelId};
use vaccel_rpc_proto::genop::{Arg, Request, Response};

impl VaccelRpcClient {
    const MAX_REQ_LEN: u64 = 4194304;

    async fn genop_stream_send_args(
        &mut self,
        sess_id: i64,
        stream: &ClientStreamSender<Request, Response>,
        args: Vec<Arg>,
        is_read: bool,
    ) {
        let sess_vaccel_id = VaccelId::try_from(sess_id).unwrap();

        let mut req = Request {
            session_id: sess_vaccel_id.into(),
            ..Default::default()
        };

        for a in args {
            if req.compute_size() + a.compute_size() < Self::MAX_REQ_LEN {
                self.profile_fn(
                    sess_vaccel_id,
                    "genop > client > ttrpc_client.genop > req create",
                    || match is_read {
                        true => req.read_args.push(a),
                        false => req.write_args.push(a),
                    },
                );
            } else {
                self.profile_async_fn(
                    sess_vaccel_id,
                    "genop > client > ttrpc_client.genop > stream",
                    || async { stream.send(&req).await.unwrap() },
                )
                .await;

                let chunks = a
                    .buf
                    .chunks(Self::MAX_REQ_LEN as usize - std::mem::size_of::<Arg>());
                let parts = chunks.len();
                for (no, c) in chunks.enumerate() {
                    let req = self
                        .profile_async_fn(
                            sess_vaccel_id,
                            "genop > client > ttrpc_client.genop > req create",
                            || async {
                                let arg = Arg {
                                    buf: c.to_vec(),
                                    size: a.buf.len() as u32,
                                    parts: parts as u32,
                                    part_no: (no + 1) as u32,
                                    ..Default::default()
                                };
                                match is_read {
                                    true => Request {
                                        session_id: sess_id,
                                        read_args: vec![arg],
                                        ..Default::default()
                                    },
                                    false => Request {
                                        session_id: sess_id,
                                        write_args: vec![arg],
                                        ..Default::default()
                                    },
                                }
                            },
                        )
                        .await;

                    self.profile_async_fn(
                        sess_vaccel_id,
                        "genop > client > ttrpc_client.genop > stream",
                        || async { stream.send(&req).await.unwrap() },
                    )
                    .await;
                }
                req = Request {
                    session_id: sess_id,
                    ..Default::default()
                };
            }
        }

        let args_len = match is_read {
            true => req.read_args.len(),
            false => req.write_args.len(),
        };
        if args_len > 0 {
            self.profile_async_fn(
                sess_vaccel_id,
                "genop > client > ttrpc_client.genop > stream",
                || async { stream.send(&req).await.unwrap() },
            )
            .await;
        }
    }

    pub fn genop_stream(
        &mut self,
        sess_id: i64,
        read_args: Vec<Arg>,
        write_args: Vec<Arg>,
    ) -> Result<Vec<Arg>> {
        let ctx = ttrpc::context::Context::default();
        let sess_vaccel_id = VaccelId::try_from(sess_id)?;

        self.start_profiling(sess_vaccel_id, "genop > client > ttrpc_client.genop");

        let tc = self.ttrpc_client.clone();
        let runtime = self.runtime.clone();
        let resp = runtime
            .block_on(async {
                let mut stream = self
                    .profile_async_fn(
                        sess_vaccel_id,
                        "genop > client > ttrpc_client.genop > stream",
                        || async { tc.genop_stream(ctx).await.unwrap() },
                    )
                    .await;

                self.genop_stream_send_args(sess_id, &stream, read_args, true)
                    .await;
                self.genop_stream_send_args(sess_id, &stream, write_args, false)
                    .await;

                self.profile_async_fn(
                    sess_vaccel_id,
                    "genop > client > ttrpc_client.genop > stream",
                    || async { stream.close_and_recv().await },
                )
                .await
            })
            .inspect_err(|_| {
                self.stop_profiling(sess_vaccel_id, "genop > client > ttrpc_client.genop");
            })?;

        self.stop_profiling(sess_vaccel_id, "genop > client > ttrpc_client.genop");

        Ok(resp.write_args)
    }
}
