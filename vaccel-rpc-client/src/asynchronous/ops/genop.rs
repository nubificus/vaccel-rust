// SPDX-License-Identifier: Apache-2.0

use super::client::VaccelRpcClient;
use crate::Result;
use protobuf::Message;
use ttrpc::asynchronous::ClientStreamSender;
use vaccel_rpc_proto::genop::{GenopArg, GenopRequest, GenopResponse};

impl VaccelRpcClient {
    const MAX_REQ_LEN: u64 = 4194304;

    async fn genop_stream_send_args(
        &mut self,
        sess_id: i64,
        stream: &ClientStreamSender<GenopRequest, GenopResponse>,
        args: Vec<GenopArg>,
        is_read: bool,
    ) {
        let mut req = GenopRequest {
            session_id: sess_id,
            ..Default::default()
        };

        for a in args {
            if req.compute_size() + a.compute_size() < Self::MAX_REQ_LEN {
                self.timer_start(sess_id, "genop > client > ttrpc_client.genop > req create");
                match is_read {
                    true => req.read_args.push(a),
                    false => req.write_args.push(a),
                };
                self.timer_stop(sess_id, "genop > client > ttrpc_client.genop > req create");
            } else {
                self.timer_start(sess_id, "genop > client > ttrpc_client.genop > stream");
                stream.send(&req).await.unwrap();
                self.timer_stop(sess_id, "genop > client > ttrpc_client.genop > stream");

                let chunks = a
                    .buf
                    .chunks(Self::MAX_REQ_LEN as usize - std::mem::size_of::<GenopArg>());
                let parts = chunks.len();
                for (no, c) in chunks.enumerate() {
                    self.timer_start(sess_id, "genop > client > ttrpc_client.genop > req create");
                    let arg = GenopArg {
                        buf: c.to_vec(),
                        size: a.buf.len() as u32,
                        parts: parts as u32,
                        part_no: (no + 1) as u32,
                        ..Default::default()
                    };
                    req = match is_read {
                        true => GenopRequest {
                            session_id: sess_id,
                            read_args: vec![arg],
                            ..Default::default()
                        },
                        false => GenopRequest {
                            session_id: sess_id,
                            write_args: vec![arg],
                            ..Default::default()
                        },
                    };
                    self.timer_stop(sess_id, "genop > client > ttrpc_client.genop > req create");

                    self.timer_start(sess_id, "genop > client > ttrpc_client.genop > stream");
                    stream.send(&req).await.unwrap();
                    self.timer_stop(sess_id, "genop > client > ttrpc_client.genop > stream");
                }
                req = GenopRequest {
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
            self.timer_start(sess_id, "genop > client > ttrpc_client.genop > stream");
            stream.send(&req).await.unwrap();
            self.timer_stop(sess_id, "genop > client > ttrpc_client.genop > stream");
        }
    }

    pub fn genop_stream(
        &mut self,
        sess_id: i64,
        read_args: Vec<GenopArg>,
        write_args: Vec<GenopArg>,
    ) -> Result<Vec<GenopArg>> {
        let ctx = ttrpc::context::Context::default();

        self.timer_start(sess_id, "genop > client > ttrpc_client.genop");
        let tc = self.ttrpc_client.clone();
        let runtime = self.runtime.clone();
        let mut resp = runtime.block_on(async {
            self.timer_start(sess_id, "genop > client > ttrpc_client.genop > stream");
            let mut stream = tc.genop_stream(ctx).await.unwrap();
            self.timer_stop(sess_id, "genop > client > ttrpc_client.genop > stream");

            self.genop_stream_send_args(sess_id, &stream, read_args, true)
                .await;
            self.genop_stream_send_args(sess_id, &stream, write_args, false)
                .await;

            self.timer_start(sess_id, "genop > client > ttrpc_client.genop > stream");
            let res = stream.close_and_recv().await;
            self.timer_stop(sess_id, "genop > client > ttrpc_client.genop > stream");

            res
        })?;
        self.timer_stop(sess_id, "genop > client > ttrpc_client.genop");
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        Ok(resp.take_result().write_args)
    }
}
