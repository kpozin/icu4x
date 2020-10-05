use super::{
    branch_head_node::BranchHeadNode,
    builder::{BytesTrieBuilder, BytesTrieNodeTree},
    errors::BytesTrieBuilderError,
    intermediate_value_node::IntermediateValueNode,
    list_branch_node::ListBranchNode,
    node::{Node, NodeContentTrait, NodeInternal},
    split_branch_node::SplitBranchNode,
    value_node::ValueNodeTrait,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct DynamicBranchNode {
    pub(crate) value: Option<i32>,
    chars: Vec<u16>,
    equal: Vec<Node>, // TODO: Maybe `Weak<Node>`?
}

impl NodeContentTrait for DynamicBranchNode {
    fn add(
        &mut self,
        node: &Node,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
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
        let c = s[0];
        let i = self.find(c);
        if i < self.chars.len() && c == self.chars[i] {
            let replacement = self.equal[i].add(builder, s, value)?;
            self.equal[i] = replacement;
        } else {
            self.chars.insert(i, c);
            self.equal
                .insert(i, builder.create_suffix_node(s, value).into())
        }
        Ok(node.clone())
    }

    fn register(&mut self, node: &Node, tree: &mut BytesTrieNodeTree) -> Node {
        let sub_node = node.register_with_limits(tree, 0, self.chars.len() as i32);
        let mut head = BranchHeadNode::new(self.chars.len() as i32, sub_node);
        let result: Node = if self.has_value() {
            let value = self.value().unwrap();
            if tree.match_nodes_can_have_values() {
                head.set_value(value);
                head.into()
            } else {
                IntermediateValueNode::new(value, tree.register_node(head.into())).into()
            }
        } else {
            head.into()
        };
        tree.register_node(result)
    }

    fn write(&mut self, builder: &mut BytesTrieBuilder) {
        ValueNodeTrait::write(self, builder);
    }
}

impl DynamicBranchNode {
    pub fn new() -> Self {
        DynamicBranchNode {
            value: None,
            chars: vec![],
            equal: vec![],
        }
    }

    // c must not be in chars yet
    pub(crate) fn add_char(&mut self, c: u16, node: Node) {
        let i = self.find(c);
        self.chars.insert(i, c);
        self.equal.insert(i, node);
    }

    /// Binary search for the given character.
    // TODO(kpozin: Replace this with Vec::binary_search or not worth it?
    fn find(&self, c: u16) -> usize {
        let mut start = 0;
        let mut limit = self.chars.len();
        while start < limit {
            let i = (start + limit) / 2;
            let middle_char = self.chars[i];
            if c < middle_char {
                limit = i;
            } else if c == middle_char {
                return i;
            } else {
                start = i + 1;
            }
        }
        start
    }
}

trait DynamicBranchNodeExt {
    fn register_with_limits(
        &mut self,
        node: &Node,
        tree: &mut BytesTrieNodeTree,
        start: i32,
        limit: i32,
    ) -> Node;
}

impl DynamicBranchNodeExt for DynamicBranchNode {
    fn register_with_limits(
        &mut self,
        node: &Node,
        tree: &mut BytesTrieNodeTree,
        start: i32,
        limit: i32,
    ) -> Node {
        let length = limit - start;
        if length > tree.max_branch_linear_sub_node_length() {
            // Branch on the middle unit.
            let middle = start + (length / 2);
            let less_than = self.register_with_limits(node, tree, start, limit);
            let greater_or_equal = self.register_with_limits(node, tree, middle, limit);
            let split_branch_node =
                SplitBranchNode::new(self.chars[middle as usize], less_than, greater_or_equal);
            return tree.register_node(split_branch_node.into());
        }
        let mut list_branch_node = ListBranchNode::new(length as usize);
        let mut start = start as usize;
        loop {
            let c = self.chars[start];
            let node = self.equal[start].clone();
            if let NodeInternal::FinalValue(final_value_node) = &*node.borrow() {
                list_branch_node.add_with_final_value(c, final_value_node.value().unwrap());
            } else {
                list_branch_node.add_with_match_node(c, node.clone());
            }
            start += 1;
            if start >= limit as usize {
                break;
            }
        }
        tree.register_node(list_branch_node.into())
    }
}
