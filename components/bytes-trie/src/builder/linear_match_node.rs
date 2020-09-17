use {
    super::{
        builder::BytesTrieBuilder,
        dynamic_branch_node::DynamicBranchNode,
        errors::BytesTrieBuilderError,
        intermediate_value_node::IntermediateValueNode,
        node::{AsDynamicBranch, AsLinearMatch, Node, NodeTrait, RcNode, RcNodeTrait, WithOffset},
        value_node::ValueNodeTrait,
    },
    std::{
        cell::{RefCell, RefMut},
        rc::Rc,
    },
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct LinearMatchNode {
    pub(crate) offset: i32,
    pub(crate) value: Option<i32>,
    length: i32,
    next: RcNode,
    string_offset: i32,
    strings: Rc<RefCell<Vec<u16>>>,
}

impl NodeTrait for LinearMatchNode {
    fn add(
        self_: &RcNode,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        start: i32,
        value: i32,
    ) -> Result<RcNode, BytesTrieBuilderError> {
        let mut linear_match_node = self_.as_linear_match();
        if start == s.len() as i32 {
            if linear_match_node.has_value() {
                return Err(BytesTrieBuilderError::DuplicateString);
            } else {
                linear_match_node.set_value(value);
                return Ok(self_.clone());
            }
        }
        let limit = linear_match_node.string_offset as usize + linear_match_node.length as usize;
        let mut start = start as usize;
        for mut i in (linear_match_node.string_offset as usize)..limit {
            if start == s.len() {
                // s is a prefix with a new value. Split self into two linear-match nodes.
                let prefix_length = i - linear_match_node.string_offset as usize;
                let mut suffix_node = LinearMatchNode::new(
                    linear_match_node.strings.clone(),
                    i as i32,
                    (linear_match_node.length - prefix_length as i32) as i32,
                    linear_match_node.next.clone(),
                );
                suffix_node.set_value(value);
                linear_match_node.length = prefix_length as i32;
                linear_match_node.next = suffix_node.into();
                return Ok(self_.clone());
            }

            let this_char = linear_match_node.strings.borrow()[i as usize];
            let new_char = s[start];
            if this_char != new_char {
                // Mismatch, insert a branch node.
                let mut branch_node = DynamicBranchNode::new();

                let (result, this_suffix_node, branch_node): (RcNode, RcNode, RcNode) =
                    if i == linear_match_node.string_offset as usize {
                        if linear_match_node.has_value() {
                            // Move the value for prefix length "start" to the new node.
                            branch_node.set_value(linear_match_node.value().unwrap());
                            linear_match_node.clear_value();
                        }
                        linear_match_node.string_offset += 1;
                        linear_match_node.length -= 1;
                        let this_suffix_node = if linear_match_node.length > 0 {
                            self_.clone()
                        } else {
                            linear_match_node.next.clone()
                        };
                        let branch_node: RcNode = branch_node.into();
                        (branch_node.clone(), this_suffix_node, branch_node)
                    } else if i == limit - 1 {
                        // Mismatch on last character, keep this node for the prefix.
                        linear_match_node.length -= 1;
                        let this_suffix_node = linear_match_node.next.clone();
                        let branch_node: RcNode = branch_node.into();
                        linear_match_node.next = branch_node.clone();
                        (self_.clone(), this_suffix_node, branch_node)
                    } else {
                        // Mismatch on intermediate character, keep this node for the prefix.
                        let prefix_length = i - linear_match_node.string_offset as usize;
                        // Suffix start offset (after this_char).
                        i += 1;
                        let this_suffix_node = LinearMatchNode::new(
                            linear_match_node.strings.clone(),
                            i as i32,
                            linear_match_node.length - (prefix_length as i32 + 1),
                            linear_match_node.next.clone(),
                        )
                        .into();
                        linear_match_node.length = prefix_length as i32;
                        let branch_node: RcNode = branch_node.into();
                        linear_match_node.next = branch_node.clone();
                        (self_.clone(), this_suffix_node, branch_node)
                    };
                let new_suffix_node = builder.create_suffix_node(s, (start + 1) as i32, value);

                branch_node
                    .as_dynamic_branch()
                    .add(this_char, this_suffix_node);
                branch_node
                    .as_dynamic_branch()
                    .add(new_char, new_suffix_node.into());
                return Ok(result);
            }
            start += 1;
        }

        linear_match_node.next = linear_match_node
            .next
            .add(builder, s, start as i32, value)?;
        Ok(self_.clone())
    }

    fn register(self_: &RcNode, builder: &mut BytesTrieBuilder) -> RcNode {
        let mut linear_match_node = self_.as_linear_match();
        linear_match_node.next = linear_match_node.next.register(builder);

        // Break the linear-match sequence into chunks of at most kMaxLinearMatchLength.
        let max_linear_match_length = builder.max_linear_match_length();
        while linear_match_node.length > max_linear_match_length {
            let next_offset = linear_match_node.string_offset + linear_match_node.length
                - max_linear_match_length;
            linear_match_node.length -= max_linear_match_length;
            let suffix_node = LinearMatchNode::new(
                linear_match_node.strings.clone(),
                next_offset,
                max_linear_match_length,
                linear_match_node.next.clone(),
            );
            linear_match_node.next = builder.register_node(suffix_node.into());
        }
        let result = if linear_match_node.has_value() && !builder.match_nodes_can_have_values() {
            let intermediate_value = linear_match_node.value().unwrap();
            linear_match_node.clear_value();
            IntermediateValueNode::new(intermediate_value, builder.register_node(self_.clone()))
                .into()
        } else {
            self_.clone()
        };
        builder.register_node(result)
    }

    fn write(&mut self, builder: &mut super::builder::BytesTrieBuilder) {
        self.next.borrow_mut().write(builder);
        builder.write_offset_and_length(self.string_offset, self.length);
        self.offset = builder
            .write_value_and_type(self.value(), builder.min_linear_match() + self.length - 1);
    }
}

impl LinearMatchNode {
    pub fn new(
        builder_strings: Rc<RefCell<Vec<u16>>>,
        offset: i32,
        length: i32,
        next_node: RcNode,
    ) -> Self {
        LinearMatchNode {
            offset: 0,
            value: None,
            length,
            next: next_node,
            string_offset: offset,
            strings: builder_strings,
        }
    }
}
