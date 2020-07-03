use super::super::*;
use crate::vaccel::Result;

pub fn new_session(
    out_args: &mut [vaccelrt_arg],
    in_args: &mut [vaccelrt_arg]
) -> Result<vaccelrt_session> {

    let mut sess = vaccelrt_session::default();
    let mut sess_type: u32 = 0;

    let ret = unsafe {
        vaccelrt_sess_init(
            &mut sess,
            out_args.as_mut_ptr(),
            in_args.as_mut_ptr(),
            out_args.len() as u32,
            in_args.len() as u32,
            &mut sess_type,
        )
    };

    if ret != VACCELRT_OK as i32 {
        return Err(ret)
    }

    Ok(sess)
}

pub fn close_session(sess: &mut vaccelrt_session) -> Result<()>
{
    unsafe {
        vaccelrt_sess_free(sess)
    };

    Ok(())
}

pub fn vaccel_op(
    sess: &mut vaccelrt_session,
    out_args: &mut [vaccelrt_arg],
    in_args: &mut [vaccelrt_arg]
) -> Result<()>
{

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
