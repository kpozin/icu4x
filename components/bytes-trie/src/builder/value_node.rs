use {
    super::{
        branch_head_node::BranchHeadNode,
        builder::{BytesTrieBuilder, BytesTrieNodeTree, BytesTrieWriter},
        dynamic_branch_node::DynamicBranchNode,
        errors::BytesTrieBuilderError,
        final_value_node::FinalValueNode,
        intermediate_value_node::IntermediateValueNode,
        linear_match_node::LinearMatchNode,
        node::{Node, NodeContentTrait, NodeInternal},
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

impl NodeContentTrait for ValueNode {
    fn register(&mut self, node: &Node, tree: &mut BytesTrieNodeTree) -> Node {
        <NodeInternal as NodeContentTrait>::register(self, node, tree)
    }

    fn write(&mut self, node: &Node, writer: &mut BytesTrieWriter) {
        match self {
            ValueNode::FinalValue(n) => NodeContentTrait::write(n, node, writer),
            ValueNode::BranchHead(n) => NodeContentTrait::write(n, node, writer),
            ValueNode::DynamicBranch(n) => NodeContentTrait::write(n, node, writer),
            ValueNode::IntermediateValue(n) => NodeContentTrait::write(n, node, writer),
            ValueNode::LinearMatch(n) => NodeContentTrait::write(n, node, writer),
        }
    }
}

pub(crate) trait ValueNodeTrait: NodeContentTrait {
    // TODO: constructors

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
        s: &[u16],
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
    fn write(&mut self, writer: &mut BytesTrieWriter) {
        let offset = writer.write_value_and_final(self.value().unwrap(), true);
        self.set_offset(offset);
    }
}

impl ValueNodeTrait for ValueNode {
    fn value(&self) -> Option<i32> {
        match self {
            ValueNode::FinalValue(n) => n.value(),
            ValueNode::BranchHead(n) => n.value(),
            ValueNode::DynamicBranch(n) => n.value(),
            ValueNode::IntermediateValue(n) => n.value(),
            ValueNode::LinearMatch(n) => n.value(),
        }
    }

    fn set_value(&mut self, value: i32) {
        match self {
            ValueNode::FinalValue(n) => n.set_value(value),
            ValueNode::BranchHead(n) => n.set_value(value),
            ValueNode::DynamicBranch(n) => n.set_value(value),
            ValueNode::IntermediateValue(n) => n.set_value(value),
            ValueNode::LinearMatch(n) => n.set_value(value),
        }
    }

    fn set_final_value(&mut self, value: i32) -> Option<i32> {
        match self {
            ValueNode::FinalValue(n) => n.set_final_value(value),
            ValueNode::BranchHead(n) => n.set_final_value(value),
            ValueNode::DynamicBranch(n) => n.set_final_value(value),
            ValueNode::IntermediateValue(n) => n.set_final_value(value),
            ValueNode::LinearMatch(n) => n.set_final_value(value),
        }
    }

    fn clear_value(&mut self) {
        match self {
            ValueNode::FinalValue(n) => n.clear_value(),
            ValueNode::BranchHead(n) => n.clear_value(),
            ValueNode::DynamicBranch(n) => n.clear_value(),
            ValueNode::IntermediateValue(n) => n.clear_value(),
            ValueNode::LinearMatch(n) => n.clear_value(),
        }
    }
}

impl From<ValueNode> for NodeInternal {
    fn from(node: ValueNode) -> Self {
        match node {
            ValueNode::FinalValue(n) => NodeInternal::FinalValue(n).into(),
            ValueNode::BranchHead(n) => NodeInternal::BranchHead(n).into(),
            ValueNode::DynamicBranch(n) => NodeInternal::DynamicBranch(n).into(),
            ValueNode::IntermediateValue(n) => NodeInternal::IntermediateValue(n).into(),
            ValueNode::LinearMatch(n) => NodeInternal::LinearMatch(n).into(),
        }
    }
}

impl From<ValueNode> for Node {
    fn from(node: ValueNode) -> Self {
        let node: NodeInternal = node.into();
        node.into()
    }
}

macro_rules! impl_value_node_trait {
    ($variant:ident) => {
        impl ValueNodeTrait for $variant {
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

impl_value_node_trait!(FinalValueNode);
impl_value_node_trait!(BranchHeadNode);
impl_value_node_trait!(DynamicBranchNode);
impl_value_node_trait!(IntermediateValueNode);
impl_value_node_trait!(LinearMatchNode);
