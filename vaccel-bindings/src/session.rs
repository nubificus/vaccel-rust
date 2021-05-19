use crate::resource::VaccelResource;
use crate::{
    vaccel_sess_free, vaccel_sess_init, vaccel_sess_register, vaccel_sess_unregister,
    vaccel_session, VACCEL_OK,
};
use crate::{Error, Result};

impl vaccel_session {
    /// Create a new vAccel session
    ///
    /// This will allocate a new vaccel_session structure on the heap and
    /// initialize it.
    ///
    /// # Arguments
    ///
    /// * `flags` - Flags for session creation. Currently ignored.
    pub fn new(flags: u32) -> Result<vaccel_session> {
        let mut sess = vaccel_session::default();

        match unsafe { vaccel_sess_init(&mut sess, flags) as u32 } {
            VACCEL_OK => Ok(sess),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Get the session id
    pub fn id(&self) -> u32 {
        self.session_id
    }

    /// Destroy a vAccel session
    ///
    /// This will close an open session and consume it.
    pub fn close(mut self) -> Result<()> {
        match unsafe { vaccel_sess_free(&mut self) as u32 } {
            VACCEL_OK => Ok(()),
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
    pub fn register(&mut self, res: &mut dyn VaccelResource) -> Result<()> {
        let res_ptr = res.to_mut_vaccel_ptr().ok_or(Error::InvalidArgument)?;

        match unsafe { vaccel_sess_register(self, res_ptr) as u32 } {
            VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    pub fn unregister(&mut self, res: &mut dyn VaccelResource) -> Result<()> {
        let res_ptr = res.to_mut_vaccel_ptr().ok_or(Error::InvalidArgument)?;

        match unsafe { vaccel_sess_unregister(self, res_ptr) as u32 } {
            VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}
