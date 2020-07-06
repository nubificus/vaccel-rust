use super::super::*;
use crate::vaccel::Result;
use std::collections::HashMap;

pub struct VaccelRuntime {
    /// should be enough for now
    next_sess_id: u32,

    /// all live sessions
    sessions: HashMap<u32, vaccelrt_session>,
}

impl VaccelRuntime {
    fn next_session_id(&mut self) -> Result<u32> {
        match self.next_sess_id.checked_add(1) {
            Some(id) => Ok(id),
            None => Err(VACCELRT_ERR),
        }
    }

    pub fn new() -> Self {
        VaccelRuntime {
            next_sess_id: 0,
            sessions: HashMap::new(),
        }
    }

    pub fn new_session(
        &mut self,
        out_args: &mut [vaccelrt_arg],
        in_args: &mut [vaccelrt_arg]
    ) -> Result<u32> {

        let mut sess = vaccelrt_session::default();
        let mut sess_type: u32 = 0;
        let id = self.next_session_id()?;

        let err = unsafe {
            vaccelrt_sess_init(
                &mut sess,
                out_args.as_mut_ptr(),
                in_args.as_mut_ptr(),
                out_args.len() as u32,
                in_args.len() as u32,
                &mut sess_type,
            )
        };

        if err != VACCELRT_OK as i32 {
            return Err(err)
        }

        self.sessions.insert(id, sess);
        Ok(id)
    }

    pub fn close_session(&mut self, sess_id: u32) -> Result<()> {
        if let Some(mut sess) = self.sessions.remove(&sess_id) {
            unsafe {
                vaccelrt_sess_free(&mut sess)
            };

            Ok(())
        } else {
            Err(VACCELRT_ERR)
        }
    }

    pub fn vaccel_op(
        &mut self,
        sess_id: u32,
        out_args: &mut [vaccelrt_arg],
        in_args: &mut [vaccelrt_arg]
    ) -> Result<()> {

        let sess = match self.sessions.get_mut(&sess_id) {
            Some(_sess) => _sess,
            None => return Err(VACCELRT_ERR),
        };

        let ret = unsafe {
            vaccelrt_do_op(
                sess,
                out_args.as_mut_ptr(),
                in_args.as_mut_ptr(),
                out_args.len() as u32,
                in_args.len() as u32,
                )
        };

        if ret != VACCELRT_OK as i32 {
            return Err(ret)
        }

        Ok(())
    }
}
