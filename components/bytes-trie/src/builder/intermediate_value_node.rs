use {
    super::{
        builder::{BytesTrieBuilder, BytesTrieNodeTree},
        node::{Node, NodeTrait, RcNode, WithOffset},
        value_node::{ValueNode, ValueNodeTrait},
    },
    std::rc::Rc,
};

#[derive(Debug, Eq, PartialEq)]

pub(crate) struct IntermediateValueNode {
    pub(crate) offset: i32,
    pub(crate) value: Option<i32>,
    next: RcNode,
}

impl NodeTrait for IntermediateValueNode {
    fn register(self_: &RcNode, tree: &mut BytesTrieNodeTree) -> RcNode {
        <Node as NodeTrait>::register(self_, tree)
    }

    fn mark_right_edges_first(&mut self, edge_number: i32) -> i32 {
        if self.offset() == 0 {
            let offset = self.next.borrow_mut().mark_right_edges_first(edge_number);
            self.set_offset(edge_number);
            offset
        } else {
            edge_number
        }
    }

    fn write(&mut self, builder: &mut super::builder::BytesTrieBuilder) {
        self.next.borrow_mut().write(builder);
        self.set_offset(builder.write_value_and_final(self.value().unwrap(), false))
    }
}

impl IntermediateValueNode {
    pub fn new(value: i32, next_node: RcNode) -> Self {
        IntermediateValueNode {
            offset: 0,
            value: Some(value),
            next: next_node,
        }
    }
}
