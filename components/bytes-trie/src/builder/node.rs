use {
    super::{
        branch_head_node::BranchHeadNode,
        builder::{BytesTrieBuilder, BytesTrieNodeTree, BytesTrieWriter},
        dynamic_branch_node::DynamicBranchNode,
        errors::BytesTrieBuilderError,
        final_value_node::FinalValueNode,
        intermediate_value_node::IntermediateValueNode,
        linear_match_node::LinearMatchNode,
        list_branch_node::ListBranchNode,
        split_branch_node::SplitBranchNode,
    },
    paste::paste,
    std::{
        cell::{Ref, RefCell, RefMut},
        fmt::Debug,
        hash::Hash,
        rc::Rc,
    },
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct Node(Rc<RefCell<NodeInternal>>);

impl Node {
    pub(crate) fn offset(&self) -> i32 {
        self.internal().offset()
    }

    pub(crate) fn set_offset(&self, offset: i32) {
        self.internal_mut().set_offset(offset);
    }

    /// Recursive method for adding a new (string, value) pair. Matches the remaining part of `s`
    /// from `start`, and adds a new node where there is a mismatch.
    ///
    /// Returns `None`, or a replacement `Node`.
    fn add(
        &self,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        self.internal_mut().add(self, builder, s, value)
    }

    /// Recursive method for registering unique nodes, after all (string, value) pairs have been
    /// added. Final-value nodes are pre-registered while `add()`ing (string, value) pairs.
    /// Other nodes created while `add()`ing `register_node()` themselves later and might replace
    /// themselves with new types of nodes for `write()`ing.
    ///
    /// Returns the registered version of this node which implements `write()`, or `None` if self
    /// is the instance registered.
    fn register(&self, tree: &mut BytesTrieNodeTree) -> Node {
        self.internal_mut().register(self, tree)
    }

    /// Traverses the `Node` graph and numbers branch edges, with rightmost edges first.
    /// This is to avoid writing a duplicate node twice.
    ///
    /// Branch nodes in this trie data structure are not symmetric.
    /// Most branch edges "jump" to other nodes but the rightmost branch edges just continue without
    /// a jump.
    /// Therefore, `write()` must write the rightmost branch edge last (trie units are written
    /// backwards), and must write it at that point even if it is a duplicate of a node previously
    /// written elsewhere.
    ///
    /// This function visits and marks right branch edges first.
    /// Edges are numbered with increasingly negative values because we share the offset field which
    /// gets positive values when nodes are written. A branch edge also remembers the first number
    /// for any of its edges.
    ///
    /// When a further-left branch edge has a number in the range of the rightmost edge's numbers,
    /// then it will be written as part of the required right edge and we can avoid writing it
    /// first.
    ///
    /// After `root.mark_right_edges_first(-1)` the offsets of all nodes are negative edge numbers.
    ///
    /// `edge_number`: The first edge number for this node and its sub-nodes.
    ///
    /// Returns an edge number that is at least the maximum-negative of the input edge number and
    /// the numbers of this node and all of its sub-nodes.
    fn mark_right_edges_first(&self, edge_number: i32) -> i32 {
        self.internal_mut().mark_right_edges_first(edge_number)
    }

    /// Must set the offset to a positive value.
    fn write(&self, writer: &mut BytesTrieWriter) {
        self.internal_mut().write(writer);
    }

    /// See `mark_right_edges_first`.
    pub(crate) fn write_unless_inside_right_edge(
        &self,
        first_right: i32,
        last_right: i32,
        writer: &mut BytesTrieWriter,
    ) {
        // Note: Edge numbers are negative, last_right <= first_right.
        // If offset > 0 then this node and its sub-nodes have been written already and we need not
        // write them again.
        // If this node is part of the unwritten right branch edge, then we wait until that is
        // written.
        let offset = self.offset();
        if offset < 0 && (offset < last_right || first_right < offset) {
            self.write(writer);
        }
    }

    fn internal<'a>(&'a self) -> Ref<'a, NodeInternal> {
        self.0.borrow()
    }

    fn internal_mut<'a>(&'a self) -> RefMut<'a, NodeInternal> {
        self.0.borrow_mut()
    }
}

pub(crate) trait NodeTrait<C: NodeContentTrait>: GetContent<C> {
    fn add(
        &self,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError>;
}

pub(crate) trait NodeContentTrait: Debug + Eq + PartialEq + Hash + 'static {
    fn mark_right_edges_first(&mut self, edge_number: i32) -> i32 {
        if self.offset() == 0 {
            self.set_offset(edge_number);
        }
        edge_number
    }

    fn write(&mut self, writer: &mut BytesTrieWriter);
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct NodeInternal {
    offset: i32,
    content: NodeContent,
}

impl NodeInternal {
    fn offset(&self) -> i32 {
        self.offset
    }

    fn set_offset(&mut self, offset: i32) {
        self.offset = offset;
    }
}

impl NodeContentTrait for NodeInternal {
    fn mark_right_edges_first(&mut self, edge_number: i32) -> i32 {
        match &mut self.content {
            NodeContent::FinalValue(n) => n.mark_right_edges_first(edge_number),
            NodeContent::BranchHead(n) => n.mark_right_edges_first(edge_number),
            NodeContent::DynamicBranch(n) => n.mark_right_edges_first(edge_number),
            NodeContent::IntermediateValue(n) => n.mark_right_edges_first(edge_number),
            NodeContent::LinearMatch(n) => n.mark_right_edges_first(edge_number),
            NodeContent::ListBranch(n) => n.mark_right_edges_first(edge_number),
            NodeContent::SplitBranch(n) => n.mark_right_edges_first(edge_number),
        }
    }

    fn write(&mut self, writer: &mut BytesTrieWriter) {
        match &mut self.content {
            NodeContent::FinalValue(n) => n.write(writer),
            NodeContent::BranchHead(n) => n.write(writer),
            NodeContent::DynamicBranch(n) => n.write(writer),
            NodeContent::IntermediateValue(n) => n.write(writer),
            NodeContent::LinearMatch(n) => n.write(writer),
            NodeContent::ListBranch(n) => n.write(writer),
            NodeContent::SplitBranch(n) => n.write(writer),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum NodeContent {
    FinalValue(FinalValueNode),
    BranchHead(BranchHeadNode),
    DynamicBranch(DynamicBranchNode),
    IntermediateValue(IntermediateValueNode),
    LinearMatch(LinearMatchNode),

    ListBranch(ListBranchNode),
    SplitBranch(SplitBranchNode),
}

pub(crate) trait NodeTrait: Debug + Eq + PartialEq {
    fn add(
        self_: &Node,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        Ok(self_.clone())
    }

    fn register(self_: &Node, tree: &mut BytesTrieNodeTree) -> Node {
        self_.clone()
    }

    fn mark_right_edges_first(&mut self, edge_number: i32) -> i32 {
        if self.offset() == 0 {
            self.set_offset(edge_number);
        }
        edge_number
    }

    /// Must set the offset to a positive value.
    fn write(&mut self, builder: &mut BytesTrieBuilder);

    /// See `mark_right_edges_first`.
    fn write_unless_inside_right_edge(
        &mut self,
        first_right: i32,
        last_right: i32,
        builder: &mut BytesTrieBuilder,
    ) {
        // Note: Edge numbers are negative, last_right <= first_right.
        // If offset > 0 then this node and its sub-nodes have been written already and we need not
        // write them again.
        // If this node is part of the unwritten right branch edge, then we wait until that is
        // written.
        let offset = self.offset();
        if offset < 0 && (offset < last_right || first_right < offset) {
            self.write(builder);
        }
    }
}

// impl NodeTrait for NodeInternal {
//     fn register(self_: &Node, tree: &mut BytesTrieNodeTree) -> Node {
//         match *self_.borrow() {
//             NodeInternal::FinalValue(n) => <FinalValueNode as NodeTrait>::register(self_, tree),
//             NodeInternal::BranchHead(n) => <BranchHeadNode as NodeTrait>::register(self_, tree),
//             NodeInternal::DynamicBranch(n) => {
//                 <DynamicBranchNode as NodeTrait>::register(self_, tree)
//             }
//             NodeInternal::IntermediateValue(n) => {
//                 <IntermediateValueNode as NodeTrait>::register(self_, tree)
//             }
//             NodeInternal::LinearMatch(n) => <LinearMatchNode as NodeTrait>::register(self_, tree),
//             NodeInternal::ListBranch(n) => <ListBranchNode as NodeTrait>::register(self_, tree),
//             NodeInternal::SplitBranch(n) => <SplitBranchNode as NodeTrait>::register(self_, tree),
//         }
//     }

//     fn write(&mut self, builder: &mut BytesTrieBuilder) {
//         match self {
//             NodeInternal::FinalValue(n) => NodeTrait::write(n, builder),
//             NodeInternal::BranchHead(n) => NodeTrait::write(n, builder),
//             NodeInternal::DynamicBranch(n) => NodeTrait::write(n, builder),
//             NodeInternal::IntermediateValue(n) => NodeTrait::write(n, builder),
//             NodeInternal::LinearMatch(n) => NodeTrait::write(n, builder),
//             NodeInternal::ListBranch(n) => NodeTrait::write(n, builder),
//             NodeInternal::SplitBranch(n) => NodeTrait::write(n, builder),
//         }
//     }
// }

pub(crate) trait RcNodeTrait {
    fn add(
        &self,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError>;

    fn register(&self, tree: &mut BytesTrieNodeTree) -> Node;
}

impl RcNodeTrait for Node {
    fn add(
        &self,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        value: i32,
    ) -> Result<Node, BytesTrieBuilderError> {
        <NodeInternal as NodeTrait>::add(self, builder, s, value)
    }

    fn register(&self, tree: &mut BytesTrieNodeTree) -> Node {
        <NodeInternal as NodeTrait>::register(self, tree)
    }
}

impl From<NodeInternal> for Node {
    fn from(node: NodeInternal) -> Self {
        Self(Rc::new(RefCell::new(node)))
    }
}

macro_rules! impl_from {
    ($inner_type:ident, $variant:ident) => {
        impl From<$inner_type> for Node {
            fn from(inner: $inner_type) -> Self {
                Self::$variant(inner)
            }
        }

        // Alas, blanket implementations for `T: Into<Node>` are not allowed, so we implement each
        // manually.
        impl From<$inner_type> for Node {
            fn from(inner: $inner_type) -> Self {
                let node: NodeInternal = inner.into();
                node.into()
            }
        }
    };
}

impl_from!(FinalValueNode, FinalValue);
impl_from!(BranchHeadNode, BranchHead);
impl_from!(DynamicBranchNode, DynamicBranch);
impl_from!(IntermediateValueNode, IntermediateValue);
impl_from!(LinearMatchNode, LinearMatch);
impl_from!(ListBranchNode, ListBranch);
impl_from!(SplitBranchNode, SplitBranch);

/// Allows mutably borrowing a `Node` as a content variant (`C`).
trait GetContent<C: NodeContentTrait> {
    fn content(&self) -> RefMut<'_, C>;
}

macro_rules! impl_get_content {
    ($variant:ident) => {
        paste! {
            impl GetContent<[<$variant Node>]> for Node {
                fn content(&self) -> std::cell::RefMut<'_, [<$variant Node>]> {
                    std::cell::RefMut::map(self.borrow_mut(), |node| match node.content {
                        Node::$variant(inner) => inner,
                        _ => panic!("Assumed wrong variant for {:?}", node)
                    })
                }
            }
        }
    };
}

impl_get_content!(FinalValue);
impl_get_content!(BranchHead);
impl_get_content!(DynamicBranch);
impl_get_content!(IntermediateValue);
impl_get_content!(LinearMatch);
impl_get_content!(ListBranch);
impl_get_content!(SplitBranch);
