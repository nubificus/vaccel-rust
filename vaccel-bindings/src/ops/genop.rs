// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Arg, Error, Handle, Result, Session};

impl Session {
    /// Performs the Generic operation.
    pub fn genop(&mut self, read: &mut [Arg], write: &mut [Arg]) -> Result<()> {
        let mut read_args: Vec<ffi::vaccel_arg> =
            read.iter().map(|e| unsafe { *e.as_ptr() }).collect();
        let mut write_args: Vec<ffi::vaccel_arg> =
            write.iter().map(|e| unsafe { *e.as_ptr() }).collect();

        match unsafe {
            let res = ffi::vaccel_genop(
                self.as_mut_ptr(),
                read_args.as_mut_ptr(),
                read_args.len() as i32,
                write_args.as_mut_ptr(),
                write_args.len() as i32,
            );
            res as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Ffi(err)),
        }
    }
}
