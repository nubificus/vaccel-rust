#[cfg(feature = "async")]
use crate::asynchronous::client::VsockClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VsockClient;
use crate::{c_pointer_to_mut_slice, c_pointer_to_slice, Error, Result};
use log::error;
#[cfg(feature = "async")]
use protocols::asynchronous::agent_ttrpc::VaccelAgentClient;
use protocols::genop::{GenopArg, GenopRequest};
#[cfg(not(feature = "async"))]
use protocols::sync::agent_ttrpc::VaccelAgentClient;
use std::{convert::TryInto, ptr};
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
            read_args,
            write_args,
            ..Default::default()
        };
        self.timer_stop(sess_id, "genop > client > req create");

        self.timer_start(sess_id, "genop > client > ttrpc_client.genop");
        let mut resp = self.execute(VaccelAgentClient::genop, ctx, &req)?;
        self.timer_stop(sess_id, "genop > client > ttrpc_client.genop");
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        Ok(resp.take_result().write_args)
    }
}

#[no_mangle]
pub unsafe extern "C" fn genop(
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
            .iter()
            .map(|e| {
                let size = e.size;
                let argtype = e.argtype;
                let buf: Vec<u8> = {
                    c_pointer_to_slice(e.buf as *mut u8, size.try_into().unwrap())
                        .unwrap_or(&[])
                        .to_vec()
                };
                GenopArg {
                    buf,
                    size,
                    argtype,
                    ..Default::default()
                }
            })
            .collect(),
        None => return ffi::VACCEL_EINVAL,
    };
    client.timer_stop(sess_id, "genop > read_args");

    client.timer_start(sess_id, "genop > write_args");
    let write_args_ref = c_pointer_to_mut_slice(write_args_ptr, nr_write_args).unwrap_or(&mut []);

    let write_args: Vec<GenopArg> = match c_pointer_to_mut_slice(write_args_ptr, nr_write_args) {
        Some(slice) => slice
            .iter_mut()
            .map(|e| {
                let size = e.size;
                let argtype = e.argtype;
                let buf: Vec<u8> = {
                    c_pointer_to_slice(e.buf as *mut u8, size.try_into().unwrap())
                        .unwrap_or(&[])
                        .to_vec()
                };
                GenopArg {
                    buf,
                    size,
                    argtype,
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
            error!("Genop: {:?}", e);
            ffi::VACCEL_EINVAL
        }
    };
    client.timer_stop(sess_id, "genop > client.genop");

    ret
}
