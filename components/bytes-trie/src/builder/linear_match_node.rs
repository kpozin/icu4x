use {
    super::{
        builder::{BytesTrieBuilder, BytesTrieBuilderCommon, BytesTrieNodeTree, BytesTrieWriter},
        dynamic_branch_node::DynamicBranchNode,
        errors::BytesTrieBuilderError,
        intermediate_value_node::IntermediateValueNode,
        node::{GetContent, Node, NodeContentTrait, NodeInternal},
        value_node::ValueNodeContentTrait,
    },
    std::{
        cell::{RefCell, RefMut},
        hash::Hash,
        rc::Rc,
    },
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct LinearMatchNode {
    pub(crate) value: Option<i32>,
    length: i32,
    next: Node,
    string_offset: i32,
    strings: Rc<RefCell<Vec<u8>>>,
}

impl NodeContentTrait for LinearMatchNode {
    fn add(
        &mut self,
        node: &Node,
        builder: &mut BytesTrieBuilder,
        s: &[u8],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        if s.is_empty() {
            if self.has_value() {
                return Err(BytesTrieBuilderError::DuplicateString);
            } else {
                self.set_value(value);
                return Ok(node.clone());
            }
        }
        let limit = self.string_offset as usize + self.length as usize;
        let mut start = 0;
        for mut i in (self.string_offset as usize)..limit {
            if start == s.len() {
                // s is a prefix with a new value. Split self into two linear-match nodes.
                let prefix_length = i - self.string_offset as usize;
                let mut suffix_node = LinearMatchNode::new(
                    self.strings.clone(),
                    i as i32,
                    (self.length - prefix_length as i32) as i32,
                    self.next.clone(),
                );
                suffix_node.set_value(value);
                self.length = prefix_length as i32;
                self.next = suffix_node.into();
                return Ok(node.clone());
            }

            let this_char = self.strings.borrow()[i as usize];
            let new_char = s[start];
            if this_char != new_char {
                // Mismatch, insert a branch node.
                let mut branch_node = DynamicBranchNode::new();

                let (result, this_suffix_node, branch_node): (Node, Node, Node) =
                    if i == self.string_offset as usize {
                        if self.has_value() {
                            // Move the value for prefix length "start" to the new node.
                            branch_node.set_value(self.value().unwrap());
                            self.clear_value();
                        }
                        self.string_offset += 1;
                        self.length -= 1;
                        let this_suffix_node = if self.length > 0 {
                            node.clone()
                        } else {
                            self.next.clone()
                        };
                        let branch_node: Node = branch_node.into();
                        (branch_node.clone(), this_suffix_node, branch_node)
                    } else if i == limit - 1 {
                        // Mismatch on last character, keep this node for the prefix.
                        self.length -= 1;
                        let this_suffix_node = self.next.clone();
                        let branch_node: Node = branch_node.into();
                        self.next = branch_node.clone();
                        (node.clone(), this_suffix_node, branch_node)
                    } else {
                        // Mismatch on intermediate character, keep this node for the prefix.
                        let prefix_length = i - self.string_offset as usize;
                        // Suffix start offset (after this_char).
                        i += 1;
                        let this_suffix_node = LinearMatchNode::new(
                            self.strings.clone(),
                            i as i32,
                            self.length - (prefix_length as i32 + 1),
                            self.next.clone(),
                        )
                        .into();
                        self.length = prefix_length as i32;
                        let branch_node: Node = branch_node.into();
                        self.next = branch_node.clone();
                        (node.clone(), this_suffix_node, branch_node)
                    };
                let new_suffix_node = builder.create_suffix_node(&s[(start + 1)..], value);

                let mut branch_node_content =
                    <GetContent<DynamicBranchNode>>::content_mut(&branch_node);
                branch_node_content.add_char(this_char, this_suffix_node);
                branch_node_content.add_char(new_char, new_suffix_node.into());
                return Ok(result);
            }
            start += 1;
        }

        self.next = self.next.add(builder, &s[start..], value)?;
        Ok(node.clone())
    }

    fn register(&mut self, node: &Node, tree: &mut BytesTrieNodeTree) -> Node {
        self.next = self.next.register(tree);

        // Break the linear-match sequence into chunks of at most kMaxLinearMatchLength.
        let max_linear_match_length = tree.max_linear_match_length();
        while self.length > max_linear_match_length {
            let next_offset = self.string_offset + self.length - max_linear_match_length;
            self.length -= max_linear_match_length;
            let suffix_node = LinearMatchNode::new(
                self.strings.clone(),
                next_offset,
                max_linear_match_length,
                self.next.clone(),
            );
            self.next = tree.register_node(suffix_node.into());
        }
        let result = if self.has_value() && !tree.match_nodes_can_have_values() {
            let intermediate_value = self.value().unwrap();
            self.clear_value();
            IntermediateValueNode::new(intermediate_value, tree.register_node(node.clone())).into()
        } else {
            node.clone()
        };
        tree.register_node(result)
    }

    fn write(&mut self, node: &Node, writer: &mut BytesTrieWriter) {
        self.next.write(writer);
        writer.write_offset_and_length(self.string_offset, self.length);
        let offset =
            writer.write_value_and_type(self.value(), writer.min_linear_match() + self.length - 1);
        node.set_offset(offset);
    }
}

impl LinearMatchNode {
    pub fn new(
        builder_strings: Rc<RefCell<Vec<u8>>>,
        offset: i32,
        length: i32,
        next_node: Node,
    ) -> Self {
        LinearMatchNode {
            value: None,
            length,
            next: next_node,
            string_offset: offset,
            strings: builder_strings,
        }
    }
}

impl Hash for LinearMatchNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.length.hash(state);
        self.next.hash(state);
        self.string_offset.hash(state);
        self.strings.borrow().hash(state);
    }
}
