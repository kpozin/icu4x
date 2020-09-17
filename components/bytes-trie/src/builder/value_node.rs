use {
    super::{
        branch_head_node::BranchHeadNode,
        builder::BytesTrieBuilder,
        dynamic_branch_node::DynamicBranchNode,
        errors::BytesTrieBuilderError,
        final_value_node::FinalValueNode,
        intermediate_value_node::IntermediateValueNode,
        linear_match_node::LinearMatchNode,
        node::{Node, NodeTrait, RcNode, WithOffset},
    },
    std::{cell::RefCell, fmt::Debug, hash::Hash, rc::Rc},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ValueNode {
    FinalValue(FinalValueNode),
    BranchHead(BranchHeadNode),
    DynamicBranch(DynamicBranchNode),
    IntermediateValue(IntermediateValueNode),
    LinearMatch(LinearMatchNode),
}

impl WithOffset for ValueNode {
    fn offset(&self) -> i32 {
        match self {
            ValueNode::FinalValue(n) => n.offset(),
            ValueNode::BranchHead(n) => n.offset(),
            ValueNode::DynamicBranch(n) => n.offset(),
            ValueNode::IntermediateValue(n) => n.offset(),
            ValueNode::LinearMatch(n) => n.offset(),
        }
    }

    fn set_offset(&mut self, offset: i32) {
        todo!()
    }
}

impl NodeTrait for ValueNode {
    fn register(self_: &RcNode, builder: &mut BytesTrieBuilder) -> RcNode {
        <Node as NodeTrait>::register(self_, builder)
    }

    fn write(&mut self, builder: &mut BytesTrieBuilder) {
        match self {
            ValueNode::FinalValue(n) => NodeTrait::write(n, builder),
            ValueNode::BranchHead(n) => NodeTrait::write(n, builder),
            ValueNode::DynamicBranch(n) => NodeTrait::write(n, builder),
            ValueNode::IntermediateValue(n) => NodeTrait::write(n, builder),
            ValueNode::LinearMatch(n) => NodeTrait::write(n, builder),
        }
    }
}

pub(crate) trait ValueNodeTrait: NodeTrait {
    // TODO: constructors

    fn value(&self) -> Option<i32>;
    fn set_value(&mut self, value: i32) -> Option<i32>;
    fn has_value(&self) -> bool {
        self.value().is_some()
    }
    fn clear_value(&mut self);
    // TODO: set_final_value??

    // Used in FinalValueNode, BranchHeadNode, IntermediateValueNode,
    fn add(
        self_: &RcNode,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        start: i32,
        value: i32,
    ) -> Result<RcNode, BytesTrieBuilderError> {
        if start == s.len() as i32 {
            return Err(BytesTrieBuilderError::DuplicateString);
        }
        // Replace self with a node for the remaining string suffix and value.
        let mut node = builder.create_suffix_node(s, start, value);
        node.set_value(value);
        Ok(node.into())
    }

    // Used in FinalValueNode, DynamicBranchNode.
    fn write(&mut self, builder: &mut BytesTrieBuilder) {
        let offset = builder.write_value_and_final(self.value().unwrap(), true);
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

    fn set_value(&mut self, value: i32) -> Option<i32> {
        match self {
            ValueNode::FinalValue(n) => n.set_value(value),
            ValueNode::BranchHead(n) => n.set_value(value),
            ValueNode::DynamicBranch(n) => n.set_value(value),
            ValueNode::IntermediateValue(n) => n.set_value(value),
            ValueNode::LinearMatch(n) => n.set_value(value),
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

impl From<ValueNode> for Node {
    fn from(node: ValueNode) -> Self {
        match node {
            ValueNode::FinalValue(n) => Node::FinalValue(n),
            ValueNode::BranchHead(n) => Node::BranchHead(n),
            ValueNode::DynamicBranch(n) => Node::DynamicBranch(n),
            ValueNode::IntermediateValue(n) => Node::IntermediateValue(n),
            ValueNode::LinearMatch(n) => Node::LinearMatch(n),
        }
    }
}

impl From<ValueNode> for RcNode {
    fn from(node: ValueNode) -> Self {
        let node: Node = node.into();
        node.into()
    }
}

macro_rules! impl_value_node_trait {
    ($variant:ident) => {
        impl ValueNodeTrait for $variant {
            fn value(&self) -> Option<i32> {
                self.value
            }

            fn set_value(&mut self, value: i32) -> Option<i32> {
                assert!(!self.has_value());
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
