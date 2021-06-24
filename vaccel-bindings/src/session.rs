use crate::ffi;
use crate::resource::Resource;
use crate::VaccelId;
use crate::{Error, Result};

/// The vAccel session  type
///
/// This is a handle for interacting with the underlying vAccel
/// runtime system.
#[derive(Debug)]
pub struct Session {
    inner: ffi::vaccel_session,
}

impl Session {
    /// Create a new vAccel session
    ///
    /// This will allocate a new vaccel_session structure on the heap and
    /// initialize it.
    ///
    /// # Arguments
    ///
    /// * `flags` - Flags for session creation. Currently ignored.
    pub fn new(flags: u32) -> Result<Self> {
        let mut inner = ffi::vaccel_session::default();

        match unsafe { ffi::vaccel_sess_init(&mut inner, flags) as u32 } {
            ffi::VACCEL_OK => Ok(Session { inner }),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Get the session id
    pub fn id(&self) -> VaccelId {
        VaccelId::from(self.inner.session_id as i64)
    }

    /// Destroy a vAccel session
    ///
    /// This will close an open session and consume it.
    pub fn close(&mut self) -> Result<()> {
        match unsafe { ffi::vaccel_sess_free(&mut self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Register a vAccel resource to a session
    ///
    /// Associate a vAccel resource (such as a TensorFlow model) with a session
    /// for subsequent use with that session
    ///
    /// # Arguments
    ///
    /// * `res` - The resource we are registering to the session. This should have been previously
    /// created in the database of vAccel runtime
    pub fn register(&mut self, res: &mut dyn Resource) -> Result<()> {
        if !res.initialized() {
            return Err(Error::Uninitialized);
        }

        let res_ptr = res.to_mut_vaccel_ptr().ok_or(Error::InvalidArgument)?;

        match unsafe { ffi::vaccel_sess_register(&mut self.inner, res_ptr) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    pub fn unregister(&mut self, res: &mut dyn Resource) -> Result<()> {
        let res_ptr = res.to_mut_vaccel_ptr().ok_or(Error::InvalidArgument)?;

        match unsafe { ffi::vaccel_sess_unregister(&mut self.inner, res_ptr) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    pub(crate) fn inner(&self) -> &ffi::vaccel_session {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_session {
        &mut self.inner
    }
}
