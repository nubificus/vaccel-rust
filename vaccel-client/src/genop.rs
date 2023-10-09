use crate::{client::VsockClient, Error, Result};
use protocols::genop::{GenopArg, GenopRequest};
use std::{convert::TryInto, ptr, slice};
use vaccel::ffi;

impl VsockClient {
    pub fn genop(
        &mut self,
        sess_id: u32,
        read_args: Vec<GenopArg>,
        write_args: Vec<GenopArg>,
    ) -> Result<Vec<GenopArg>> {
        let ctx = ttrpc::context::Context::default();

        let timers = self.get_timers_entry(sess_id);
        timers.start("genop > client > req create");
        let req = GenopRequest {
            session_id: sess_id,
            read_args: read_args,
            write_args: write_args,
            ..Default::default()
        };
        timers.stop("genop > client > req create");

        timers.start("genop > client > ttrpc_client.genop");
        let mut resp = self.ttrpc_client.genop(ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }
        let timers = self.get_timers_entry(sess_id);
        timers.stop("genop > client > ttrpc_client.genop");

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

    let timers = client.get_timers_entry(sess_id);
    timers.start("genop > read_args");
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
                let argtype = e.argtype;
                let size = e.size;
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

    timers.start("genop > client.genop");
    let ret = match client.genop(sess_id, read_args, write_args) {
        Ok(result) => {
            let timers = client.get_timers_entry(sess_id);
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
    let timers = client.get_timers_entry(sess_id);
    timers.stop("genop > client.genop");

    //timers.print_total();
    //timers.print();

    ret
}

fn c_pointer_to_slice<'a, T>(buf: *const T, len: usize) -> Option<&'a [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(buf, len) })
    }
}

fn c_pointer_to_mut_slice<'a, T>(buf: *mut T, len: usize) -> Option<&'a mut [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts_mut(buf, len) })
    }
}
