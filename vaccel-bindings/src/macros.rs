// SPDX-License-Identifier: Apache-2.0

#![allow(unused_macros)]

/// Implements Handle with minimal boilerplate.
macro_rules! impl_component_handle {
    (
        $struct_name:ident $(<$($generic:ident),+ $(,)?>)?,
        $c_type:ty,
        $ptr_field:ident,
        $owned_field:ident
        $(, extra_fields: { $($extra_field:ident : $extra_value:expr),* $(,)? })?
        $(, extra_vec_fields: { $($extra_vec_field:ident : $extra_vec_value:expr),* $(,)? })?
        $(, where: $($where_clause:tt)*)?
    ) => {
        impl$(<$($generic),+>)? crate::handle::Handle for $struct_name$(<$($generic),+>)?
        $(where $($where_clause)*)?
        {
            type CType = $c_type;

            fn as_ptr(&self) -> *const Self::CType {
                self.$ptr_field.as_ptr() as *const Self::CType
            }

            fn as_mut_ptr(&mut self) -> *mut Self::CType {
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
                        $(
                            $(
                                $extra_vec_field: $extra_vec_value,
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
                        $(
                            $(
                                $extra_vec_field: $extra_vec_value,
                            )*
                        )?
                    })
                    .ok_or(Error::EmptyValue)
            }

            fn into_ptr(mut self) -> Result<*mut Self::CType> {
                if !self.is_owned() {
                    return Err(Error::ConversionFailed(
                        "Cannot get owned pointer from unowned data".to_string(),
                    ));
                }

                // Validate fields in extra_vec_fields section
                $(
                    $(
                        if let Some(ref vec) = self.$extra_vec_field {
                            if !vec.is_empty() {
                                return Err(Error::ConversionFailed(
                                    "Cannot convert a struct with Rust-owned data into a pointer".to_string()
                                ));
                            }
                        }
                    )*
                )?

                let ptr = self.as_mut_ptr();
                let self_ = self.set_owned(false);
                std::mem::forget(self_);
                Ok(ptr)
            }
        }
    };
}

/// Implements Drop for components with owned data.
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

/// Implements a complete component with Handle and Drop.
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
