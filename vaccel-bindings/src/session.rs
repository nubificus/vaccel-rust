// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Result, VaccelId};

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

    /// update hint for session
    pub fn update(&mut self, flags: u32) {
        self.inner.hint = flags;
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

    pub(crate) fn inner(&self) -> &ffi::vaccel_session {
        &self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_session {
        &mut self.inner
    }
}
