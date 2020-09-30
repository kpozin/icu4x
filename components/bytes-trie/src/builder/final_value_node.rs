use super::{
    builder::BytesTrieBuilder,
    errors::BytesTrieBuilderError,
    node::{Node, NodeTrait, RcNode},
    value_node::ValueNodeTrait,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FinalValueNode {
    pub(crate) offset: i32,
    pub(crate) value: Option<i32>,
}

impl NodeTrait for FinalValueNode {
    fn add(
        self_: &RcNode,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        value: i32,
    ) -> Result<RcNode, BytesTrieBuilderError> {
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
