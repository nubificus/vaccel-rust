use crate::ffi;
use crate::{Error, Result};

use protocols::tensorflow::TFNode;

use std::ffi::{CStr, CString};
use std::fmt;

pub struct Node {
    inner: *mut ffi::vaccel_tf_node,
}

impl Node {
    pub fn new(name: &str, id: i64) -> Self {
        let name = CString::new(name)
            .expect("Invalid TensorFlow node name")
            .into_raw();

        let inner = unsafe { ffi::vaccel_tf_node_new(name, id) };
        assert!(!inner.is_null(), "Memory allocation failure");

        Node { inner }
    }

    pub unsafe fn from_vaccel_node(node: *mut ffi::vaccel_tf_node) -> Result<Self> {
        let name = ffi::vaccel_tf_node_get_name(node);
        if name.is_null() {
            return Err(Error::InvalidArgument);
        }

        Ok(Node { inner: node })
    }

    pub fn id(&self) -> i64 {
        unsafe { ffi::vaccel_tf_node_get_id(self.inner) }
    }

    pub fn name(&self) -> String {
        let cmsg = unsafe { CStr::from_ptr(ffi::vaccel_tf_node_get_name(self.inner)) };
        cmsg.to_str().unwrap_or("").to_owned()
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.name(), self.id())
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
        unsafe { ffi::vaccel_tf_node_destroy(self.inner) }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Convert a `TFNode` to a `tensorflow::Node`
///
/// This can fail if the creating the underlying node
/// fails.
///
impl From<&TFNode> for Node {
    fn from(node: &TFNode) -> Self {
        Node::new(node.get_name(), node.get_id())
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

impl From<&ffi::vaccel_tf_node> for Node {
    fn from(node: &ffi::vaccel_tf_node) -> Self {
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
