use crate::ffi::{vaccel_arg, vaccel_genop, vaccel_session, VACCEL_OK};
use crate::{Error, Result};

impl vaccel_session {
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
    pub fn genop(&mut self, read: &mut [vaccel_arg], write: &mut [vaccel_arg]) -> Result<()> {
        match unsafe {
            vaccel_genop(
                self,
                read.as_mut_ptr(),
                read.len() as i32,
                write.as_mut_ptr(),
                write.len() as i32,
            ) as u32
        } {
            VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}
