use crate::{Error, Result, c_pointer_to_mut_slice, c_pointer_to_slice};
use super::client::VsockClient;
use protocols::genop::{GenopArg, GenopRequest, GenopStreamRequest};
use std::{convert::TryInto, ptr};
use vaccel::{ffi, profiling::ProfRegions};
use protobuf::Message;

impl VsockClient {
    pub fn genop(
        &mut self,
        sess_id: u32,
        read_args: Vec<GenopArg>,
        write_args: Vec<GenopArg>,
    ) -> Result<Vec<GenopArg>> {
        let ctx = ttrpc::context::Context::default();

        let mut lock = self.timers.lock().unwrap();
        let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
        //let timers = self.get_timers_entry(sess_id);
        timers.start("genop > client > req create");
        let req = GenopRequest {
            session_id: sess_id,
            read_args: read_args,
            write_args: write_args,
            ..Default::default()
        };
        timers.stop("genop > client > req create");

        timers.start("genop > client > ttrpc_client clone");
        let tc = self.ttrpc_client.clone();
        timers.stop("genop > client > ttrpc_client clone");
        timers.start("genop > client > ttrpc_client.genop");
        let task = async {
            tokio::spawn(async move {
                tc.genop(ctx, &req).await
            }).await
        };

        let resp = self.runtime.block_on(task)?;
 
        /*
        if resp.has_error() {
            return Err(resp.take_error().into());
        }
        */
        //let timers = self.get_timers_entry(sess_id);
        timers.stop("genop > client > ttrpc_client.genop");

        Ok(resp?.take_result().write_args.into())
    }

    pub fn genop_stream(
        &mut self,
        sess_id: u32,
        read_args: Vec<GenopArg>,
        write_args: Vec<GenopArg>,
    ) -> Result<Vec<GenopArg>> {
        let ctx = ttrpc::context::Context::default();

        let mut lock = self.timers.lock().unwrap();
        let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
        //let timers = self.get_timers_entry(sess_id);
        timers.start("genop > client > req create");
        //let (a, b) = read_args.split
        //println!("VEC: {:?}", &read_args);
        /*
        let r_args: Vec<GenopArg> = Vec::new();
        for arg in read_args.iter() {
            r_args.push(GenopArg { size: arg.size, buf: arg.buf };
            println!("ARG: {:?}", arg);
        }
        */
        let req = GenopRequest {
            session_id: sess_id,
            read_args: read_args,
            //read_args: r_args,
            write_args: write_args,
            ..Default::default()
        };
        let max_len = 4194304 - 4 - 1;
        /*
        let mut frombytes = GenopRequest::default();
            let c = frombytes.merge_from_bytes(&[bytes[i]]);
            println!("BYTE: {:?}", c); 
        };
        */
        //let frombytes = GenopRequest::parse_from_bytes(&bytes);
        timers.stop("genop > client > req create");

        timers.start("genop > client > ttrpc_client.genop");
        std::mem::drop(lock);
        let tc = self.ttrpc_client.clone();
        let tmrs = self.timers.clone();
        let task = async {
            tokio::spawn(async move {
                {
                let mut lock = tmrs.lock().unwrap();
                let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                timers.start("genop > client > ttrpc_client.genop > stream");
                }
                let mut stream = tc.genop_stream(ctx).await.unwrap();
                {
                let mut lock = tmrs.lock().unwrap();
                let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                timers.stop("genop > client > ttrpc_client.genop > stream");
                }
                {
                let mut lock = tmrs.lock().unwrap();
                let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                timers.start("genop > client > ttrpc_client.genop > b");
                }
                let bytes = req.write_to_bytes().unwrap();
                let mut len = bytes.len();
                {
                let mut lock = tmrs.lock().unwrap();
                let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                timers.stop("genop > client > ttrpc_client.genop > b");
                }
                //println!("BYTES: {:?}", bytes);
                /*
                for i in 0..bytes.len() {
                    let b = GenopStreamRequest {
                        data: vec![bytes[i]],
                        ..Default::default()
                    };
                    stream.send(&b).await.unwrap();
                }
                */
                let mut s = 0;
                while len > max_len {
                    {
                    let mut lock = tmrs.lock().unwrap();
                    let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                    timers.start("genop > client > ttrpc_client.genop > b");
                    }
                    let b = GenopStreamRequest {
                        data: bytes[s..s+max_len].to_vec(),
                        ..Default::default()
                    };
                    s = s + max_len ;
                    len = len - max_len;
                    {
                    let mut lock = tmrs.lock().unwrap();
                    let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                    timers.stop("genop > client > ttrpc_client.genop > b");
                    }
                    {
                    let mut lock = tmrs.lock().unwrap();
                    let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                    timers.start("genop > client > ttrpc_client.genop > stream");
                    }
                    stream.send(&b).await.unwrap();
                    {
                    let mut lock = tmrs.lock().unwrap();
                    let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                    timers.stop("genop > client > ttrpc_client.genop > stream");
                    }
                }
                    {
                    let mut lock = tmrs.lock().unwrap();
                    let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                    timers.start("genop > client > ttrpc_client.genop > b");
                    }
                let b = GenopStreamRequest {
                    data: bytes[s..s+len].to_vec(),
                    ..Default::default()
                };
                    {
                    let mut lock = tmrs.lock().unwrap();
                    let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                    timers.stop("genop > client > ttrpc_client.genop > b");
                    }

                {
                let mut lock = tmrs.lock().unwrap();
                let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                timers.start("genop > client > ttrpc_client.genop > stream");
                }
                stream.send(&b).await.unwrap();
                {
                let mut lock = tmrs.lock().unwrap();
                let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                timers.stop("genop > client > ttrpc_client.genop > stream");
                }
                /*
                for a in read_args.iter() {
                    r_args.push(GenopArg { size: arg.size, buf: arg.buf };
                                println!("ARG: {:?}", arg);
                                }
                                */
                {
                let mut lock = tmrs.lock().unwrap();
                let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                timers.start("genop > client > ttrpc_client.genop > stream");
                }
                let res = stream.close_and_recv().await;
                {
                let mut lock = tmrs.lock().unwrap();
                let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
                timers.stop("genop > client > ttrpc_client.genop > stream");
                }
                res
            }).await
        };

        let resp = self.runtime.block_on(task)?;
 
        /*
        if resp.has_error() {
            return Err(resp.take_error().into());
        }
        */
        //let timers = self.get_timers_entry(sess_id);
        let mut lock = self.timers.lock().unwrap();
        let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
        timers.stop("genop > client > ttrpc_client.genop");

        Ok(resp?.take_result().write_args.into())
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

    // FIXME: 1) this will lock until the function finishes
    // 2) should check about lock errors
    let mut lock = client.timers.lock().unwrap();
    let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
    timers.start("genop > read_args");
    let read_args: Vec<GenopArg> = match c_pointer_to_slice(read_args_ptr, nr_read_args) {
        /*
        Some(slice) => {
            let args: Vec<GenopArg> = Vec::new();
            for a in slice.into_iter() {
                let size = e.size / 2;
                let buf: Vec<u8> = {
                    c_pointer_to_slice(e.buf as *mut u8, size.try_into().unwrap())
                        .unwrap_or(&[])
                        .to_vec()
                };
                args.push(GenopArg {
                    buf: buf,
                    size: size,
                    ..Default::default()
                });
                args.push(GenopArg {
                    buf: buf,
                    size: size,
                    ..Default::default()
                });
            }
        },*/
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
    timers.stop("genop > read_args");

    timers.start("genop > write_args");
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
    timers.stop("genop > write_args");

    //std::mem::drop(lock);
    timers.start("genop > client.genop");
    std::mem::drop(lock);
    #[cfg(feature = "async-stream")]
    let do_genop = client.genop_stream(sess_id, read_args, write_args);
    #[cfg(not(feature = "async-stream"))]
    let do_genop = client.genop(sess_id, read_args, write_args);
    let ret = match do_genop {
        Ok(result) => {
            let mut lock = client.timers.lock().unwrap();
            let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
            //let mut mg: Option<std::sync::MutexGuard::<'_, std::collections::BTreeMap<u32, ProfRegions>>> = None;
            //let timers = client.get_timers_entry(sess_id, &mut mg);
            timers.start("genop > write_args copy");
            for (w, r) in write_args_ref.iter_mut().zip(result.iter()) {
                unsafe {
                    ptr::copy_nonoverlapping(r.buf.as_ptr(), w.buf as *mut u8, r.size as usize)
                }
            }
            timers.stop("genop > write_args copy");

            ffi::VACCEL_OK
        }
        Err(Error::ClientError(err)) => err,
        Err(_) => ffi::VACCEL_EINVAL,
    };
    let mut lock = client.timers.lock().unwrap();
    let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
    //let timers = client.get_timers_entry(sess_id);
    timers.stop("genop > client.genop");

    //timers.print_total();
    //timers.print();

    ret
}
