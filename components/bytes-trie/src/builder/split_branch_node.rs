use {
    super::{
        builder::BytesTrieWriter,
        node::{Node, NodeContentTrait},
    },
    std::rc::Rc,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct SplitBranchNode {
    first_edge_number: i32,
    unit: u8,
    less_than: Node,
    greater_or_equal: Node,
}

impl NodeContentTrait for SplitBranchNode {
    fn mark_right_edges_first(&mut self, node: &Node, mut edge_number: i32) -> i32 {
        if node.offset() == 0 {
            self.first_edge_number = edge_number;
            edge_number = self.greater_or_equal.mark_right_edges_first(edge_number);
            edge_number = self.less_than.mark_right_edges_first(edge_number - 1);
            node.set_offset(edge_number);
        }
        edge_number
    }

    fn write(&mut self, node: &Node, writer: &mut BytesTrieWriter) {
        // Encode the less-than branch first.
        self.less_than.write_unless_inside_right_edge(
            self.first_edge_number,
            self.greater_or_equal.offset(),
            writer,
        );
        // Encode the greater-or-equal branch last because we do not jump for it at all.
        self.greater_or_equal.write(writer);
        // Write this node.
        let less_than_offset = self.less_than.offset();
        assert!(less_than_offset > 0);
        writer.write_delta_to(less_than_offset);
        node.set_offset(writer.write_unit(self.unit) as i32);
    }
}

impl SplitBranchNode {
    pub fn new(middle_unit: u8, less_than: Node, greater_or_equal: Node) -> Self {
        Self {
            first_edge_number: 0,
            unit: middle_unit,
            less_than,
            greater_or_equal,
        }
    }
}
