// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::Result;
use log::error;
use std::{ffi::c_int, ptr};
use vaccel::{c_pointer_to_mut_slice, ffi, profiling::SessionProfiler, Arg, Handle};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;
use vaccel_rpc_proto::genop::{Arg as ProtoArg, GenopRequest};
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;

impl VaccelRpcClient {
    pub fn genop(
        &mut self,
        sess_id: i64,
        read_args: Vec<ProtoArg>,
        write_args: Vec<ProtoArg>,
    ) -> Result<Vec<ProtoArg>> {
        let ctx = ttrpc::context::Context::default();

        let req = self.profile_fn(sess_id.into(), "genop > client > req create", || {
            GenopRequest {
                session_id: sess_id,
                read_args,
                write_args,
                ..Default::default()
            }
        });

        let resp = self.profile_fn(
            sess_id.into(),
            "genop > client > ttrpc_client.genop",
            || self.execute(AgentServiceClient::genop, ctx, &req),
        )?;

        Ok(resp.write_args)
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `read_args_ptr` and `write_args_ptr` are expected to be valid pointers to
/// objects allocated manually or by the respective vAccel functions.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_genop(
    client_ptr: *mut VaccelRpcClient,
    sess_id: i64,
    read_args_ptr: *mut ffi::vaccel_arg,
    nr_read_args: usize,
    write_args_ptr: *mut ffi::vaccel_arg,
    nr_write_args: usize,
) -> c_int {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let proto_read_args = {
        let _scope = client.profile_scope(sess_id.into(), "genop > read_args");

        let read_args = match c_pointer_to_mut_slice(read_args_ptr, nr_read_args) {
            Some(slice) => slice,
            None => return ffi::VACCEL_EINVAL as c_int,
        };
        match read_args
            .iter_mut()
            .map(|a| Ok(Arg::from_ref(a)?.into()))
            .collect::<Result<Vec<ProtoArg>>>()
        {
            Ok(arg) => arg,
            Err(e) => {
                error!("{}", e);
                return e.to_ffi() as c_int;
            }
        }
    };

    let (write_args, proto_write_args) = {
        let _scope = client.profile_scope(sess_id.into(), "genop > write_args");

        let write_args = c_pointer_to_mut_slice(write_args_ptr, nr_write_args).unwrap_or(&mut []);
        let proto_write_args = match write_args
            .iter_mut()
            .map(|a| Ok(Arg::from_ref(a)?.into()))
            .collect::<Result<Vec<ProtoArg>>>()
        {
            Ok(arg) => arg,
            Err(e) => {
                error!("{}", e);
                return e.to_ffi() as c_int;
            }
        };

        (write_args, proto_write_args)
    };

    client.start_profiling(sess_id.into(), "genop > client.genop");
    #[cfg(feature = "async-stream")]
    let do_genop = client.genop_stream(sess_id, proto_read_args, proto_write_args);
    #[cfg(not(feature = "async-stream"))]
    let do_genop = client.genop(sess_id, proto_read_args, proto_write_args);
    let ret = match do_genop {
        Ok(result) => {
            client.profile_fn(sess_id.into(), "genop > write_args copy", || {
                for (w, r) in write_args.iter_mut().zip(result.iter()) {
                    unsafe {
                        ptr::copy_nonoverlapping(r.buf.as_ptr(), w.buf as *mut u8, r.size as usize)
                    }
                }
            });

            ffi::VACCEL_OK
        }
        Err(e) => {
            error!("{}", e);
            e.to_ffi()
        }
    } as c_int;
    client.stop_profiling(sess_id.into(), "genop > client.genop");

    ret
}
