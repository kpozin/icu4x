use {
    super::{node::Node, value_node::ValueNode},
    std::rc::Rc,
};

#[derive(Debug)]
pub struct BytesTrieBuilder {}

impl BytesTrieBuilder {
    pub(crate) fn create_suffix_node(
        &mut self,
        s: &str,
        start: i32,
        value: i32,
    ) -> Box<dyn ValueNode> {
        todo!()
    }
}
