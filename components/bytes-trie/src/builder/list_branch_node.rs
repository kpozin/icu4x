use {
    super::{
        builder::BytesTrieWriter,
        node::{Node, NodeContentTrait, NodeInternal},
        value_node::ValueNodeContentTrait,
    },
    std::rc::Rc,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct ListBranchNode {
    first_edge_number: i32,
    equal: Vec<Option<Node>>, // `None` means "has final value"
    length: usize,
    values: Vec<i32>,
    units: Vec<u8>,
}

impl NodeContentTrait for ListBranchNode {
    fn mark_right_edges_first(&mut self, node: &Node, mut edge_number: i32) -> i32 {
        if node.offset() == 0 {
            self.first_edge_number = edge_number;
            let mut step = 0;
            let mut i = self.length;
            loop {
                i -= 1;
                if let Some(edge) = &self.equal[i as usize] {
                    edge_number = edge.mark_right_edges_first(edge_number - step);
                }
                // For all but the rightmost edge, decrement the edge number.
                step = 1;
                if i <= 0 {
                    break;
                }
            }
            node.set_offset(edge_number);
        }
        edge_number
    }

    fn write(&mut self, node: &Node, writer: &mut BytesTrieWriter) {
        // Write the sub-nodes in reverse order: The jump lengths are deltas from after their own
        // positions, so if we wrote the `min_unit` sub-node first, then its jump delta would be
        // larger. Instead we write the `min_unit` sub-node last, for a shorter delta.
        let mut unit_number = self.length - 1;
        let right_edge = &self.equal[unit_number as usize];
        let right_edge_number = match right_edge {
            Some(right_edge) => right_edge.offset(),
            None => self.first_edge_number,
        };
        loop {
            unit_number -= 1;
            if let Some(node) = &self.equal[unit_number as usize] {
                node.write_unless_inside_right_edge(
                    self.first_edge_number,
                    right_edge_number,
                    writer,
                );
            }
            if unit_number <= 0 {
                break;
            }
        }

        // The `max_unit` sub-node is written as the very last one because we do not jump for it at
        // all.
        unit_number = self.length - 1;
        match right_edge {
            Some(right_edge) => {
                right_edge.write(writer);
            }
            None => {
                writer.write_value_and_final(Some(self.values[unit_number as usize]), true);
            }
        }

        node.set_offset(writer.write_unit(self.units[unit_number as usize]) as i32);

        // Write the rest of this node's unit-value pairs.
        for unit_number in (0..(unit_number - 1)).rev() {
            let (value, is_final) = match &self.equal[unit_number as usize] {
                Some(equal_node) => {
                    assert!(equal_node.offset() > 0);
                    (node.offset() - equal_node.offset(), false)
                }
                None => (self.values[unit_number as usize], true),
            };
            writer.write_value_and_final(Some(value), is_final);
            node.set_offset(writer.write_unit(self.units[unit_number as usize]) as i32);
        }
    }
}

impl ListBranchNode {
    pub fn new(capacity: usize) -> Self {
        ListBranchNode {
            first_edge_number: 0,
            equal: vec![None; capacity],
            length: 0,
            values: vec![0; capacity],
            units: vec![0; capacity],
        }
    }

    /// Adds a unit with a final value.
    pub fn add_with_final_value(&mut self, c: u8, value: i32) {
        self.units[self.length] = c;
        self.equal[self.length] = None;
        self.values[self.length] = value;
        self.length += 1;
    }

    /// Adds a unit which leads to another match node.
    pub fn add_with_match_node(&mut self, c: u8, node: Node) {
        self.units[self.length] = c;
        self.equal[self.length] = Some(node);
        self.values[self.length] = 0;
        self.length += 1;
    }
}
