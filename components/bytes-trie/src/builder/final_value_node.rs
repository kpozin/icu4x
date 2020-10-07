use super::{
    builder::{BytesTrieBuilder, BytesTrieWriter},
    errors::BytesTrieBuilderError,
    node::{Node, NodeContentTrait},
    value_node::ValueNodeContentTrait,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct FinalValueNode {
    pub(crate) value: Option<i32>,
}

impl NodeContentTrait for FinalValueNode {
    fn add(
        &mut self,
        node: &Node,
        builder: &mut BytesTrieBuilder,
        s: &[u8],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        ValueNodeContentTrait::add(self, node, builder, s, value)
    }

    fn write(&mut self, node: &Node, writer: &mut BytesTrieWriter) {
        ValueNodeContentTrait::write(self, node, writer);
    }
}

impl FinalValueNode {
    pub fn new(value: i32) -> FinalValueNode {
        FinalValueNode { value: Some(value) }
    }
}
