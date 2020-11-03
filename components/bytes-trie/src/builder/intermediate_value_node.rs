use {
    super::{
        builder::{BytesTrieBuilder, BytesTrieNodeTree, BytesTrieWriter},
        node::{Node, NodeContentTrait, NodeInternal},
        value_node::{ValueNode, ValueNodeContentTrait},
    },
    std::convert::TryInto,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct IntermediateValueNode {
    pub(crate) value: Option<i32>,
    next: Node,
}

impl NodeContentTrait for IntermediateValueNode {
    fn mark_right_edges_first(&mut self, node: &Node, edge_number: i32) -> i32 {
        if node.offset() == 0 {
            let offset = self.next.mark_right_edges_first(edge_number);
            node.set_offset(edge_number);
            offset
        } else {
            edge_number
        }
    }

    /// Returns the number of bytes written.
    fn write(&mut self, node: &Node, writer: &mut BytesTrieWriter) {
        self.next.write(writer);
        node.set_offset(
            writer
                .write_value_and_final(self.value().unwrap(), false)
                .try_into()
                .unwrap(),
        );
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
