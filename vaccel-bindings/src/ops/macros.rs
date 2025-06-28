// SPDX-License-Identifier: Apache-2.0

/// Implements TensorType and data type enum methods for Rust types
macro_rules! impl_tensor_types {
    ($enum_type:ty; $($type:ty => $variant:ident),* $(,)?) => {
        // Generate TensorType implementations
        $(
            impl TensorType for $type {
                fn data_type() -> $enum_type {
                    <$enum_type>::$variant
                }
                fn one() -> Self {
                    <$type as num_traits::One>::one()
                }
                fn zero() -> Self {
                    <$type as num_traits::Zero>::zero()
                }
            }
        )*

        // Generate data type enum  methods using the same mapping
        impl $enum_type {
            /// Returns the size of the corresponding Rust type in bytes
            pub fn size_of(&self) -> usize {
                match self {
                    $(<$enum_type>::$variant => std::mem::size_of::<$type>(),)*
                    // For any variant not covered above, return 0 or panic
                    _ => 0, // or panic!("No size defined for {:?}", self)
                }
            }

            /// Returns the name of the corresponding Rust type
            pub fn type_name(&self) -> &'static str {
                match self {
                    $(<$enum_type>::$variant => stringify!($type),)*
                    _ => "unknown",
                }
            }

            /// Returns true if there is a corresponding Rust type
            pub fn has_type(&self) -> bool {
                match self {
                    $(<$enum_type>::$variant => true,)*
                    _ => false,
                }
            }

            /// Returns the size of the corresponding Rust type in bytes if
            /// there is one
            pub fn try_size_of(&self) -> Option<usize> {
                match self {
                    $(<$enum_type>::$variant => Some(std::mem::size_of::<$type>()),)*
                    _ => None,
                }
            }
        }
    };
}
