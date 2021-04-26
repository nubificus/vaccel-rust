use crate::{vaccel_noop, vaccel_session, VACCEL_OK};
use crate::{Error, Result};

impl vaccel_session {
    /// The vAccel noop operation
    ///
    /// This is just a debug operation
    pub fn noop(&mut self) -> Result<()> {
        match unsafe { vaccel_noop(self) as u32 } {
            VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}
