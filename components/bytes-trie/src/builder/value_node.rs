use {
    super::{
        builder::BytesTrieBuilder,
        errors::BytesTrieBuilderError,
        node::{Node, NodeImpl},
    },
    std::rc::Rc,
};

pub(crate) trait ValueNode: Node {
    // TODO: constructors

    fn value(&self) -> Option<i32>;
    fn set_value(&mut self, value: i32) -> Option<i32>;
    fn has_value(&self) -> bool {
        self.value().is_some()
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct ValueNodeImpl {
    node: NodeImpl,
    value: Option<i32>,
}

impl Node for ValueNodeImpl {
    fn offset(&self) -> i32 {
        self.node.offset()
    }

    fn set_offset(&mut self, offset: i32) {
        self.node.set_offset(offset)
    }

    fn add(
        &mut self,
        builder: &mut BytesTrieBuilder,
        s: &str,
        start: i32,
        value: i32,
    ) -> Result<Option<Rc<dyn Node>>, BytesTrieBuilderError> {
        if start == s.len() as i32 {
            Err(BytesTrieBuilderError::DuplicateString)
        } else {
            let mut value_node = builder.create_suffix_node(s, start, value);
            value_node.set_value(value);
            let value_node: Rc<dyn ValueNode> = value_node.into();
            Ok(Some(value_node.as_node()))
        }
    }
}

impl ValueNode for ValueNodeImpl {
    fn value(&self) -> Option<i32> {
        self.value
    }

    fn set_value(&mut self, value: i32) -> Option<i32> {
        assert!(!self.has_value());
        self.value.replace(value)
    }
}
