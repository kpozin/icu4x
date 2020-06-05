use {
    std::{collections::HashMap, rc::Rc},
    thiserror::Error,
};

const MAX_STRING_LENGTH: usize = 0xffff;

#[derive(Debug, Clone)]
pub struct BytesTrieBuilder {
    root: Option<Rc<Node>>,
    nodes: HashMap<Rc<Node>, Rc<Node>>,
    strings: Rc<String>,
    state: State,
    lookup_final_value_node: Option<Rc<Node>>,
}

#[derive(Debug, Clone)]
enum State {
    Adding,
    BuildingSmall,
    BuildingFast,
}

impl BytesTrieBuilder {
    fn add_impl(&mut self, s: &str, value: i32) -> Result<(), BytesTrieBuilderError> {
        if s.len() > MAX_STRING_LENGTH {
            return Err(BytesTrieBuilderError::StringLengthOutOfBounds);
        }

        match &mut self.root {
            None => {
                self.root.replace(self.create_suffix_node(s, 0, value));
            }
            Some(root) => {
                *root = root.add(self, s, 0, value);
            }
        }

        Ok(())
    }

    fn build(self) {
        unimplemented!()
    }

    /// Makes sure that there is only one unique node registered that is equivalent to `new_node`,
    /// unless `BuildingFast`.
    /// Returns `new_node` if it is the first of its kind, or an equivalent node if `new_node` is a
    /// duplicate.
    fn register_node(&mut self, new_node: Rc<Node>) -> Rc<Node> {
        if let State::BuildingFast = self.state {
            return new_node;
        }

        // BuildingSmall
        if let Some(old_node) = self.nodes.get(&new_node) {
            return old_node.clone();
        }

        // If insert() returns a `Some` value from an equivalent, previously registered node, then
        // get() failed to find that and we will leak new_node.
        let old_node = self.nodes.insert(new_node.clone(), new_node);
        debug_assert!(old_node.is_none());
        new_node
    }

     /// Makes sure that there is only one unique `FinalValueNode` registered with this value.
     /// Avoids creating a node if the value is a duplicate.
     ///
     /// Returns a `FinalValueNode` with the given value.
    fn register_final_value(&mut self, value: i32) -> Rc<Node> {
        // We always register final values because while `Adding` we do not know yet whether we will
        // build fast or small.
        
        self.lookup_final_value_node
        unimplemented!()
    }

    fn create_suffix_node(&mut self, s: &str, start: usize, value: i32) -> Result<Rc<Node>, BytesTrieBuilderError> {
        let mut value_node = self.register_final_value(value);
        if start < s.len() {
            let offset = self.strings.len();
            self.strings.push_str(&s[start..]);
            value_node = Node::new_linear_match_node(
                self.strings.clone(),
                offset,
                s.len() - start,
                value_node,
            );
        }
        Ok(value_node)
    }

    fn match_nodes_can_have_values(&self) -> bool {
        unimplemented!()
    }

    fn max_branch_linear_sub_node_length(&self) -> bool {
        unimplemented!()
    }

    fn min_linear_match(&self) -> bool {
        unimplemented!()
    }

    fn max_linear_match_length(&self) -> bool {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Node {
    offset: i32,
    contents: ValueOrBranchNode,
}

impl Node {
    pub fn new_linear_match_node(
        builder_strings: Rc<String>,
        string_offset: usize,
        len: usize,
        next_node: Rc<Node>,
    ) -> Rc<Self> {
        unimplemented!()
    }

    pub fn new_final_value_node(value: i32) -> Rc<Self> {
        unimplemented!()
    }

    pub fn new_dynamic_branch_node() -> Rc<Self> {
        unimplemented!()
    }

    pub fn add(mut self: Rc<Node>, builder: &mut BytesTrieBuilder, s: &str, start: i32, value: i32) -> Result<Rc<Node>, BytesTrieBuilderError> {
        match &mut self.contents {
            ValueOrBranchNode::Value(self_value_node) => {
                match &mut self_value_node.extension {
                    ValueNodeExtension::LinearMatch(self_linear_match_node) => {
                        if start == s.len() as i32 {
                            if self_value_node.value.is_some() {
                                return Err(BytesTrieBuilderError::DuplicateString);
                            } else {
                                self_value_node.set_value(value);
                                return Ok(self.clone());
                            }
                        }
                        let string_offset = self_linear_match_node.string_offset;
                        let length = self_linear_match_node.length;
                        let limit =  string_offset + length;
                        for (i, start) in (string_offset..limit).zip(start..start+length) {
                            if start == s.len() as i32 {
                                let prefix_length = i - string_offset;
                                let mut suffix_node = Node::new_linear_match_node(
                                    self_linear_match_node.strings.clone(), 
                                    string_offset as usize, 
                                    length as usize, 
                                    self_linear_match_node.next.clone());
                                suffix_node.set_value(value);
                                self_linear_match_node.length = prefix_length;
                                self_linear_match_node.next = self.clone();
                                return Ok(self);
                            }
                            let this_char = self_linear_match_node.strings.char_at(i as usize);
                            let new_char = s.chars().next();
                            // TODO: What if one or more is None?
                            if this_char != new_char {
                                // Mismatch, insert a branch node.
                                let branch_node = Node::new_dynamic_branch_node();
                                let (result_node, this_suffix_node) = if i == string_offset {
                                    if self_value_node.value.is_some() {
                                        branch_node.set_value(value);
                                        self_value_node.va
                                    }
                                } else if i == limit - 1 {

                                } else {

                                };
                            }
                        }
                    },
                    _ => {

                    }
                }
                if start == s.len() as i32 {
                    return Err(BytesTrieBuilderError::DuplicateString);
                }
                let mut node = builder.create_suffix_node(s, start, value)?;
                node.set_value(value);
                Ok(node)
            }
            ValueOrBranchNode::Branch(branch_node) => {
                match branch_node.extension {
                    BranchNodeExtension::ListBranch(list_branch_node) => {
                        
                    },
                    BranchNodeExtension::SplitBranch(split_branch_node) => {

                    },
                }
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum ValueOrBranchNode {
    Value(ValueNode),
    Branch(BranchNode),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]

struct ValueNode {
    value: Option<i32>,
    extension: ValueNodeExtension,
}

impl ValueNode {
    fn has_value(&self) -> bool {
        self.value.is_some()
    }

    fn set_value(&mut self, value: i32) {
        self.value = Some(value);
    }

    fn clear_value(&mut self) {
        self.value = None;
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]

enum ValueNodeExtension {
    FinalValue,
    IntermediateValue(IntermediateValueNode),
    LinearMatch(LinearMatchNode),
    DynamicBranch(DynamicBranchNode),
    BranchHead(BranchHeadNode),
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]

struct IntermediateValueNode {
    next: Rc<Node>,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]

struct LinearMatchNode {
    strings: Rc<String>,
    string_offset: i32,
    length: i32,
    next: Rc<Node>,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]

struct DynamicBranchNode {
    chars: Vec<char>,
    equal: Option<Vec<Rc<Node>>>,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]

struct BranchHeadNode {
    length: i32,
    next: Rc<Node>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct BranchNode {
    first_edge_number: i32,
    extension: BranchNodeExtension,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]

enum BranchNodeExtension {
    ListBranch(ListBranchNode),
    SplitBranch(SplitBranchNode),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]

struct ListBranchNode {
    equal: Option<Vec<Rc<Node>>>,
    length: i32,
    values: Vec<i32>,
    units: Vec<char>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]

struct SplitBranchNode {
    unit: char,
    less: Rc<Node>,
    greater_or_equal: Rc<Node>,
}

#[derive(Error, Debug)]
pub enum BytesTrieBuilderError {
    #[error("The maximum string length is 0xffff")]
    StringLengthOutOfBounds,
    #[error("Duplicate string")]
    DuplicateString,
}

trait StrExt {
    fn char_at(&self, index: usize) -> Option<char>;
}

impl<T> StrExt for T
where T: std::borrow::Borrow<str> {
    fn char_at(&self, index: usize) -> Option<char> {
        let s: &str = self.borrow();
        s.get(index..=index).and_then(|s| s.chars().next())
    }
}