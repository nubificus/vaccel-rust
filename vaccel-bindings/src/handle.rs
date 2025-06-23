// SPDX-License-Identifier: Apache-2.0

#![allow(unused_macros)]

use crate::{Error, Result};

/// Core trait for all components with pointer-based ownership.
pub trait Handle: Sized {
    /// The underlying C type this handle wraps.
    type CType;

    /// Returns the raw pointer for FFI calls (immutable access).
    fn as_ptr(&self) -> *const Self::CType;

    /// Returns the raw mutable pointer for FFI calls that require mutation.
    fn as_mut_ptr(&mut self) -> *mut Self::CType;

    /// Checks if this handle owns the underlying resource.
    fn is_owned(&self) -> bool;

    /// Sets the ownership flag.
    #[doc(hidden)]
    fn set_owned(self, owned: bool) -> Self;

    /// Creates a borrowed handle from a raw pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to the C type.
    /// - Caller retains ownership of the underlying resource.
    /// - Resource must remain valid for the lifetime of this handle.
    unsafe fn from_ptr(ptr: *mut Self::CType) -> Result<Self>;

    /// Creates an owned handle from a raw pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to the C type.
    /// - Caller transfers ownership to this handle.
    unsafe fn from_ptr_owned(ptr: *mut Self::CType) -> Result<Self>;

    /// Creates a borrowed handle from a reference.
    ///
    /// # Safety
    ///
    /// - Caller retains ownership of the underlying resource.
    /// - Resource must remain valid for the lifetime of this handle.
    unsafe fn from_ref(ref_: &Self::CType) -> Result<Self> {
        let ptr = ref_ as *const _ as *mut _;
        Self::from_ptr(ptr)
    }

    /// Creates a borrowed handle from a mutable reference.
    ///
    /// # Safety
    /// - Caller retains ownership of the underlying resource.
    /// - Resource must remain valid for the lifetime of this handle.
    unsafe fn from_mut_ref(ref_: &mut Self::CType) -> Result<Self> {
        let ptr = ref_ as *mut _;
        Self::from_ptr(ptr)
    }

    /// Takes ownership of the underlying resource.
    fn take_ownership(self) -> Self {
        self.set_owned(true)
    }

    /// Releases ownership and returns a raw pointer.
    fn into_ptr(mut self) -> Result<*mut Self::CType> {
        if !self.is_owned() {
            return Err(Error::ConversionFailed(
                "Cannot get owned pointer from unowned data".to_string(),
            ));
        }

        let ptr = self.as_mut_ptr();
        let self_ = self.set_owned(false);
        std::mem::forget(self_);
        Ok(ptr)
    }
}
