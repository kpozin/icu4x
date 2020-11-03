use {
    super::{
        branch_head_node::BranchHeadNode,
        builder::{BytesTrieBuilder, BytesTrieNodeTree, BytesTrieWriter},
        dynamic_branch_node::DynamicBranchNode,
        errors::BytesTrieBuilderError,
        final_value_node::FinalValueNode,
        intermediate_value_node::IntermediateValueNode,
        linear_match_node::LinearMatchNode,
        node::{Node, NodeContent, NodeContentTrait, NodeInternal},
    },
    std::{cell::RefCell, fmt::Debug, hash::Hash, rc::Rc},
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) enum ValueNode {
    FinalValue(FinalValueNode),
    BranchHead(BranchHeadNode),
    DynamicBranch(DynamicBranchNode),
    IntermediateValue(IntermediateValueNode),
    LinearMatch(LinearMatchNode),
}

pub(crate) trait ValueNodeContentTrait: NodeContentTrait {
    fn value(&self) -> Option<i32>;
    /// Will panic if the node already has a value.
    fn set_value(&mut self, value: i32);
    /// Same as `set_value`, but does not panic if the node already has a value.
    ///
    /// Returns the previous value, if any.
    fn set_final_value(&mut self, value: i32) -> Option<i32>;
    fn has_value(&self) -> bool {
        self.value().is_some()
    }
    fn clear_value(&mut self);

    // Used in FinalValueNode, BranchHeadNode, IntermediateValueNode,
    fn add(
        &mut self,
        node: &Node,
        builder: &mut BytesTrieBuilder,
        s: &[u8],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        if s.is_empty() {
            return Err(BytesTrieBuilderError::DuplicateString);
        }
        // Replace self with a node for the remaining string suffix and value.
        let mut node = builder.create_suffix_node(s, value);
        node.set_value(value);
        Ok(node.into())
    }

    // Used in FinalValueNode, DynamicBranchNode.
    fn write(&mut self, node: &Node, writer: &mut BytesTrieWriter) {
        let offset = writer.write_value_and_final(self.value().unwrap(), true);
        node.set_offset(offset as i32);
    }
}

trait ValueNodeTrait {
    fn value(&self) -> Option<i32>;
    fn set_value(&self, value: i32);
}

impl ValueNodeTrait for Node {
    fn value(&self) -> Option<i32> {
        match &*self.content() {
            NodeContent::FinalValue(n) => n.value(),
            NodeContent::BranchHead(n) => n.value(),
            NodeContent::DynamicBranch(n) => n.value(),
            NodeContent::IntermediateValue(n) => n.value(),
            NodeContent::LinearMatch(n) => n.value(),
            _ => panic!("Attempted to get value from non-value node"),
        }
    }

    fn set_value(&self, value: i32) {
        match &mut *self.content_mut() {
            NodeContent::FinalValue(n) => n.set_value(value),
            NodeContent::BranchHead(n) => n.set_value(value),
            NodeContent::DynamicBranch(n) => n.set_value(value),
            NodeContent::IntermediateValue(n) => n.set_value(value),
            NodeContent::LinearMatch(n) => n.set_value(value),
            _ => panic!("Attempt to set value on a non-value node"),
        }
    }
}

// impl ValueNodeTrait for ValueNode {
//     fn value(&self) -> Option<i32> {
//         match self {
//             ValueNode::FinalValue(n) => n.value(),
//             ValueNode::BranchHead(n) => n.value(),
//             ValueNode::DynamicBranch(n) => n.value(),
//             ValueNode::IntermediateValue(n) => n.value(),
//             ValueNode::LinearMatch(n) => n.value(),
//         }
//     }

//     fn set_value(&mut self, value: i32) {
//         match self {
//             ValueNode::FinalValue(n) => n.set_value(value),
//             ValueNode::BranchHead(n) => n.set_value(value),
//             ValueNode::DynamicBranch(n) => n.set_value(value),
//             ValueNode::IntermediateValue(n) => n.set_value(value),
//             ValueNode::LinearMatch(n) => n.set_value(value),
//         }
//     }

//     fn set_final_value(&mut self, value: i32) -> Option<i32> {
//         match self {
//             ValueNode::FinalValue(n) => n.set_final_value(value),
//             ValueNode::BranchHead(n) => n.set_final_value(value),
//             ValueNode::DynamicBranch(n) => n.set_final_value(value),
//             ValueNode::IntermediateValue(n) => n.set_final_value(value),
//             ValueNode::LinearMatch(n) => n.set_final_value(value),
//         }
//     }

//     fn clear_value(&mut self) {
//         match self {
//             ValueNode::FinalValue(n) => n.clear_value(),
//             ValueNode::BranchHead(n) => n.clear_value(),
//             ValueNode::DynamicBranch(n) => n.clear_value(),
//             ValueNode::IntermediateValue(n) => n.clear_value(),
//             ValueNode::LinearMatch(n) => n.clear_value(),
//         }
//     }
// }

impl From<ValueNode> for NodeInternal {
    fn from(node: ValueNode) -> Self {
        match node {
            ValueNode::FinalValue(n) => NodeContent::FinalValue(n).into(),
            ValueNode::BranchHead(n) => NodeContent::BranchHead(n).into(),
            ValueNode::DynamicBranch(n) => NodeContent::DynamicBranch(n).into(),
            ValueNode::IntermediateValue(n) => NodeContent::IntermediateValue(n).into(),
            ValueNode::LinearMatch(n) => NodeContent::LinearMatch(n).into(),
        }
    }
}

impl From<ValueNode> for Node {
    fn from(node: ValueNode) -> Self {
        let node: NodeInternal = node.into();
        node.into()
    }
}

macro_rules! impl_value_node_content_trait {
    ($variant:ident) => {
        impl ValueNodeContentTrait for $variant {
            fn value(&self) -> Option<i32> {
                self.value
            }

            fn set_value(&mut self, value: i32) {
                assert!(!self.has_value());
                self.value.replace(value);
            }

            fn set_final_value(&mut self, value: i32) -> Option<i32> {
                self.value.replace(value)
            }

            fn clear_value(&mut self) {
                self.value = None;
            }
        }
    };
}

impl_value_node_content_trait!(FinalValueNode);
impl_value_node_content_trait!(BranchHeadNode);
impl_value_node_content_trait!(DynamicBranchNode);
impl_value_node_content_trait!(IntermediateValueNode);
impl_value_node_content_trait!(LinearMatchNode);
