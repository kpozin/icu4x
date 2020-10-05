use super::{
    builder::{BytesTrieBuilder, BytesTrieWriter},
    errors::BytesTrieBuilderError,
    node::{Node, NodeContentTrait},
    value_node::ValueNodeTrait,
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
        s: &[u16],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        <FinalValueNode as ValueNodeTrait>::add(node, builder, s, value)
    }

    fn write(&mut self, builder: &mut BytesTrieWriter) {
        ValueNodeTrait::write(self, builder);
    }
}

impl FinalValueNode {
    pub fn new(value: i32) -> FinalValueNode {
        FinalValueNode { value: Some(value) }
    }
}
