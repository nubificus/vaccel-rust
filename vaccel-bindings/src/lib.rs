// SPDX-License-Identifier: Apache-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::slice;

#[macro_use]
mod macros;

pub mod arg;
pub mod blob;
pub mod config;
pub mod error;
pub mod ffi;
pub mod handle;
pub mod ops;
pub mod profiling;
pub mod resource;
pub mod session;
pub mod vaccel;

pub use arg::{Arg, ArgType};
pub use blob::{Blob, BlobType};
pub use config::Config;
pub use error::{Error, Result};
pub use handle::Handle;
pub use resource::{Resource, ResourceType};
pub use session::Session;
pub use vaccel::{bootstrap, bootstrap_with_config, cleanup, is_initialized, VaccelId};

/// Wrapper for `slice::from_raw_parts()` with null pointer checking.
///
/// # Safety
///
/// `buf` must be a valid pointer to an array of objects of type `T` with the provided len.
/// See also: https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety
pub unsafe fn c_pointer_to_slice<'a, T>(buf: *const T, len: usize) -> Option<&'a [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(buf, len) })
    }
}

/// Wrapper for `slice::from_raw_parts_mut()` with null pointer checking.
///
/// # Safety
///
/// `buf` must be a valid pointer to an array of objects of type `T` with the provided len.
/// See also: https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety
pub unsafe fn c_pointer_to_mut_slice<'a, T>(buf: *mut T, len: usize) -> Option<&'a mut [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts_mut(buf, len) })
    }
}
