use {
    super::{
        builder::BytesTrieBuilder,
        node::{Node, NodeTrait, RcNode, RcNodeTrait, WithOffset},
    },
    std::rc::Rc,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SplitBranchNode {
    pub(crate) offset: i32,
    first_edge_number: i32,
    unit: u16,
    less_than: RcNode,
    greater_or_equal: RcNode,
}

impl NodeTrait for SplitBranchNode {
    fn mark_right_edges_first(&mut self, mut edge_number: i32) -> i32 {
        if self.offset == 0 {
            self.first_edge_number = edge_number;
            edge_number = self
                .greater_or_equal
                .borrow_mut()
                .mark_right_edges_first(edge_number);
            edge_number = self
                .less_than
                .borrow_mut()
                .mark_right_edges_first(edge_number - 1);
            self.offset = edge_number;
        }
        edge_number
    }

    fn write(&mut self, builder: &mut super::builder::BytesTrieBuilder) {
        // Encode the less-than branch first.
        self.less_than.borrow_mut().write_unless_inside_right_edge(
            self.first_edge_number,
            self.greater_or_equal.borrow().offset(),
            builder,
        );
        // Encode the greater-or-equal branch last because we do not jump for it at all.
        self.greater_or_equal.borrow_mut().write(builder);
        // Write this node.
        let less_than_offset = self.less_than.borrow().offset();
        assert!(less_than_offset > 0);
        builder.write_delta_to(less_than_offset);
        self.offset = builder.write_unit(self.unit);
    }
}

impl SplitBranchNode {
    pub fn new(middle_unit: u16, less_than: RcNode, greater_or_equal: RcNode) -> Self {
        Self {
            offset: 0,
            first_edge_number: 0,
            unit: middle_unit,
            less_than,
            greater_or_equal,
        }
    }
}
