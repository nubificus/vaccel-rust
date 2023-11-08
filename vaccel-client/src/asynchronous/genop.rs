use super::client::VsockClient;
use crate::{c_pointer_to_mut_slice, c_pointer_to_slice, Error, Result};
use protobuf::Message;
use protocols::genop::{GenopArg, GenopRequest, GenopResponse};
use std::{convert::TryInto, ptr};
use ttrpc::asynchronous::ClientStreamSender;
use vaccel::ffi;

impl VsockClient {
    pub fn genop(
        &mut self,
        sess_id: u32,
        read_args: Vec<GenopArg>,
        write_args: Vec<GenopArg>,
    ) -> Result<Vec<GenopArg>> {
        let ctx = ttrpc::context::Context::default();

        self.timer_start(sess_id, "genop > client > req create");
        let req = GenopRequest {
            session_id: sess_id,
            read_args: read_args,
            write_args: write_args,
            ..Default::default()
        };
        self.timer_stop(sess_id, "genop > client > req create");

        self.timer_start(sess_id, "genop > client > ttrpc_client.genop");
        let tc = self.ttrpc_client.clone();
        let mut resp = self.runtime.block_on(async { tc.genop(ctx, &req).await })?;
        self.timer_stop(sess_id, "genop > client > ttrpc_client.genop");

        Ok(resp.take_result().write_args.into())
    }

    const MAX_REQ_LEN: u64 = 4194304;

    async fn genop_stream_send_args(
        &mut self,
        sess_id: u32,
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

                let b = a
                    .buf
                    .chunks(Self::MAX_REQ_LEN as usize - std::mem::size_of::<GenopArg>() as usize);
                let parts = b.len();
                let mut no = 0;
                for i in b {
                    no = no + 1;
                    self.timer_start(sess_id, "genop > client > ttrpc_client.genop > req create");
                    let arg = GenopArg {
                        buf: i.to_vec(),
                        size: a.buf.len() as u32,
                        parts: parts as u32,
                        part_no: no,
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
                continue;
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
        sess_id: u32,
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

        Ok(resp.take_result().write_args.into())
    }
}

#[no_mangle]
pub extern "C" fn genop(
    client_ptr: *mut VsockClient,
    sess_id: u32,
    read_args_ptr: *mut ffi::vaccel_arg,
    nr_read_args: usize,
    write_args_ptr: *mut ffi::vaccel_arg,
    nr_write_args: usize,
) -> u32 {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL,
    };

    client.timer_start(sess_id, "genop > read_args");
    let read_args: Vec<GenopArg> = match c_pointer_to_slice(read_args_ptr, nr_read_args) {
        Some(slice) => slice
            .into_iter()
            .map(|e| {
                let size = e.size;
                let argtype = e.argtype;
                let buf: Vec<u8> = {
                    c_pointer_to_slice(e.buf as *mut u8, size.try_into().unwrap())
                        .unwrap_or(&[])
                        .to_vec()
                };
                GenopArg {
                    buf: buf,
                    size: size,
                    argtype: argtype,
                    ..Default::default()
                }
            })
            .collect(),
        None => return ffi::VACCEL_EINVAL,
    };
    client.timer_stop(sess_id, "genop > read_args");

    client.timer_start(sess_id, "genop > write_args");
    let write_args_ref = match c_pointer_to_mut_slice(write_args_ptr, nr_write_args) {
        Some(slice) => slice,
        None => &mut [],
    };

    let write_args: Vec<GenopArg> = match c_pointer_to_mut_slice(write_args_ptr, nr_write_args) {
        Some(slice) => slice
            .into_iter()
            .map(|e| {
                let size = e.size;
                let argtype = e.argtype;
                let buf: Vec<u8> = {
                    c_pointer_to_slice(e.buf as *mut u8, size.try_into().unwrap())
                        .unwrap_or(&[])
                        .to_vec()
                };
                GenopArg {
                    buf: buf,
                    size: size,
                    argtype: argtype,
                    ..Default::default()
                }
            })
            .collect(),
        None => vec![],
    };
    client.timer_stop(sess_id, "genop > write_args");

    client.timer_start(sess_id, "genop > client.genop");
    #[cfg(feature = "async-stream")]
    let do_genop = client.genop_stream(sess_id, read_args, write_args);
    #[cfg(not(feature = "async-stream"))]
    let do_genop = client.genop(sess_id, read_args, write_args);
    let ret = match do_genop {
        Ok(result) => {
            client.timer_start(sess_id, "genop > write_args copy");
            for (w, r) in write_args_ref.iter_mut().zip(result.iter()) {
                unsafe {
                    ptr::copy_nonoverlapping(r.buf.as_ptr(), w.buf as *mut u8, r.size as usize)
                }
            }
            client.timer_stop(sess_id, "genop > write_args copy");

            ffi::VACCEL_OK
        }
        Err(Error::ClientError(err)) => err,
        Err(e) => {
            println!("-- {:#?}", e);
            ffi::VACCEL_EINVAL
        }
    };
    client.timer_stop(sess_id, "genop > client.genop");

    ret
}
