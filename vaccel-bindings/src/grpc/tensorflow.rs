use memchr::memchr;
use std::convert::From;
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;

use protobuf::ProtobufEnum;
use protocols::tensorflow::{TFDataType, TFNode, TFTensor};

use crate::ffi::{vaccel_tf_buffer, vaccel_tf_node, vaccel_tf_tensor};

/// Convert a `vaccel_tf_buffer` in a slice of `u8`
///
/// The owner of the `vaccel_tf_buffer` is responsible for ensuring
/// that it outlives the data array
impl From<&'_ vaccel_tf_buffer> for &'_ [u8] {
    fn from(buff: &'_ vaccel_tf_buffer) -> Self {
        unsafe { std::slice::from_raw_parts(buff.data as *const u8, buff.size as usize) }
    }
}

/// Convert a slice of `u8` intο a vaccel_tf_buffer
///
/// The owner of the slice is responsible for making sure that
/// the underlying memory is responsible for making sure that it
/// outlives the `vaccel_tf_buffer`
impl From<&'_ mut [u8]> for vaccel_tf_buffer {
    fn from(slice: &'_ mut [u8]) -> Self {
        vaccel_tf_buffer {
            data: slice.as_mut_ptr() as *mut c_void,
            size: slice.len() as u64,
        }
    }
}

/// Convert a `TFNode` to a `vaccel_tf_node`
///
/// TFNode contains the name of the node as a `std::string::String` which
/// is not guaranteed to be null-terminated. This will modify the TFNode::name,
/// in place, to be null-terminated, if needed, before performing the conversion.
/// The owner of the `TFNode` is responsible for ensuring that it will outlive
/// the `vaccel_tf_node`
impl From<&mut TFNode> for vaccel_tf_node {
    fn from(node: &mut TFNode) -> Self {
        match memchr(0, node.get_name().as_bytes()) {
            Some(_) => {}
            None => {
                node.mut_name().push_str("\0");
            }
        }

        vaccel_tf_node {
            name: node.mut_name().as_mut_ptr() as *mut c_char,
            id: node.get_id(),
        }
    }
}

/// Convert a `vaccel_tf_node` to `TFNode`
///
/// This will clone the `name` so we do not take ownership
/// of the C-side memory holding the data.
impl From<&vaccel_tf_node> for TFNode {
    fn from(node: &vaccel_tf_node) -> Self {
        let c_str = unsafe { CStr::from_ptr(node.name) };

        // This clones, so that we do not take ownership
        // of the C-side string
        let name = c_str.to_str().unwrap_or("").to_owned();

        TFNode {
            name,
            id: node.id,
            ..Default::default()
        }
    }
}

/// Convert a `TFTensor` into a `vaccel_tf_tensor`
///
/// This creates a `vaccel_tf_tensor` which points to the data of
/// the initial `TFTensor`. The owner of the `TFTensor` is responsible
/// for ensuring that it outlives the `vaccel_tf_tensor`.
impl From<&mut TFTensor> for vaccel_tf_tensor {
    fn from(tensor: &mut TFTensor) -> Self {
        let data = &mut tensor.data;
        let dims = &mut tensor.dims;

        vaccel_tf_tensor {
            data: data.as_mut_ptr() as *mut c_void,
            size: data.len() as u64,
            dims: dims.as_mut_ptr(),
            nr_dims: dims.len() as i32,
            data_type: tensor.get_field_type() as u32,
        }
    }
}

/// Convert a `vaccel_tf_tensor` into a `TFTensor`
///
/// This will clone the memory of the `data` and `dims` pointers
/// into the respective `TFTensor` types, to avoid taking ownership
/// of the C-side memory holding the data.
impl From<&vaccel_tf_tensor> for TFTensor {
    fn from(node: &vaccel_tf_tensor) -> Self {
        let data = match node.data.is_null() {
            true => &[],
            false => unsafe {
                std::slice::from_raw_parts(node.data as *mut u8, node.size as usize)
            },
        };

        let dims = match node.dims.is_null() {
            true => &[],
            false => unsafe { std::slice::from_raw_parts(node.dims, node.nr_dims as usize) },
        };

        let field_type = TFDataType::from_i32(node.data_type as i32).unwrap_or(TFDataType::UNUSED);

        TFTensor {
            data: data.to_owned(),
            dims: dims.to_owned(),
            field_type,
            ..Default::default()
        }
    }
}

/// Convert a `vaccel_tf_tensor` into a `TFTensor`
///
/// This will clone the memory of the `data` and `dims` pointers
/// into the respective `TFTensor` types, to avoid taking ownership
/// of the C-side memory holding the data.
impl From<vaccel_tf_tensor> for TFTensor {
    fn from(node: vaccel_tf_tensor) -> Self {
        let data = match node.data.is_null() {
            true => &[],
            false => unsafe {
                std::slice::from_raw_parts(node.data as *mut u8, node.size as usize)
            },
        };

        let dims = match node.dims.is_null() {
            true => &[],
            false => unsafe { std::slice::from_raw_parts(node.dims, node.nr_dims as usize) },
        };

        let field_type = TFDataType::from_i32(node.data_type as i32).unwrap_or(TFDataType::UNUSED);

        TFTensor {
            data: data.to_owned(),
            dims: dims.to_owned(),
            field_type,
            ..Default::default()
        }
    }
}
