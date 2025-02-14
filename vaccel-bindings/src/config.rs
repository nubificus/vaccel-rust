// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Result};
use log::warn;
use std::{
    ffi::{c_char, c_uint, CString},
    ptr,
};

#[derive(Debug)]
pub struct Config {
    inner: ffi::vaccel_config,
    initialized: bool,
}

// inner (`ffi::vaccel_config`) is only accessed through this (Config) struct's
// methods and does not access TLS variables or global state so this should be
// safe
unsafe impl Send for Config {}

impl Config {
    /// Create a new resource object
    pub fn new(
        plugins: Option<&str>,
        log_level: u8,
        log_file: Option<&str>,
        profiling_enabled: bool,
        version_ignore: bool,
    ) -> Result<Self> {
        let plugins_cstr: CString;
        let plugins_ptr: *const c_char;
        match plugins {
            Some(p) => {
                plugins_cstr = CString::new(p).map_err(|e| {
                    Error::ConversionFailed(format!(
                        "Could not convert `plugins` to `CString` [{}]",
                        e
                    ))
                })?;
                plugins_ptr = plugins_cstr.as_c_str().as_ptr();
            }
            None => {
                plugins_ptr = ptr::null();
            }
        };

        let log_file_cstr: CString;
        let log_file_ptr: *const c_char;
        match log_file {
            Some(l) => {
                log_file_cstr = CString::new(l).map_err(|e| {
                    Error::ConversionFailed(format!(
                        "Could not convert `log_file` to `CString` [{}]",
                        e
                    ))
                })?;
                log_file_ptr = log_file_cstr.as_c_str().as_ptr();
            }
            None => {
                log_file_ptr = ptr::null();
            }
        };

        let mut inner = ffi::vaccel_config::default();
        match unsafe {
            ffi::vaccel_config_init(
                &mut inner,
                plugins_ptr,
                log_level as c_uint,
                log_file_ptr,
                profiling_enabled,
                version_ignore,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(Config {
                inner,
                initialized: true,
            }),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Create new config from environment variables
    pub fn from_env() -> Result<Self> {
        let mut inner = ffi::vaccel_config::default();
        match unsafe { ffi::vaccel_config_init_from_env(&mut inner) as u32 } {
            ffi::VACCEL_OK => Ok(Config {
                inner,
                initialized: true,
            }),
            err => Err(Error::Ffi(err)),
        }
    }

    /// Returns `true` if the config has been initialized
    pub fn initialized(&self) -> bool {
        self.initialized
    }

    /// Release config data
    pub fn release(&mut self) -> Result<()> {
        if !self.initialized {
            return Err(Error::Uninitialized);
        }

        match unsafe { ffi::vaccel_config_release(&mut self.inner) as u32 } {
            ffi::VACCEL_OK => {
                self.initialized = false;
                Ok(())
            }
            err => Err(Error::Ffi(err)),
        }
    }

    // Warning: Do not copy internal raw pointers
    pub(crate) fn inner(&self) -> &ffi::vaccel_config {
        &self.inner
    }

    // Warning: Do not copy internal raw pointers
    pub(crate) fn inner_mut(&mut self) -> &mut ffi::vaccel_config {
        &mut self.inner
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        if self.initialized && self.release().is_err() {
            warn!("Could not release config");
        }
    }
}
