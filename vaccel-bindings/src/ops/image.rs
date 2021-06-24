use crate::ffi;
use crate::session::Session;
use crate::{Error, Result};

use std::os::raw::c_void;

impl Session {
    /// Perform image classification
    ///
    /// vAccel image classification using a pre-defined model (TODO: use a registered model)
    ///
    /// # Arguments
    ///
    /// * `img` - The image to classify
    pub fn image_classification(&mut self, img: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut tags = vec![0; 1024];
        let mut out_img = vec![0; 1024];

        match unsafe {
            ffi::vaccel_image_classification(
                self.inner_mut(),
                img.as_ptr() as *mut c_void,
                tags.as_mut_ptr(),
                out_img.as_mut_ptr(),
                img.len() as u64,
                tags.len() as u64,
                out_img.len() as u64,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok((tags, out_img)),
            err => Err(Error::Runtime(err)),
        }
    }

    pub fn image_detection(&mut self, img: &mut [u8]) -> Result<Vec<u8>> {
        let mut out_img = vec![0; img.len()];

        match unsafe {
            ffi::vaccel_image_detection(
                self.inner_mut(),
                img.as_mut_ptr() as *mut c_void,
                out_img.as_mut_ptr(),
                img.len() as u64,
                out_img.len() as u64,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(out_img),
            err => Err(Error::Runtime(err)),
        }
    }

    pub fn image_segmentation(&mut self, img: &mut [u8]) -> Result<Vec<u8>> {
        let mut out_img = vec![0; img.len()];

        match unsafe {
            ffi::vaccel_image_segmentation(
                self.inner_mut(),
                img.as_mut_ptr() as *mut c_void,
                out_img.as_mut_ptr(),
                img.len() as u64,
                out_img.len() as u64,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(out_img),
            err => Err(Error::Runtime(err)),
        }
    }
}
