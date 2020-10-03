use {
    super::{
        builder::BytesTrieBuilder,
        node::{NodeInternal, NodeTrait, Node, RcNodeTrait, WithOffset},
        value_node::ValueNodeTrait,
    },
    std::rc::Rc,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ListBranchNode {
    first_edge_number: i32,
    equal: Vec<Option<Node>>, // `None` means "has final value"
    length: usize,
    values: Vec<i32>,
    units: Vec<u16>,
}

impl NodeTrait for ListBranchNode {
    fn mark_right_edges_first(&mut self, mut edge_number: i32) -> i32 {
        if self.offset == 0 {
            self.first_edge_number = edge_number;
            let mut step = 0;
            let mut i = self.length;
            loop {
                i -= 1;
                if let Some(edge) = &self.equal[i as usize] {
                    edge_number = edge.borrow_mut().mark_right_edges_first(edge_number - step);
                }
                // For all but the rightmost edge, decrement the edge number.
                step = 1;
                if i <= 0 {
                    break;
                }
            }
            self.offset = edge_number;
        }
        edge_number
    }

    fn write(&mut self, builder: &mut BytesTrieBuilder) {
        // Write the sub-nodes in reverse order: The jump lengths are deltas from after their own
        // positions, so if we wrote the `min_unit` sub-node first, then its jump delta would be
        // larger. Instead we write the `min_unit` sub-node last, for a shorter delta.
        let mut unit_number = self.length - 1;
        let right_edge = &self.equal[unit_number as usize];
        let right_edge_number = match right_edge {
            Some(right_edge) => right_edge.borrow().offset(),
            None => self.first_edge_number,
        };
        loop {
            unit_number -= 1;
            if let Some(node) = &self.equal[unit_number as usize] {
                node.borrow_mut().write_unless_inside_right_edge(
                    self.first_edge_number,
                    right_edge_number,
                    builder,
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
                right_edge.borrow_mut().write(builder);
            }
            None => {
                builder.write_value_and_final(self.values[unit_number as usize], true);
            }
        }

        self.offset = builder.write_unit(self.units[unit_number as usize]);

        // Write the rest of this node's unit-value pairs.
        for unit_number in (0..(unit_number - 1)).rev() {
            let (value, is_final) = match &self.equal[unit_number as usize] {
                Some(node) => {
                    assert!(node.borrow().offset() > 0);
                    (self.offset - node.borrow().offset(), false)
                }
                None => (self.values[unit_number as usize], true),
            };
            builder.write_value_and_final(value, is_final);
            self.offset = builder.write_unit(self.units[unit_number as usize]);
        }
    }
}

impl ListBranchNode {
    pub fn new(capacity: usize) -> Self {
        ListBranchNode {
            offset: 0,
            first_edge_number: 0,
            equal: vec![None; capacity],
            length: 0,
            values: vec![0; capacity],
            units: vec![0; capacity],
        }
    }

    /// Adds a unit with a final value.
    pub fn add_with_final_value(&mut self, c: u16, value: i32) {
        self.units[self.length] = c;
        self.equal[self.length] = None;
        self.values[self.length] = value;
        self.length += 1;
    }

    /// Adds a unit which leads to another match node.
    pub fn add_with_match_node(&mut self, c: u16, node: Node) {
        self.units[self.length] = c;
        self.equal[self.length] = Some(node);
        self.values[self.length] = 0;
        self.length += 1;
    }
}
