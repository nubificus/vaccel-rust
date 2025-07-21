// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Handle, Result, Session};
use std::os::raw::c_void;

impl Session {
    /// Performs image classification.
    pub fn image_classification(&mut self, img: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut tags = vec![0; 1024];
        let mut out_img = vec![0; 1024];

        match unsafe {
            ffi::vaccel_image_classification(
                self.as_mut_ptr(),
                img.as_ptr() as *mut c_void,
                tags.as_mut_ptr(),
                out_img.as_mut_ptr(),
                img.len(),
                tags.len(),
                out_img.len(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok((tags, out_img)),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Performs image detection.
    pub fn image_detection(&mut self, img: &mut [u8]) -> Result<Vec<u8>> {
        let mut out_img = vec![0; img.len()];

        match unsafe {
            ffi::vaccel_image_detection(
                self.as_mut_ptr(),
                img.as_mut_ptr() as *mut c_void,
                out_img.as_mut_ptr(),
                img.len(),
                out_img.len(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(out_img),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Performs image segmentation.
    pub fn image_segmentation(&mut self, img: &mut [u8]) -> Result<Vec<u8>> {
        let mut out_img = vec![0; img.len()];

        match unsafe {
            ffi::vaccel_image_segmentation(
                self.as_mut_ptr(),
                img.as_mut_ptr() as *mut c_void,
                out_img.as_mut_ptr(),
                img.len(),
                out_img.len(),
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(out_img),
            err => Err(Error::Ffi(err)),
        }
    }
}
