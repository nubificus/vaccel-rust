// SPDX-License-Identifier: Apache-2.0

use crate::ffi;
use vaccel_rpc_proto::genop::Arg as ProtoArg;

#[derive(Debug)]
pub struct Arg {
    inner: ffi::vaccel_arg,
    buf: Vec<u8>,
    size: usize,
    argtype: usize,
}

impl Arg {
    pub fn new(buffer: &mut [u8], size: usize, argtype: usize) -> Self {
        let mut b = buffer.to_owned();
        Arg {
            inner: ffi::vaccel_arg {
                buf: b.as_mut_ptr() as *mut libc::c_void,
                size: size as u32,
                argtype: argtype as u32,
            },
            buf: b,
            size,
            argtype,
        }
    }
    pub fn size(&self) -> u32 {
        self.inner.size
    }

    pub fn set_size(&mut self, v: usize) {
        self.size = v;
        self.inner.size = v as u32;
    }

    pub fn buf(&self) -> *mut u8 {
        self.inner.buf as *mut u8
    }

    pub fn argtype(&self) -> u32 {
        self.inner.argtype
    }

    pub fn set_buf(&mut self, b: &mut [u8]) {
        self.buf = b.to_owned();
    }

    pub(crate) fn inner(&self) -> &ffi::vaccel_arg {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_arg {
        &mut self.inner
    }
}

impl From<&mut ProtoArg> for Arg {
    fn from(arg: &mut ProtoArg) -> Self {
        let argtype = arg.argtype;
        let size = arg.size;
        let buf = arg.buf.as_mut_slice();
        Arg::new(buf, size as usize, argtype as usize)
    }
}

impl From<&Arg> for ProtoArg {
    fn from(arg: &Arg) -> Self {
        ProtoArg {
            buf: arg.buf.to_owned(),
            size: arg.size as u32,
            ..Default::default()
        }
    }
}
