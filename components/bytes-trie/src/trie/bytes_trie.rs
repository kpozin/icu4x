use crate::trie::encoding::*;

#[derive(Debug)]
pub struct BytesTrie {
    bytes: Vec<u8>,
}

impl BytesTrie {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        BytesTrie { bytes }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

#[derive(Debug)]
pub struct BytesTrieEntry<'a> {
    key: &'a [u8],
    value: Option<i32>,
}

impl<'a> BytesTrieEntry<'a> {
    /// The
    pub fn key(&'a self) -> &'a [u8] {
        self.key
    }

    pub fn key_len(&self) -> usize {
        self.key().len()
    }

    /// The value associated with the byte sequence
    pub fn value(&self) -> Option<i32> {
        self.value
    }
}

/// Entry in the stack that is built while doing a depth-first traversal of the trie.
struct StackEntry {
    /// Position from the beginning of the trie's byte array.
    position: usize,
    /// Length of the key (string) from before the node.
    key_length: usize,
    /// Remaining length of the branch of the trie.
    remaining_branch_length: usize,
}

enum BranchNextResult {
    /// Final value of a trie branch
    FinalValue(i32),
    /// New offset in the byte array after branching
    NewPosition(usize),
}

pub struct BytesTrieIterator<'a> {
    /// Reference to the BytesTrie's contents.
    bytes: &'a [u8],
    position: Option<usize>,

    /// The stack stores state entries for backtracking to another outbound edge of a branch node.
    stack: Vec<StackEntry>,

    key_buffer: Vec<u8>,
    max_key_length: Option<usize>,
}

impl<'a> BytesTrieIterator<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: Some(0),
            stack: vec![],
            key_buffer: vec![],
            max_key_length: None,
        }
    }

    pub fn next(&mut self) -> Option<BytesTrieEntry> {
        let mut position = match self.position {
            None => {
                // Pop the state off the stack and continue with the next outbound edge of the
                // branch node.
                let stack_top = self.stack.pop();
                match stack_top {
                    None => {
                        return None;
                    }
                    Some(stack_top) => {
                        self.key_buffer.truncate(stack_top.key_length);
                        if stack_top.remaining_branch_length > 1 {
                            match self
                                .branch_next(stack_top.position, stack_top.remaining_branch_length)
                            {
                                BranchNextResult::FinalValue(final_value) => {
                                    return Some(BytesTrieEntry {
                                        key: self.key_bytes(),
                                        value: Some(final_value),
                                    });
                                }
                                BranchNextResult::NewPosition(new_position) => new_position,
                            }
                        } else {
                            self.key_buffer.push(self.bytes[stack_top.position]);
                            stack_top.position + 1
                        }
                    }
                }
            }
            Some(current_position) => current_position,
        };

        // Diff from Java/CPP: Not porting remaining_match_length.

        loop {
            let mut node = self.bytes[position];
            position += 1;

            if node >= CompactValue::MIN_LEAD {
                // Deliver value for the byte sequence so far.
                let is_final = CompactValue::is_final(node);
                let value = CompactValue::read_int(&self.bytes, position);
                if is_final || self.is_at_max_key_length() {
                    self.position = None;
                } else {
                    self.position = Some(position + CompactValue::get_width(&self.bytes, position));
                }
                return Some(BytesTrieEntry {
                    key: self.key_bytes(),
                    value: Some(value),
                });
            }

            if self.is_at_max_key_length() {
                return Some(self.truncate_and_stop());
            }

            if node < MIN_LINEAR_MATCH {
                if node == 0 {
                    node = self.bytes[position];
                    position += 1;
                }
                position = match self.branch_next(position, (node + 1) as usize) {
                    BranchNextResult::FinalValue(final_value) => {
                        return Some(BytesTrieEntry {
                            key: self.key_bytes(),
                            value: Some(final_value),
                        });
                    }
                    BranchNextResult::NewPosition(new_position) => new_position,
                };
            } else
            /* if node >= MIN_LINEAR_MATCH */
            {
                // Linear-match node, append `added_length` bytes to buffer.
                let added_length = (node - MIN_LINEAR_MATCH + 1) as usize;
                match self.max_key_length {
                    Some(max_key_length) if self.key_length() + added_length > max_key_length => {
                        self.key_buffer.extend_from_slice(
                            &self.bytes[position..position + max_key_length - self.key_length()],
                        );
                        return Some(self.truncate_and_stop());
                    }
                    _ => {
                        self.key_buffer
                            .extend_from_slice(&self.bytes[position..position + added_length]);
                        position += added_length;
                    }
                }
            }
        }
    }

    fn branch_next(
        &mut self,
        mut position: usize,
        mut remaining_branch_length: usize,
    ) -> BranchNextResult {
        while remaining_branch_length > MAX_BRANCH_LINEAR_SUB_NODE_LENGTH as usize {
            // Ignore the comparison byte
            // TODO(kpozin): What comparison byte?
            position += 1;
            // Push state for the greater-or-equal edge.
            {
                let saved_position = BytesTrieIterator::skip_delta(self.bytes, position);
                let saved_remaining_branch_length =
                    remaining_branch_length - (remaining_branch_length >> 1);
                let saved_key_length = self.key_buffer.len();
                self.push_state(
                    saved_position,
                    saved_remaining_branch_length,
                    saved_key_length,
                );
            }
            // Follow the less-than edge.
            remaining_branch_length >>= 1;
            position = BytesTrieIterator::jump_by_delta(self.bytes, position);
        }

        // List of key-value pairs where values are either final values or jump deltas.
        // Read the first (key, value) pair.

        let key_byte = self.bytes[position];
        position += 1;
        let node = self.bytes[position];
        let is_final = CompactValue::is_final(node);

        let value = CompactValue::read_int(self.bytes, position);
        position += CompactValue::get_width(self.bytes, position);
        self.push_state(position, remaining_branch_length - 1, self.key_buffer.len());
        self.key_buffer.push(key_byte);

        if is_final {
            BranchNextResult::FinalValue(value)
        } else {
            BranchNextResult::NewPosition(position + value as usize)
        }
    }

    fn push_state(&mut self, position: usize, remaining_branch_length: usize, key_length: usize) {
        self.stack.push(StackEntry {
            position,
            key_length,
            remaining_branch_length,
        })
    }

    fn key_bytes(&self) -> &[u8] {
        self.key_buffer.as_slice()
    }

    fn key_length(&self) -> usize {
        self.key_buffer.len()
    }

    fn is_at_max_key_length(&self) -> bool {
        match self.max_key_length {
            None => false,
            Some(max_length) => max_length == self.key_length(),
        }
    }

    fn truncate_and_stop(&mut self) -> BytesTrieEntry {
        self.position = None;
        return BytesTrieEntry {
            key: self.key_bytes(),
            value: None,
        };
    }

    fn skip_delta(bytes: &[u8], offset: usize) -> usize {
        offset + CompactDelta::get_width(bytes, offset) as usize
    }

    fn jump_by_delta(bytes: &[u8], offset: usize) -> usize {
        let delta = CompactDelta::read_int(bytes, offset);
        assert!(delta >= 0);
        let delta_width_bytes = CompactDelta::get_width(bytes, offset);
        offset + delta_width_bytes + delta as usize
    }
}
