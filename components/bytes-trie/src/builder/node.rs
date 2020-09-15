use {
    super::{builder::BytesTrieBuilder, errors::BytesTrieBuilderError},
    std::{fmt::Debug, hash::Hash, rc::Rc},
};

pub(crate) trait AsNode {
    fn as_node(self: Rc<Self>) -> Rc<dyn Node>;
}

impl<T: Node> AsNode for T {
    fn as_node(self: Rc<Self>) -> Rc<dyn Node> {
        self
    }
}

pub(crate) trait Node: AsNode + Debug + 'static {
    fn offset(&self) -> i32;

    fn set_offset(&mut self, offset: i32);

    /// Recursive method for adding a new (string, value) pair. Matches the remaining part of `s`
    /// from `start`, and adds a new node where there is a mismatch.
    ///
    /// Returns `None`, or a replacement `Node`.
    fn add(
        &mut self,
        builder: &mut BytesTrieBuilder,
        s: &str,
        start: i32,
        value: i32,
    ) -> Result<Option<Rc<dyn Node>>, BytesTrieBuilderError> {
        Ok(None)
    }

    /// Recursive method for registering unique nodes, after all (string, value) pairs have been
    /// added. Final-value nodes are pre-registered while `add()`ing (string, value) pairs.
    /// Other nodes created while `add()`ing `register_node()` themselves later and might replace
    /// themselves with new types of nodes for `write()`ing.
    fn register(&mut self, builder: &mut BytesTrieBuilder) -> Option<Rc<Box<dyn Node>>> {
        None
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
    fn write(&mut self, builder: &mut BytesTrieBuilder) {}

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

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct NodeImpl {
    offset: i32,
}

impl Node for NodeImpl {
    fn offset(&self) -> i32 {
        self.offset
    }

    fn set_offset(&mut self, offset: i32) {
        self.offset = offset;
    }
}
