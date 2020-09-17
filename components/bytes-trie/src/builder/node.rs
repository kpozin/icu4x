use {
    super::{
        branch_head_node::BranchHeadNode, builder::BytesTrieBuilder,
        dynamic_branch_node::DynamicBranchNode, errors::BytesTrieBuilderError,
        final_value_node::FinalValueNode, intermediate_value_node::IntermediateValueNode,
        linear_match_node::LinearMatchNode, list_branch_node::ListBranchNode,
        split_branch_node::SplitBranchNode,
    },
    paste::paste,
    std::{cell::RefCell, fmt::Debug, hash::Hash, rc::Rc},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Node {
    FinalValue(FinalValueNode),
    BranchHead(BranchHeadNode),
    DynamicBranch(DynamicBranchNode),
    IntermediateValue(IntermediateValueNode),
    LinearMatch(LinearMatchNode),

    ListBranch(ListBranchNode),
    SplitBranch(SplitBranchNode),
}

pub(crate) type RcNode = Rc<RefCell<Node>>;

impl Node {
    pub fn is_value_node(&self) -> bool {
        match self {
            Node::FinalValue(_)
            | Node::BranchHead(_)
            | Node::DynamicBranch(_)
            | Node::IntermediateValue(_)
            | Node::LinearMatch(_) => true,
            _ => false,
        }
    }

    pub fn is_branch_node(&self) -> bool {
        !self.is_value_node()
    }
}

pub(crate) trait WithOffset {
    fn offset(&self) -> i32;
    fn set_offset(&mut self, offset: i32);
}

impl WithOffset for Node {
    fn offset(&self) -> i32 {
        match self {
            Node::FinalValue(n) => n.offset(),
            Node::BranchHead(n) => n.offset(),
            Node::DynamicBranch(n) => n.offset(),
            Node::IntermediateValue(n) => n.offset(),
            Node::LinearMatch(n) => n.offset(),
            Node::ListBranch(n) => n.offset(),
            Node::SplitBranch(n) => n.offset(),
        }
    }

    fn set_offset(&mut self, offset: i32) {
        match self {
            Node::FinalValue(n) => n.set_offset(offset),
            Node::BranchHead(n) => n.set_offset(offset),
            Node::DynamicBranch(n) => n.set_offset(offset),
            Node::IntermediateValue(n) => n.set_offset(offset),
            Node::LinearMatch(n) => n.set_offset(offset),
            Node::ListBranch(n) => n.set_offset(offset),
            Node::SplitBranch(n) => n.set_offset(offset),
        }
    }
}

macro_rules! impl_with_offset {
    ($variant:ident) => {
        impl WithOffset for $variant {
            fn offset(&self) -> i32 {
                self.offset
            }

            fn set_offset(&mut self, offset: i32) {
                self.offset = offset;
            }
        }
    };
}

impl_with_offset!(BranchHeadNode);
impl_with_offset!(DynamicBranchNode);
impl_with_offset!(FinalValueNode);
impl_with_offset!(IntermediateValueNode);
impl_with_offset!(LinearMatchNode);
impl_with_offset!(ListBranchNode);
impl_with_offset!(SplitBranchNode);

pub(crate) trait NodeTrait: WithOffset + Debug + Eq + PartialEq + 'static {
    /// Recursive method for adding a new (string, value) pair. Matches the remaining part of `s`
    /// from `start`, and adds a new node where there is a mismatch.
    ///
    /// Returns `None`, or a replacement `Node`.
    fn add(
        self_: &RcNode,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        start: i32,
        value: i32,
    ) -> Result<RcNode, BytesTrieBuilderError> {
        Ok(self_.clone())
    }

    /// Recursive method for registering unique nodes, after all (string, value) pairs have been
    /// added. Final-value nodes are pre-registered while `add()`ing (string, value) pairs.
    /// Other nodes created while `add()`ing `register_node()` themselves later and might replace
    /// themselves with new types of nodes for `write()`ing.
    ///
    /// Returns the registered version of this node which implements `write()`, or `None` if self
    /// is the instance registered.
    fn register(self_: &RcNode, builder: &mut BytesTrieBuilder) -> RcNode {
        self_.clone()
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

impl NodeTrait for Node {
    fn register(self_: &RcNode, builder: &mut BytesTrieBuilder) -> RcNode {
        match *self_.borrow() {
            Node::FinalValue(_) => <FinalValueNode as NodeTrait>::register(self_, builder),
            Node::BranchHead(_) => <BranchHeadNode as NodeTrait>::register(self_, builder),
            Node::DynamicBranch(_) => <DynamicBranchNode as NodeTrait>::register(self_, builder),
            Node::IntermediateValue(_) => {
                <IntermediateValueNode as NodeTrait>::register(self_, builder)
            }
            Node::LinearMatch(_) => <LinearMatchNode as NodeTrait>::register(self_, builder),
            Node::ListBranch(_) => <ListBranchNode as NodeTrait>::register(self_, builder),
            Node::SplitBranch(_) => <SplitBranchNode as NodeTrait>::register(self_, builder),
        }
    }

    fn write(&mut self, builder: &mut BytesTrieBuilder) {
        match self {
            Node::FinalValue(n) => NodeTrait::write(n, builder),
            Node::BranchHead(n) => NodeTrait::write(n, builder),
            Node::DynamicBranch(n) => NodeTrait::write(n, builder),
            Node::IntermediateValue(n) => NodeTrait::write(n, builder),
            Node::LinearMatch(n) => NodeTrait::write(n, builder),
            Node::ListBranch(n) => NodeTrait::write(n, builder),
            Node::SplitBranch(n) => NodeTrait::write(n, builder),
        }
    }
}

pub(crate) trait RcNodeTrait {
    fn add(
        &self,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        start: i32,
        value: i32,
    ) -> Result<RcNode, BytesTrieBuilderError>;

    fn register(&self, builder: &mut BytesTrieBuilder) -> RcNode;
}

impl RcNodeTrait for RcNode {
    fn add(
        &self,
        builder: &mut BytesTrieBuilder,
        s: &[u16],
        start: i32,
        value: i32,
    ) -> Result<RcNode, BytesTrieBuilderError> {
        <Node as NodeTrait>::add(self, builder, s, start, value)
    }

    fn register(&self, builder: &mut BytesTrieBuilder) -> RcNode {
        <Node as NodeTrait>::register(self, builder)
    }
}

impl From<Node> for RcNode {
    fn from(node: Node) -> Self {
        Rc::new(RefCell::new(node))
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
        impl From<$inner_type> for RcNode {
            fn from(inner: $inner_type) -> Self {
                let node: Node = inner.into();
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

macro_rules! impl_as_inner {
    ($variant:ident) => {
        paste! {
            #[doc = "Allows mutably borrowing an `RcNode` as a `" $variant "Node`."]
            pub(crate) trait [<As $variant>] {
                fn [<as_ $variant:snake>] (&self) -> std::cell::RefMut<'_, [<$variant Node>]>;
                fn inner(&self) -> std::cell::RefMut<'_, [<$variant Node>]>;
            }

            impl [<As $variant>] for RcNode {
                fn [<as_ $variant:snake>] (&self) -> std::cell::RefMut<'_, [<$variant Node>]> {
                    std::cell::RefMut::map(self.borrow_mut(), |node| match node {
                        Node::$variant(inner) => inner,
                        _ => panic!("Assumed wrong variant for {:?}", node)
                    })
                }

                fn inner(&self) -> std::cell::RefMut<'_, [<$variant Node>]> {
                    self.[<as_ $variant:snake>]()
                }
            }
        }
    };
}

impl_as_inner!(FinalValue);
impl_as_inner!(BranchHead);
impl_as_inner!(DynamicBranch);
impl_as_inner!(IntermediateValue);
impl_as_inner!(LinearMatch);
impl_as_inner!(ListBranch);
impl_as_inner!(SplitBranch);
