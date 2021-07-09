use crate::ffi;
use crate::Session;
use crate::{Error, Result};

pub struct GenopArg {
    inner: ffi::vaccel_arg,
}

impl GenopArg {
    pub fn new(ptr: *mut u8, size: usize) -> Self {
        GenopArg {
            inner: ffi::vaccel_arg {
                buf: ptr as *mut libc::c_void,
                size: size as u32,
            },
        }
    }
}

impl Session {
    /// vAccel generic operation
    ///
    /// Execute an arbitrary vAccel operation passing to vaccelrt arguments
    /// in the generic form of `vaccel_arg` slices. `vaccel_arg` describes an
    /// argument as a generic `void *` pointer and a size.
    ///
    /// # Arguments
    ///
    /// * `read` - A slice of `vaccel_arg` with the arguments that are read only. The first
    /// argument of the slice is the type of the operation
    /// * `write` - A slice of `vaccel_arg` with the read-write arguments of the operation.
    pub fn genop(&mut self, read: &mut [GenopArg], write: &mut [GenopArg]) -> Result<()> {
        let mut read_args: Vec<ffi::vaccel_arg> = read.iter().map(|e| e.inner).collect();
        let mut write_args: Vec<ffi::vaccel_arg> = write.iter().map(|e| e.inner).collect();

        match unsafe {
            ffi::vaccel_genop(
                self.inner_mut(),
                read_args.as_mut_ptr(),
                read_args.len() as i32,
                write_args.as_mut_ptr(),
                write_args.len() as i32,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}
