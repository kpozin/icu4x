use {
    super::{
        branch_head_node::BranchHeadNode,
        builder::{BytesTrieBuilder, BytesTrieNodeTree},
        errors::BytesTrieBuilderError,
        intermediate_value_node::IntermediateValueNode,
        list_branch_node::ListBranchNode,
        node::{AsDynamicBranch, NodeInternal, NodeTrait, Node, RcNodeTrait},
        split_branch_node::SplitBranchNode,
        util::StrExt,
        value_node::ValueNodeTrait,
    },
    std::rc::Rc,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct DynamicBranchNode {
    pub(crate) value: Option<i32>,
    chars: Vec<u16>,
    equal: Vec<Node>, // TODO: Maybe `Weak<Node>`?
}

impl NodeTrait for DynamicBranchNode {
    fn add(
        self_: &Node,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        let mut dynamic_branch_node = self_.as_dynamic_branch();
        if s.is_empty() {
            if dynamic_branch_node.has_value() {
                return Err(BytesTrieBuilderError::DuplicateString);
            } else {
                dynamic_branch_node.set_value(value);
                return Ok(self_.clone());
            }
        }
        let c = s[0];
        let i = dynamic_branch_node.find(c);
        if i < dynamic_branch_node.chars.len() && c == dynamic_branch_node.chars[i] {
            let replacement = dynamic_branch_node.equal[i].add(builder, s, value)?;
            dynamic_branch_node.equal[i] = replacement;
        } else {
            dynamic_branch_node.chars.insert(i, c);
            dynamic_branch_node
                .equal
                .insert(i, builder.create_suffix_node(s, value).into())
        }
        Ok(self_.clone())
    }

    fn register(self_: &Node, tree: &mut BytesTrieNodeTree) -> Node {
        let mut dynamic_branch_node = self_.as_dynamic_branch();
        let sub_node = self_.register_with_limits(tree, 0, dynamic_branch_node.chars.len() as i32);
        let mut head = BranchHeadNode::new(dynamic_branch_node.chars.len() as i32, sub_node);
        let result: Node = if dynamic_branch_node.has_value() {
            let value = dynamic_branch_node.value().unwrap();
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
            offset: 0,
            value: None,
            chars: vec![],
            equal: vec![],
        }
    }

    // c must not be in chars yet
    pub(crate) fn add(&mut self, c: u16, node: Node) {
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
    fn register_with_limits(&self, tree: &mut BytesTrieNodeTree, start: i32, limit: i32) -> Node;
}

impl DynamicBranchNodeExt for Node {
    fn register_with_limits(&self, tree: &mut BytesTrieNodeTree, start: i32, limit: i32) -> Node {
        let mut dynamic_branch_node = self.as_dynamic_branch();
        let length = limit - start;
        if length > tree.max_branch_linear_sub_node_length() {
            // Branch on the middle unit.
            let middle = start + (length / 2);
            let less_than = self.register_with_limits(tree, start, limit);
            let greater_or_equal = self.register_with_limits(tree, middle, limit);
            let split_branch_node = SplitBranchNode::new(
                dynamic_branch_node.chars[middle as usize],
                less_than,
                greater_or_equal,
            );
            return tree.register_node(split_branch_node.into());
        }
        let mut list_branch_node = ListBranchNode::new(length as usize);
        let mut start = start as usize;
        loop {
            let c = dynamic_branch_node.chars[start];
            let node = dynamic_branch_node.equal[start].clone();
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
