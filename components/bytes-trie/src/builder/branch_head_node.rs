use {
    super::{
        builder::{BytesTrieBuilder, BytesTrieBuilderCommon, BytesTrieNodeTree, BytesTrieWriter},
        node::{Node, NodeContentTrait, NodeInternal},
        value_node::{ValueNode, ValueNodeContentTrait},
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
            node.set_offset(edge_number);
        }
        edge_number
    }

    fn write(&mut self, node: &Node, writer: &mut BytesTrieWriter) {
        self.next.write(writer);
        let length = self.length;
        let offset = if length <= writer.min_linear_match() as i32 {
            writer.write_value_and_type(self.value(), self.length - 1)
        } else {
            writer.write_unit((length - 1).try_into().unwrap());
            writer.write_value_and_type(self.value(), 0)
        };
        // TODO(kpozin): Check expected type
        node.set_offset(offset.try_into().unwrap());
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
