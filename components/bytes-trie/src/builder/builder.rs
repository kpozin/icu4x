use {
    super::{
        errors::BytesTrieBuilderError,
        node::{Node, RcNode, RcNodeTrait},
        value_node::ValueNode,
        linear_match_node::LinearMatchNode,
    },
    std::{collections::HashMap, rc::Rc, cell::RefCell},
};

const MAX_KEY_LENGTH: usize = 0xffff;

/// Builder state.
#[derive(Debug, Eq, PartialEq)]
enum State {
    Adding,
    BuildingFast,
    BuildingSmall,
    Built,
}

#[derive(Debug)]
pub struct BytesTrieBuilder {
    state: State,

    /// Strings and sub-strings for linear-match nodes.
    strings: Rc<RefCell<Vec<u16>>>,

    root: Option<RcNode>,

    /// Hash set of nodes, maps from nodes to integer 1.
    nodes: HashMap<RcNode, RcNode>,

    lookup_final_value_node: RcNode,
}

impl BytesTrieBuilder {
    pub(crate) fn add_impl(&mut self, s: &[u16], value: i32) -> Result<(), BytesTrieBuilderError> {
        if self.state != State::Adding {
            return Err(BytesTrieBuilderError::AddAfterBuild);
        }
        if s.len() > MAX_KEY_LENGTH {
            return Err(BytesTrieBuilderError::KeyTooLong);
        }

        if self.root.is_none() {
            self.root = Some(self.create_suffix_node(s, 0, value).into());
        } else {
            self.root = Some(self.root.take().unwrap().add(self, s, 0, value)?);
        }

        Ok(())
    }

    // pub(crate) build_impl(&mut self)

    pub(crate) fn create_suffix_node(&mut self, s: &[u16], start: usize, value: i32) -> ValueNode {
        let node = self.register_final_value(value);
        let strings_len = self.strings.borrow().len();
        if start < strings_len {
            let offset = strings_len;
            self.strings.borrow_mut().extend_from_slice(&s[start..]);
            LinearMatchNode::new(builder_strings.clone(), offset, s.len(), next_node)
        }
        node
    }

    pub(crate) fn min_linear_match(&self) -> i32 {
        todo!()
    }

    pub(crate) fn write_unit(&mut self, unit: u16) -> i32 {
        todo!()
    }

    pub(crate) fn write_offset_and_length(&mut self, offset: i32, length: i32) -> i32 {
        todo!()
    }

    pub(crate) fn write_value_and_type(&mut self, value: Option<i32>, node: i32) -> i32 {
        todo!()
    }

    pub(crate) fn write_value_and_final(&mut self, value: i32, is_final: bool) -> i32 {
        todo!()
    }

    pub(crate) fn max_branch_linear_sub_node_length(&self) -> i32 {
        todo!()
    }

    pub(crate) fn register_node(&mut self, node: RcNode) -> RcNode {
        todo!()
    }

    pub(crate) fn match_nodes_can_have_values(&self) -> bool {
        todo!()
    }

    pub(crate) fn max_linear_match_length(&self) -> i32 {
        todo!()
    }

    pub(crate) fn write_delta_to(&mut self, jump_target: i32) -> i32 {
        todo!()
    }
}

/// Build options for `BytesTrieBuilder`.
#[derive(Debug)]
pub enum BuildMode {
    /// Builds a trie quickly.
    Fast,
    /// Builds a trie more slowly, attempting to generate a shorter but equivalent serialization.
    /// This build option also uses more memory.
    ///
    /// This option can be effective when many integer values are the same and string/byte sequence
    /// suffixes can be shared. Runtime speed is not expected to improve.
    Small,
}
