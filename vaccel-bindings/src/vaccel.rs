use crate::*;
use std::os::raw::{c_void};

pub type Result<T> = std::result::Result<T, u32>;

unsafe impl Send for vaccel_session {}

pub fn new_session(
    sess: &mut vaccel_session,
    flags: u32
    ) -> Result<()> {
    let err = unsafe { vaccel_sess_init(sess, flags) as u32 };

    if err == VACCEL_OK {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn close_session(sess: &mut vaccel_session) -> Result<()> {
    let err = unsafe { vaccel_sess_free(sess) as u32 };

    if err == VACCEL_OK {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn noop(sess: &mut vaccel_session) -> Result<()> {
    let err = unsafe { vaccel_noop(sess) as u32 };

    if err == VACCEL_OK {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn sgemm(
    sess: &mut vaccel_session,
    k: u32,
    m: u32,
    n: u32,
    a: &mut [f32],
    b: &mut [f32],
    c: &mut [f32],
) -> Result<()> {
    let err = unsafe {
        vaccel_sgemm(
            sess,
            k, m, n,
            a.len() as u64,
            b.len() as u64,
            c.len() as u64,
            a.as_mut_ptr(),
            b.as_mut_ptr(),
            c.as_mut_ptr(),
        ) as u32
    };

    if err == VACCEL_OK {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn image_classification(
    sess: &mut vaccel_session,
    img: &mut [u8],
    out_text: &mut [u8],
    out_imgname: &mut [u8],
) -> Result<()> {
    let err = unsafe {
        vaccel_image_classification(
            sess,
            img.as_mut_ptr() as *mut c_void,
            out_text.as_mut_ptr(),
            out_imgname.as_mut_ptr(),
            img.len() as u64,
            out_text.len() as u64,
            out_imgname.len() as u64,
        ) as u32
    };

    if err == VACCEL_OK {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn image_detection(
    sess: &mut vaccel_session,
    img: &mut [u8],
    out_imgname: &mut [u8],
) -> Result<()> {
    let err = unsafe {
        vaccel_image_detection(
            sess,
            img.as_mut_ptr() as *mut c_void,
            out_imgname.as_mut_ptr(),
            img.len() as u64,
            out_imgname.len() as u64
        ) as u32
    };

    if err == VACCEL_OK {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn image_segmentation(
    sess: &mut vaccel_session,
    img: &mut [u8],
    out_imgname: &mut [u8],
) -> Result<()> {
    let err = unsafe {
        vaccel_image_segmentation(
            sess,
            img.as_mut_ptr() as *mut c_void,
            out_imgname.as_mut_ptr(),
            img.len() as u64,
            out_imgname.len() as u64
        ) as u32
    };

    if err == VACCEL_OK {
        Ok(())
    } else {
        Err(err)
    }
}
