// SPDX-License-Identifier: Apache-2.0

use crate::tf::Node;
use vaccel::{ops::tf::Node as VaccelNode, Error, Result};

impl TryFrom<&Node> for VaccelNode {
    type Error = Error;

    fn try_from(node: &Node) -> Result<Self> {
        Self::new(&node.name, node.id)
    }
}

impl TryFrom<Node> for VaccelNode {
    type Error = Error;

    fn try_from(node: Node) -> Result<Self> {
        Self::try_from(&node)
    }
}

impl TryFrom<&VaccelNode> for Node {
    type Error = Error;

    fn try_from(vaccel: &VaccelNode) -> Result<Self> {
        Ok(Self {
            name: vaccel.name()?,
            id: vaccel.id(),
            ..Default::default()
        })
    }
}

impl TryFrom<VaccelNode> for Node {
    type Error = Error;

    fn try_from(vaccel: VaccelNode) -> Result<Self> {
        Self::try_from(&vaccel)
    }
}
