use std::{slice, ptr, convert::TryInto};

use crate::{Error, Result, client::VsockClient};

use vaccel::ffi;

use protobuf::RepeatedField;

use protocols::{genop::{GenopArg, GenopRequest}};

impl VsockClient {
    pub fn genop(
        &self,
        session_id: u32,
        read_args: Vec<GenopArg>,
        write_args: Vec<GenopArg>,
    ) -> Result<Vec<GenopArg>> {
        let ctx = ttrpc::context::Context::default();

        let req = GenopRequest {
            session_id,
            read_args: RepeatedField::from_vec(read_args),
            write_args: RepeatedField::from_vec(write_args),
            ..Default::default()
        };

        let mut resp = self.ttrpc_client.genop(ctx, &req)?;
        if resp.has_error() {
            return Err(resp.take_error().into());
        }

        Ok(resp.take_result().take_write_args().into())
    }
}

#[no_mangle]
pub extern "C" fn genop(
    client_ptr: *const VsockClient,
    sess_id: u32,
    read_args_ptr: *mut ffi::vaccel_arg,
    nr_read_args: usize,
    write_args_ptr: *mut ffi::vaccel_arg,
    nr_write_args: usize,
) -> u32 {

    let read_args: Vec<GenopArg> = match c_pointer_to_slice(read_args_ptr, nr_read_args) {
        Some(slice) => slice.into_iter()
            .map(|e| {
                let size = e.size;
                let buf: Vec<u8> = {
                    c_pointer_to_slice(e.buf as *mut u8, size.try_into().unwrap())
                    .unwrap_or(&[])
                    .to_vec()
                };
                GenopArg {
                    buf: buf,
                    size: size,
                    ..Default::default()
                }
            })
            .collect(),
        None => return ffi::VACCEL_EINVAL,
    };

    let write_args_ref = match c_pointer_to_mut_slice(write_args_ptr, nr_write_args) {
        Some(slice) => slice,
        None => &mut [],
    };

    let write_args: Vec<GenopArg> = match c_pointer_to_mut_slice(write_args_ptr, nr_write_args) {
        Some(slice) => {
            slice.into_iter()
            .map(|e| {
                let size = e.size;
                let buf: Vec<u8> = {
                    c_pointer_to_slice(e.buf as *mut u8, size.try_into().unwrap())
                    .unwrap_or(&[])
                    .to_vec()
                };
                GenopArg {
                    buf: buf,
                    size: size,
                    ..Default::default()
                }
            })
            .collect()
        },
        None => vec![],
    };
 
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL,
    };

    let ret = match client.genop(
        sess_id,
        read_args,
        write_args,
    ) {
        Ok(result) => {
            for (w, r) in write_args_ref.iter_mut().zip(result.iter()) {
                unsafe { 
                    ptr::copy_nonoverlapping(r.buf.as_ptr(), w.buf as *mut u8, r.size as usize)
                }
            };

            ffi::VACCEL_OK
        }
        Err(Error::ClientError(err)) => err,
        Err(_) => ffi::VACCEL_EINVAL,
    };

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
