use super::{
    builder::BytesTrieBuilder,
    errors::BytesTrieBuilderError,
    node::{NodeInternal, NodeTrait, Node},
    value_node::ValueNodeTrait,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FinalValueNode {
    pub(crate) value: Option<i32>,
}

impl NodeTrait for FinalValueNode {
    fn add(
        self_: &Node,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        <FinalValueNode as ValueNodeTrait>::add(self_, builder, s, value)
    }

    fn write(&mut self, builder: &mut super::builder::BytesTrieBuilder) {
        ValueNodeTrait::write(self, builder);
    }
}

impl FinalValueNode {
    pub fn new(value: i32) -> FinalValueNode {
        FinalValueNode {
            offset: 0,
            value: Some(value),
        }
    }
}
