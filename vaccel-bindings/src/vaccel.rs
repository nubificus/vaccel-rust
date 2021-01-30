use crate::*;
use std::os::raw::{c_void};

pub type Result<T> = std::result::Result<T, u32>;

unsafe impl Send for vaccel_session {}

impl vaccel_session {
    pub fn new(flags: u32) -> Result<vaccel_session> {
        let mut sess = vaccel_session::default();

        match unsafe { vaccel_sess_init(&mut sess, flags) as u32 } {
            VACCEL_OK => Ok(sess),
            err => Err(err),
        }
    }

    pub fn close(&mut self) -> Result<()> {
        match unsafe { vaccel_sess_free(self) as u32 } {
            VACCEL_OK => Ok(()),
            err => Err(err)
        }
    }

    pub fn noop(&mut self) -> Result<()> {
        match unsafe { vaccel_noop(self) as u32 } {
            VACCEL_OK => Ok(()),
            err => Err(err)
        }
    }

    pub fn image_classification(&mut self, img: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut tags = vec![0; 1024];
        let mut out_img = vec![0; 1024];

        match unsafe {
            vaccel_image_classification(
                self,
                img.as_ptr() as *mut c_void,
                tags.as_mut_ptr(),
                out_img.as_mut_ptr(),
                img.len() as u64,
                tags.len() as u64,
                out_img.len() as u64) as u32
        } {
            VACCEL_OK => Ok((tags, out_img)),
            err => Err(err),
        }
    }

    pub fn image_detection(&mut self, img: &mut [u8]) -> Result<Vec<u8>> {
        let mut out_img = vec![0; img.len()];

        match unsafe {
            vaccel_image_detection(
                self,
                img.as_mut_ptr() as *mut c_void,
                out_img.as_mut_ptr(),
                img.len() as u64,
                out_img.len() as u64) as u32
        } {
            VACCEL_OK => Ok(out_img),
            err => Err(err)
        }
    }

    pub fn image_segmentation(&mut self, img: &mut [u8]) -> Result<Vec<u8>> {
        let mut out_img = vec![0; img.len()];

        match unsafe {
            vaccel_image_segmentation(
                self,
                img.as_mut_ptr() as *mut c_void,
                out_img.as_mut_ptr(),
                img.len() as u64,
                out_img.len() as u64) as u32
        } {
            VACCEL_OK => Ok(out_img),
            err => Err(err)
        }
    }
}
