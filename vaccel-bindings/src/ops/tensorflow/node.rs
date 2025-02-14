// SPDX-License-Identifier: Apache-2.0

use crate::{ffi, Error, Result};
use derive_more::Display;
use std::ffi::{CStr, CString};
use vaccel_rpc_proto::tensorflow::TFNode;

#[derive(Debug, Display)]
#[display("{}:{}", self.name(), self.id())]
pub struct Node {
    inner: *mut ffi::vaccel_tf_node,
}

impl Node {
    pub fn new(name: &str, id: i32) -> Result<Self> {
        let name = match CString::new(name) {
            Ok(n) => n.into_raw(),
            Err(e) => {
                return Err(Error::ConversionFailed(format!(
                    "Could not convert `name` to `CString` [{}]",
                    e
                )))?
            }
        };

        let mut inner: *mut ffi::vaccel_tf_node = std::ptr::null_mut();
        match unsafe { ffi::vaccel_tf_node_new(&mut inner, name, id) as u32 } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }
        assert!(!inner.is_null());
        unsafe { assert!(!(*inner).name.is_null()) };

        Ok(Node { inner })
    }

    /// # Safety
    ///
    /// `node` is expected to be a valid pointer to an object allocated
    /// manually or by the respective vAccel function.
    pub unsafe fn from_vaccel_node(node: *mut ffi::vaccel_tf_node) -> Result<Self> {
        if node.is_null() || (*node).name.is_null() {
            return Err(Error::InvalidArgument(
                "`node` and `node.name` cannot be `null`".to_string(),
            ));
        }

        Ok(Node { inner: node })
    }

    pub fn id(&self) -> i32 {
        unsafe { (*self.inner).id }
    }

    pub fn name(&self) -> String {
        let cmsg = unsafe { CStr::from_ptr((*self.inner).name) };
        cmsg.to_str().unwrap_or("").to_owned()
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_tf_node {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_tf_node {
        self.inner
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe { ffi::vaccel_tf_node_delete(self.inner) };
    }
}

/// Convert a `TFNode` to a `tensorflow::Node`
///
/// This can fail if the creating the underlying node
/// fails.
///
impl TryFrom<&TFNode> for Node {
    type Error = Error;

    fn try_from(node: &TFNode) -> Result<Self> {
        Node::new(&node.name, node.id)
    }
}

/// Convert a `tensorflow::Node` to `TFNode`
///
/// This will clone the `name` so we do not take ownership
/// of the C-side memory holding the data.
impl From<&Node> for TFNode {
    fn from(node: &Node) -> Self {
        TFNode {
            name: CString::new(node.name()).unwrap().into_string().unwrap(),
            id: node.id(),
            ..Default::default()
        }
    }
}

impl TryFrom<&ffi::vaccel_tf_node> for Node {
    type Error = Error;

    fn try_from(node: &ffi::vaccel_tf_node) -> Result<Self> {
        let name = unsafe { CStr::from_ptr(node.name) };

        Node::new(name.as_ref().to_str().unwrap(), node.id)
    }
}

impl From<&ffi::vaccel_tf_node> for TFNode {
    fn from(node: &ffi::vaccel_tf_node) -> Self {
        TFNode {
            name: unsafe {
                CStr::from_ptr(node.name)
                    .to_str()
                    .unwrap_or("blah")
                    .to_owned()
            },
            id: node.id,
            ..Default::default()
        }
    }
}
