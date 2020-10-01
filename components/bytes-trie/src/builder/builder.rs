use std::convert::TryFrom;

use {
    super::{
        errors::BytesTrieBuilderError,
        final_value_node::FinalValueNode,
        linear_match_node::LinearMatchNode,
        node::{Node, NodeTrait, RcNode, RcNodeTrait},
        value_node::{ValueNode, ValueNodeTrait},
    },
    std::{cell::RefCell, collections::HashSet, rc::Rc},
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
struct CommonData {
    /// Strings and sub-strings for linear-match nodes.
    strings: Rc<RefCell<Vec<u16>>>,

    root: Option<RcNode>,

    /// Hash set of nodes, maps from nodes to integer 1.
    nodes: HashSet<RcNode>,

    lookup_final_value_node: RcNode,
}

/// Initial state of `BytesTrieBuilder`.
#[derive(Debug)]
pub struct BytesTrieBuilder {
    common_data: CommonData,
}

trait BytesTrieBuilderCommon {
    fn common_data(&self) -> &CommonData;

    fn common_data_mut(&mut self) -> &mut CommonData;

    fn register_final_value(&mut self, value: i32) -> RcNode {
        let data = self.common_data_mut();
        // We always register final values because while ADDING we do not know yet whether we will
        // build fast or small.
        match &mut *data.lookup_final_value_node.borrow_mut() {
            Node::FinalValue(node) => {
                node.set_final_value(value);
            }
            _ => {
                panic!("Unexpected node type: {:?}", data.lookup_final_value_node);
            }
        };
        let old_node = data.nodes.get(&data.lookup_final_value_node);
        if let Some(old_node) = old_node {
            return old_node.clone();
        }

        let new_node: RcNode = FinalValueNode::new(value).into();
        // If `insert()` indicates that there was an equivalent, previously registered node, then
        // `get()` failed to find that and we will leak `new_node`.
        let was_absent = data.nodes.insert(new_node.clone());
        assert!(was_absent);

        new_node
    }
}

impl BytesTrieBuilder {
    pub fn build_fast(self) -> Result<Vec<u8>, BytesTrieBuilderError> {
        self.build_impl(BuildMode::Fast)
    }

    pub fn build_small(self) -> Result<Vec<u8>, BytesTrieBuilderError> {
        self.build_impl(BuildMode::Small)
    }

    fn build_impl(self, build_mode: BuildMode) -> Result<Vec<u8>, BytesTrieBuilderError> {
        let tree = BytesTrieNodeTree::from_builder(self, build_mode)?;
    }

    pub(crate) fn add_impl(&mut self, s: &[u16], value: i32) -> Result<(), BytesTrieBuilderError> {
        if s.len() > MAX_KEY_LENGTH {
            return Err(BytesTrieBuilderError::KeyTooLong);
        }

        let data = self.common_data();
        if data.root.is_none() {
            data.root = Some(self.create_suffix_node(s, value).into());
        } else {
            data.root = Some(data.root.take().unwrap().add(self, s, 0, value)?);
        }

        Ok(())
    }

    // pub(crate) build_impl(&mut self)

    pub(crate) fn create_suffix_node(&mut self, s: &[u16], value: i32) -> ValueNode {
        let data = self.common_data();
        let node = self.register_final_value(value);
        if !s.is_empty() {
            let offset = data.strings.borrow().len();
            data.strings.borrow_mut().extend_from_slice(s);
            LinearMatchNode::new(data.strings.clone(), offset, s.len(), next_node).into()
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

#[derive(Debug)]
pub(crate) struct BytesTrieNodeTree {
    common_data: CommonData,
    build_mode: BuildMode,
}

impl BytesTrieNodeTree {
    fn from_builder(
        builder: BytesTrieBuilder,
        build_mode: BuildMode,
    ) -> Result<Self, BytesTrieBuilderError> {
        let BytesTrieBuilder { common_data } = builder;

        let tree = BytesTrieNodeTree {
            common_data,
            build_mode,
        };

        tree.common_data.root.register(&mut tree);
    }

    pub(crate) fn register_node(&mut self, new_node: RcNode) -> RcNode {
        if self.build_mode == BuildMode::Fast {
            return new_node;
        }
        // BuildMode::Small

        let old_node = self.common_data.nodes.get(&new_node);
        if Some(old_node) = old_node {
            old_node.clone()
        } else {
            let was_absent = self.common_data.nodes.insert(new_node.clone());
            assert!(was_absent);
            new_node
        }
    }
}

impl BytesTrieBuilderCommon for BytesTrieBuilder {
    fn common_data(&self) -> &CommonData {
        &self.common_data
    }

    fn common_data_mut(&mut self) -> &mut CommonData {
        &mut self.common_data
    }
}

/// Build options for `BytesTrieBuilder`.
#[derive(Debug, Eq, PartialEq)]
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
