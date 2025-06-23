// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Handle, Result};
use std::{
    ffi::{c_char, c_uint, CString},
    ptr::{self, NonNull},
};

/// Wrapper for the `struct vaccel_config` C object.
#[derive(Debug)]
pub struct Config {
    inner: NonNull<ffi::vaccel_config>,
    owned: bool,
}

// inner (`*ffi::vaccel_config`) is only accessed through this (Config) struct's
// methods and does not access TLS variables or global state so this should be
// safe
unsafe impl Send for Config {}

impl Config {
    /// Creates a new `Config`.
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

        let mut ptr: *mut ffi::vaccel_config = ptr::null_mut();
        match unsafe {
            ffi::vaccel_config_new(
                &mut ptr,
                plugins_ptr,
                log_level as c_uint,
                log_file_ptr,
                profiling_enabled,
                version_ignore,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        unsafe { Self::from_ptr_owned(ptr) }
    }

    /// Creates a new `Config` from environment variables.
    pub fn from_env() -> Result<Self> {
        let mut ptr: *mut ffi::vaccel_config = ptr::null_mut();
        match unsafe { ffi::vaccel_config_from_env(&mut ptr) as u32 } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        unsafe { Self::from_ptr_owned(ptr) }
    }
}

impl_component_drop!(Config, vaccel_config_delete, inner, owned);

impl_component_handle!(Config, ffi::vaccel_config, inner, owned);
