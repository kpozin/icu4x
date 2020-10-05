use {
    super::{
        builder::{BytesTrieBuilder, BytesTrieNodeTree, BytesTrieWriter},
        node::{Node, NodeContentTrait, NodeInternal},
        value_node::{ValueNode, ValueNodeTrait},
    },
    std::rc::Rc,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct IntermediateValueNode {
    pub(crate) value: Option<i32>,
    next: Node,
}

impl NodeContentTrait for IntermediateValueNode {
    fn mark_right_edges_first(&mut self, edge_number: i32) -> i32 {
        if self.offset() == 0 {
            let offset = self.next.borrow_mut().mark_right_edges_first(edge_number);
            self.set_offset(edge_number);
            offset
        } else {
            edge_number
        }
    }

    fn write(&mut self, builder: &mut BytesTrieWriter) {
        self.next.borrow_mut().write(builder);
        self.set_offset(builder.write_value_and_final(self.value().unwrap(), false))
    }
}

impl IntermediateValueNode {
    pub fn new(value: i32, next_node: Node) -> Self {
        IntermediateValueNode {
            value: Some(value),
            next: next_node,
        }
    }
}
