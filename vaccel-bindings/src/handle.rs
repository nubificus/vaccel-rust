#![allow(unused_macros)]

use crate::Result;

/// Core trait for all vaccel components with pointer-based ownership
pub trait Handle: Sized {
    /// The underlying C type this handle wraps
    type CType;

    /// Get the raw pointer for FFI calls (immutable access)
    fn as_ptr(&self) -> *const Self::CType;

    /// Get the raw mutable pointer for FFI calls that require mutation
    fn as_mut_ptr(&self) -> *mut Self::CType;

    /// Check if this handle owns the underlying resource
    fn is_owned(&self) -> bool;

    /// Set ownership flag - internal helper for default implementations
    #[doc(hidden)]
    fn set_owned(self, owned: bool) -> Self;

    /// Create borrowed handle from raw pointer (default behavior)
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to the C type
    /// - Caller retains ownership of the underlying resource
    /// - Resource must remain valid for the lifetime of this handle
    unsafe fn from_ptr(ptr: *mut Self::CType) -> Result<Self>;

    /// Create owned handle from raw pointer
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to the C type
    /// - Caller transfers ownership to this handle
    unsafe fn from_ptr_owned(ptr: *mut Self::CType) -> Result<Self>;

    /// Create borrowed handle from reference
    /// Default implementation
    ///
    /// # Safety
    /// - Caller retains ownership of the underlying resource
    /// - Resource must remain valid for the lifetime of this handle
    unsafe fn from_ref(ref_: &Self::CType) -> Result<Self> {
        let ptr = ref_ as *const _ as *mut _;
        Self::from_ptr(ptr)
    }

    /// Take ownership of the underlying resource
    /// Default implementation for simple owned flag toggle
    fn take_ownership(self) -> Self {
        self.set_owned(true)
    }

    /// Release ownership and return raw pointer
    /// Default implementation
    fn into_ptr(self) -> *mut Self::CType {
        let ptr = self.as_mut_ptr();
        let owned_self = self.set_owned(false);
        std::mem::forget(owned_self);
        ptr
    }
}

/// Extension trait for handles that can create borrowed references
pub trait Borrowable: Handle {
    /// Borrowed reference type
    type BorrowedRef<'a>: Handle<CType = Self::CType>
    where
        Self: 'a;

    /// Create a borrowed reference that can't outlive self
    fn as_ref(&self) -> Self::BorrowedRef<'_>;
}

/// Macro to implement Handle with minimal boilerplate
macro_rules! impl_component_handle {
    (
        $struct_name:ident $(<$($generic:ident),+ $(,)?>)?,
        $c_type:ty,
        $ptr_field:ident,
        $owned_field:ident
        $(, extra_fields: { $($extra_field:ident : $extra_value:expr),* $(,)? })?
        $(, where: $($where_clause:tt)*)?
    ) => {
        impl$(<$($generic),+>)? crate::handle::Handle for $struct_name$(<$($generic),+>)?
        $(where $($where_clause)*)?
        {
            type CType = $c_type;

            fn as_ptr(&self) -> *const Self::CType {
                self.$ptr_field.as_ptr() as *const Self::CType
            }

            fn as_mut_ptr(&self) -> *mut Self::CType {
                self.$ptr_field.as_ptr()
            }

            fn is_owned(&self) -> bool {
                self.$owned_field
            }

            fn set_owned(mut self, owned: bool) -> Self {
                self.$owned_field = owned;
                self
            }

            unsafe fn from_ptr(ptr: *mut Self::CType) -> Result<Self> {
                std::ptr::NonNull::new(ptr)
                    .map(|$ptr_field| $struct_name {
                        $ptr_field,
                        $owned_field: false,
                        $(
                            $(
                                $extra_field: $extra_value,
                            )*
                        )?
                    })
                    .ok_or(Error::EmptyValue)
            }

            unsafe fn from_ptr_owned(ptr: *mut Self::CType) -> Result<Self> {
                std::ptr::NonNull::new(ptr)
                    .map(|$ptr_field| $struct_name {
                        $ptr_field,
                        $owned_field: true,
                        $(
                            $(
                                $extra_field: $extra_value,
                            )*
                        )?
                    })
                    .ok_or(Error::EmptyValue)
            }
        }
    };
}

/// Macro to implement Drop for owned resources
macro_rules! impl_component_drop {
    (
        $struct_name:ident $(<$($generic:ident),+ $(,)?>)?,
        $drop_fn:ident,
        $ptr_field:ident,
        $owned_field:ident
        $(, where: $($where_clause:tt)*)?
    ) => {
        impl$(<$($generic),+>)? Drop for $struct_name$(<$($generic),+>)?
        $(where $($where_clause)*)?
        {
            fn drop(&mut self) {
                if self.$owned_field {
                    let ret = unsafe { ffi::$drop_fn(self.$ptr_field.as_ptr()) } as u32;
                    if ret != ffi::VACCEL_OK {
                        log::warn!(
                            "Could not delete {} inner: {}",
                            stringify!($struct_name),
                            ret
                        );
                    }
                }
            }
        }
    };
}

/// Macro for complete component implementation
macro_rules! define_component {
    (
        $(#[$attr:meta])*
        $struct_name:ident $(<$($generic:ident),+ $(,)?>)?,
        $c_type:ty,
        drop: $drop_fn:ident
        $(, extra_fields: { $($field_name:ident : $field_type:ty = $field_value:expr),* $(,)? })?
        $(, where: $($where_clause:tt)*)?
    ) => {
        $(#[$attr])*
        pub struct $struct_name$(<$($generic),+>)?
        $(where $($where_clause)*)?
        {
            ptr: std::ptr::NonNull<$c_type>,
            owned: bool,
            $(
                $(
                    $field_name: $field_type,
                )*
            )?
        }

        impl_component_handle!(
            $struct_name$(<$($generic),+>)?,
            $c_type,
            ptr,
            owned
            $(, extra_fields: { $($field_name: $field_value),* })?
            $(, where: $($where_clause)*)?
        );

        impl_component_drop!(
            $struct_name$(<$($generic),+>)?,
            $drop_fn,
            ptr,
            owned
            $(, where: $($where_clause)*)?
        );

        unsafe impl$(<$($generic),+>)? Send for $struct_name$(<$($generic),+>)?
        $(where $($where_clause)*)?
        {}
    };
}

/// Implements `Borrowable` for a given type and its reference wrapper
macro_rules! impl_component_borrowable {
    (
        $struct_name:ident,
        $ref_struct:ident
    ) => {
        impl Borrowable for $struct_name {
            type BorrowedRef<'a> = $ref_struct<'a>;

            fn as_ref(&self) -> Self::BorrowedRef<'_> {
                $ref_struct {
                    inner: self.ptr,
                    _marker: std::marker::PhantomData,
                }
            }
        }
    };
}

macro_rules! impl_borrowable_handle {
    (
        $struct_name:ident,
        $c_type:ty,
        $ptr_field:ident,
        $owned_field:ident
    ) => {
        impl<'a> crate::handle::Handle for $struct_name<'a> {
            type CType = $c_type;

            fn as_ptr(&self) -> *const Self::CType {
                self.$ptr_field.as_ptr() as *const Self::CType
            }

            fn as_mut_ptr(&self) -> *mut Self::CType {
                self.$ptr_field.as_ptr()
            }

            fn is_owned(&self) -> bool {
                false
            }

            fn set_owned(mut self, owned: bool) -> Self {
                self
            }

            unsafe fn from_ptr(ptr: *mut Self::CType) -> Result<Self> {
                std::ptr::NonNull::new(ptr)
                    .map(|$ptr_field| $struct_name {
                        $ptr_field,
                        _marker: std::marker::PhantomData,
                    })
                    .ok_or(Error::EmptyValue)
            }

            unsafe fn from_ptr_owned(ptr: *mut Self::CType) -> Result<Self> {
                Self::from_ptr(ptr)
            }
        }
    };
}

macro_rules! define_component_borrowable {
    (
        $(#[$attr:meta])*
        $struct_name:ident,
        $ref_struct:ident,
        $c_type:ty,
    ) => {
        $(#[$attr])*
        pub struct $ref_struct<'a> {
            inner: std::ptr::NonNull<$c_type>,
            _marker: std::marker::PhantomData<&'a $struct_name>,
        }

        impl_borrowable_handle!($struct_name, $c_type, ptr, owned);
        impl_component_borrowable!($struct_name, $ref_struct, ptr);
    };
}
