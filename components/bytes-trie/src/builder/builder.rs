use {
    super::{
        errors::BytesTrieBuilderError,
        final_value_node::FinalValueNode,
        linear_match_node::LinearMatchNode,
        node::{Node, NodeContent, NodeInternal, NodeTrait},
        value_node::{ValueNode, ValueNodeContentTrait},
    },
    crate::trie::encoding::{CompactDelta, CompactInt, CompactValue, VALUE_IS_FINAL},
    std::{cell::RefCell, collections::HashSet, convert::TryInto, rc::Rc},
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
pub(crate) struct CommonData {
    /// Strings and sub-strings for linear-match nodes.
    strings: Rc<RefCell<Vec<u8>>>,

    root: Option<Node>,

    /// Hash set of nodes, maps from nodes to integer 1.
    nodes: HashSet<Node>,

    lookup_final_value_node: Node,
}

/// Initial state of `BytesTrieBuilder`.
#[derive(Debug)]
pub struct BytesTrieBuilder {
    common_data: CommonData,
}

pub(crate) trait BytesTrieBuilderCommon {
    fn common_data(&self) -> &CommonData;

    fn common_data_mut(&mut self) -> &mut CommonData;

    fn register_final_value(&mut self, value: i32) -> Node {
        let data = self.common_data_mut();
        // We always register final values because while ADDING we do not know yet whether we will
        // build fast or small.
        match &mut *data.lookup_final_value_node.content_mut() {
            NodeContent::FinalValue(node) => {
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

        let new_node: Node = FinalValueNode::new(value).into();
        // If `insert()` indicates that there was an equivalent, previously registered node, then
        // `get()` failed to find that and we will leak `new_node`.
        let was_absent = data.nodes.insert(new_node.clone());
        assert!(was_absent);

        new_node
    }

    fn min_linear_match(&self) -> i32 {
        todo!()
    }

    fn max_branch_linear_sub_node_length(&self) -> i32 {
        todo!()
    }

    fn match_nodes_can_have_values(&self) -> bool {
        todo!()
    }

    fn max_linear_match_length(&self) -> i32 {
        todo!()
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
        let writer = BytesTrieWriter::from_node_tree(tree)?;
        // writer.ma
        todo!()
    }

    pub(crate) fn add_impl(&mut self, s: &[u8], value: i32) -> Result<(), BytesTrieBuilderError> {
        if s.len() > MAX_KEY_LENGTH {
            return Err(BytesTrieBuilderError::KeyTooLong);
        }

        // Note: can't put common_data_mut() in a variable due to borrowing restrictions.
        if self.common_data_mut().root.is_none() {
            self.common_data_mut().root = Some(self.create_suffix_node(s, value).into());
        } else {
            self.common_data_mut().root = Some(
                self.common_data_mut()
                    .root
                    .take()
                    .unwrap()
                    .add(self, s, value)?,
            );
        }

        Ok(())
    }

    // pub(crate) build_impl(&mut self)

    pub(crate) fn create_suffix_node(&mut self, s: &[u8], value: i32) -> Node {
        let node = self.register_final_value(value);
        let node = if s.is_empty() {
            node
        } else {
            let data = self.common_data();
            let offset = data.strings.borrow().len();
            data.strings.borrow_mut().extend_from_slice(s);
            LinearMatchNode::new(data.strings.clone(), offset as i32, s.len() as i32, node).into()
        };
        node
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

        let mut tree = BytesTrieNodeTree {
            common_data,
            build_mode,
        };

        let root = tree.common_data().root.as_ref().unwrap().clone();
        root.register(&mut tree);

        Ok(tree)
    }

    pub(crate) fn register_node(&mut self, new_node: Node) -> Node {
        if self.build_mode == BuildMode::Fast {
            return new_node;
        }
        // BuildMode::Small

        let old_node = self.common_data.nodes.get(&new_node);
        if let Some(old_node) = old_node {
            old_node.clone()
        } else {
            let was_absent = self.common_data.nodes.insert(new_node.clone());
            assert!(was_absent);
            new_node
        }
    }
}

impl BytesTrieBuilderCommon for BytesTrieNodeTree {
    fn common_data(&self) -> &CommonData {
        &self.common_data
    }

    fn common_data_mut(&mut self) -> &mut CommonData {
        &mut self.common_data
    }
}

#[derive(Debug)]
pub(crate) struct BytesTrieWriter {
    common_data: CommonData,
    bytes: Vec<u8>,
}

impl BytesTrieWriter {
    fn from_node_tree(tree: BytesTrieNodeTree) -> Result<Self, BytesTrieBuilderError> {
        let BytesTrieNodeTree {
            common_data,
            build_mode: _,
        } = tree;
        Ok(BytesTrieWriter {
            common_data,
            bytes: vec![],
        })
    }

    pub(crate) fn write_unit(&mut self, unit: u8) -> usize {
        self.bytes.push(unit);
        self.bytes.len()
    }

    pub(crate) fn write_offset_and_length(&mut self, offset: usize, length: usize) -> usize {
        let source = &self.common_data().strings.borrow()[offset..(offset + length)];
        self.bytes.extend_from_slice(source);
        self.bytes.len()
    }

    pub(crate) fn write_value_and_final(&mut self, value: i32, is_final: bool) -> usize {
        let final_mask: u8 = if is_final { VALUE_IS_FINAL } else { 0 };
        if (0..=CompactValue::MAX_ONE_BYTE as i32).contains(&value) {
            let unit = ((CompactValue::MIN_ONE_BYTE_LEAD as i32 + value) << 1) | final_mask as i32;
            return self.write_unit(unit as u8);
        }

        let mut bytes: Vec<u8> = Vec::with_capacity(5);
        // Doesn't fit in three bytes.
        if value < 0 || value > 0xffffff {
            bytes.push(CompactValue::FIVE_BYTE_LEAD as u8);
            bytes.push((value >> 24) as u8);
            bytes.push((value >> 16) as u8);
            bytes.push((value >> 8) as u8);
            bytes.push(value as u8);
        } else {
            if value <= CompactValue::MAX_TWO_BYTE {
                bytes.push((CompactValue::MIN_TWO_BYTE_LEAD as i32 + (value >> 8)) as u8);
            } else {
                if value <= CompactValue::MAX_THREE_BYTE {
                    bytes.push((CompactValue::MIN_THREE_BYTE_LEAD as i32 + (value >> 16)) as u8);
                } else {
                    bytes.push(CompactValue::FOUR_BYTE_LEAD as u8);
                    bytes.push((value >> 16) as u8);
                }
                bytes.push((value >> 8) as u8);
            }
            bytes.push(value as u8);
        }
        bytes[0] = ((bytes[0] << 1) | final_mask) as u8;
        self.write_bytes(&bytes[..])
    }

    pub(crate) fn write_value_and_type(&mut self, value: Option<i32>, node: i32) -> usize {
        let offset = self.write_unit(node as u8);
        if let Some(value) = value {
            self.write_value_and_final(value, false)
        } else {
            offset
        }
    }

    pub(crate) fn write_delta_to(&mut self, jump_target: i32) -> usize {
        let i = self.bytes.len() as i32 - jump_target;
        assert!(i > 0);
        if i <= CompactDelta::MAX_ONE_BYTE as i32 {
            return self.write_unit(i as u8);
        }

        let bytes: Vec<u8> = Vec::with_capacity(5);

        if i <= CompactDelta::MAX_TWO_BYTE {
            bytes.push((CompactDelta::MIN_TWO_BYTE_LEAD as i32 + (i >> 8)) as u8);
        } else {
            if i <= CompactDelta::MAX_THREE_BYTE {
                bytes.push((CompactDelta::MIN_THREE_BYTE_LEAD as i32 + (i >> 16)) as u8);
            } else {
                if i <= 0xffffff {
                    bytes.push(CompactDelta::FOUR_BYTE_LEAD as u8);
                } else {
                    bytes.push(CompactDelta::FIVE_BYTE_LEAD);
                    bytes.push(value)
                }
            }
        }

        todo!()
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> usize {
        self.bytes.extend_from_slice(bytes);
        self.bytes.len()
    }
}

impl BytesTrieBuilderCommon for BytesTrieWriter {
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
