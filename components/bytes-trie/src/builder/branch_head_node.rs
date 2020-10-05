use {
    super::{
        builder::{BytesTrieBuilder, BytesTrieNodeTree, BytesTrieWriter},
        node::{Node, NodeContentTrait, NodeInternal},
        value_node::{ValueNode, ValueNodeTrait},
    },
    std::{cell::RefCell, convert::TryInto, rc::Rc},
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct BranchHeadNode {
    pub(crate) value: Option<i32>,
    length: i32,
    next: Node,
}

impl NodeContentTrait for BranchHeadNode {
    fn mark_right_edges_first(&mut self, node: &Node, mut edge_number: i32) -> i32 {
        if node.offset() == 0 {
            edge_number = self.next.mark_right_edges_first(edge_number);
            node.offset = edge_number;
        }
        edge_number
    }

    fn write(&mut self, node: &Node, builder: &mut BytesTrieWriter) {
        self.next.write(builder);
        let length = self.length;
        let offset = if length <= builder.min_linear_match() {
            builder.write_value_and_type(self.value(), self.length - 1)
        } else {
            builder.write_unit((length - 1).try_into().unwrap());
            builder.write_value_and_type(self.value(), 0)
        };
        self.set_offset(offset);
    }
}

impl BranchHeadNode {
    pub fn new(length: i32, sub_node: Node) -> BranchHeadNode {
        BranchHeadNode {
            value: None,
            length,
            next: sub_node,
        }
    }
}

impl From<BranchHeadNode> for ValueNode {
    fn from(node: BranchHeadNode) -> Self {
        ValueNode::BranchHead(node)
    }
}
