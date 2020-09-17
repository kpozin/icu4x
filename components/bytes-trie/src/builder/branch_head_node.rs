use {
    super::{
        builder::BytesTrieBuilder,
        node::{Node, NodeTrait, RcNode, WithOffset},
        value_node::{ValueNode, ValueNodeTrait},
    },
    std::{cell::RefCell, convert::TryInto, rc::Rc},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct BranchHeadNode {
    pub(crate) offset: i32,
    pub(crate) value: Option<i32>,
    length: i32,
    next: RcNode,
}

impl NodeTrait for BranchHeadNode {
    fn register(self_: &RcNode, builder: &mut BytesTrieBuilder) -> RcNode {
        unimplemented!()
    }

    fn mark_right_edges_first(&mut self, mut edge_number: i32) -> i32 {
        if self.offset() == 0 {
            edge_number = self.next.borrow_mut().mark_right_edges_first(edge_number);
            self.offset = edge_number;
        }
        edge_number
    }

    fn write(&mut self, builder: &mut BytesTrieBuilder) {
        self.next.borrow_mut().write(builder);
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
    pub fn new(length: i32, sub_node: RcNode) -> BranchHeadNode {
        BranchHeadNode {
            offset: 0,
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
