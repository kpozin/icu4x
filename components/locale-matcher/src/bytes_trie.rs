use bitflags::_core::intrinsics::offset;
use criterion::Throughput::Bytes;

#[derive(Debug)]
pub struct BytesTrie {
    bytes: Vec<u8>,
    offset: usize,
}

impl<'a> IntoIterator for BytesTrie {
    type Item = BytesTrieEntry<'a>;
    type IntoIter = BytesTrieIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct BytesTrieEntry<'a> {
    key: &'a [u8],
    value: Option<i32>,
}

impl BytesTrieEntry {
    /// The
    pub fn key(&self) -> &[u8] {
        unimplemented!()
    }

    pub fn key_len(&self) -> usize {
        self.key().len()
    }

    /// The value associated with the byte sequence
    pub fn value(&self) -> i32 {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct BytesTrieIterator<'a> {
    bytes: Vec<u8>,
    offset: Option<usize>,
    /// The stack stores state entries for backtracking to another outbound edge of a branch node.
    stack: Vec<StackEntry>,
    key_buffer: KeyBuffer,

    // remaining_match_length:,
    max_key_length: Option<usize>,
}

impl BytesTrieIterator {
    fn branch_next(
        &mut self,
        mut offset: usize,
        mut remaining_branch_length: usize,
    ) -> BranchNextResult {
        while remaining_branch_length > MAX_BRANCH_LINEAR_SUB_NODE_LENGTH as usize {
            // Ignore the comparison byte
            // TODO(kpozin): What comparison byte?
            offset += 1;
            // Push state for the greater-or-equal edge.
            self.push_state(
                BytesTrieIterator::skip_delta(&self.bytes, offset),
                remaining_branch_length - (remaining_branch_length >> 1),
                self.key_buffer.len(),
            );
            // Follow the less-than edge.
            remaining_branch_length >>= 1;
            offset = BytesTrieIterator::jump_by_delta(&self.bytes, offset)
        }

        // List of key-value pairs where values are either final values or jump deltas.
        // Read the first (key, value) pair.

        let key_byte = bytes[offset];
        offset += 1;
        let node = bytes[offset];
        let is_final = CompactValue::is_final(node);

        let value = CompactValue::read_int(&self.bytes, offset);
        offset += CompactValue::get_width(&self.bytes, offset);
        self.push_state(offset, remaining_branch_length - 1, self.key_buffer.len());
        self.key_buffer.push(key_byte);

        if is_final {
            BranchNextResult::FinalValue(value)
        } else {
            BranchNextResult::NewOffset(offset + value as usize)
        }
    }

    fn push_state(&mut self, offset: usize, remaining_branch_length: usize, key_length: usize) {
        self.stack.push(StackEntry {
            offset,
            key_length,
            remaining_branch_length,
        })
    }

    fn is_at_max_key_length(&self) -> bool {
        match self.max_key_length {
            None => false,
            Some(max_length) => max_length == self.key_buffer.len(),
        }
    }

    fn truncate_and_stop(&mut self) -> BytesTrieEntry {
        self.offset = None;
        return BytesTrieEntry {
            key: self.key_buffer.as_slice(),
            value: None,
        };
    }

    fn skip_delta(bytes: &Vec<u8>, offset: usize) -> usize {
        offset + CompactDelta::get_width(bytes, offset) as usize
    }

    fn jump_by_delta(bytes: &Vec<u8>, offset: usize) -> usize {
        let delta = CompactDelta::read_int(bytes, offset);
        assert!(delta >= 0);
        let delta_width_bytes = CompactDelta::get_width(bytes, offset);
        offset + delta_width_bytes + delta as usize
    }
}

impl<'a> Iterator for BytesTrieIterator<'a> {
    type Item = BytesTrieEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = match self.offset {
            None => {
                let top = self.stack.pop();
                match top {
                    None => return None,
                    Some(top) => {
                        self.key_buffer.truncate(top.key_length);
                        if top.remaining_branch_length > 1 {
                            let branch_result =
                                self.branch_next(top.offset, top.remaining_branch_length);
                            match self.branch_next(top.offset, top.remaining_branch_length) {
                                BranchNextResult::FinalValue(final_value) => {
                                    return Some(BytesTrieEntry {
                                        key: self.key_buffer.as_slice(),
                                        value: Some(final_value),
                                    });
                                }
                                BranchNextResult::NewOffset(new_offset) => new_offset,
                            }
                        } else {
                            key_bytes.push(self.bytes[top.offset]);
                            top.offset + 1
                        }
                    }
                }
            }
            Some(offset) => offset,
        };

        // if (remainingMatchLength_ >= 0) {
        //     // We only get here if we started in a pending linear-match node
        //     // with more than maxLength remaining bytes.
        //     return truncateAndStop();
        // }

        loop {
            let mut node = self.bytes[offset];

            if node >= CompactValue::MIN_LEAD as u8 {
                // Deliver value for the byte sequence so far.
                let is_final = CompactValue::is_final(node);
                let value = CompactValue::read_int(&self.bytes, offset);
                if is_final || self.is_at_max_key_length() {
                    self.offset = None
                } else {
                    self.offset = Some(offset + CompactValue::get_width(&self.bytes, offset))
                }
                return Some(BytesTrieEntry {
                    key: self.key_buffer.as_slice(),
                    value: Some(value),
                });
            } else {
                offset += 1;
            }

            if self.is_at_max_key_length() {
                return Some(self.truncate_and_stop());
            }

            if node < MIN_LINEAR_MATCH as u8 {
                if node == 0 {
                    node = bytes[offset];
                    offset += 1;
                }
                offset = match self.branch_next(offset, (node + 1) as usize) {
                    BranchNextResult::FinalValue(final_value) => {
                        return Some(BytesTrieEntry {
                            key: self.key_buffer.as_slice(),
                            value: Some(value),
                        });
                    }
                    BranchNextResult::NewOffset(new_offset) => new_offset,
                }
            } else {
                // Linear-match node, append length bytes.
                let length: usize = node as usize - MIN_LINEAR_MATCH as usize + 1;
                if self.max_key_length.is_some()
                    && (self.key_buffer.len() + length as usize) > self.max_key_length.unwrap()
                {
                    self.key_buffer
                        .extend_from_slice(&self.bytes[offset..(offset + length)]);
                }
                offset += length;
            }
        }
    }
}

/// Holds the current key (or prefix of a key) while traversing the trie.
///
/// The length of `bytes` may at times exceed the actual `length` of the key -- for example, after
/// completing the traversal of a node's less-than branch and popping up to continue to the
/// greater-than branch. Instead of truncating the `Vec` and then resizing it, we just update the
/// `length` field (removing the need for repeated alloc and memcpy operations).
/// allocationsmemcpy,
#[derive(Debug)]
struct KeyBuffer {
    bytes: Vec<u8>,
    length: usize,
}

impl KeyBuffer {
    fn len(&self) -> usize {
        self.length
    }

    fn push(&mut self, key_byte: u8) {
        assert(self.bytes.len() >= self.len());
        self.length += 1;
        if self.bytes.len() < self.length {
            self.bytes.resize(self.length, key_byte);
        } else {
            self.bytes[self.length - 1] = key_byte;
        }
    }

    fn extend_from_slice(&mut self, other: &[u8]) {
        assert!(self.bytes.len() >= self.len());
        let old_length = self.length;
        self.length += other.len();
        self.bytes.resize(self.length, 0);
        for (i, key_byte) in other.iter().enumerate() {
            self.bytes[old_length + i] = *key_byte;
        }
    }

    fn truncate(&mut self, length: usize) {
        assert!(length <= self.len());
        self.length = length;
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes[0..self.len()]
    }

    fn to_vec(&self) -> Vec<u8> {
        self.as_slice().to_owned()
    }
}

enum BranchNextResult {
    /// Final value of a trie branch
    FinalValue(i32),
    /// New offset in the byte array after branching
    NewOffset(usize),
}

/// Entry in the stack that is built while doing a depth-first traversal of the trie.
struct StackEntry {
    /// Position from the beginning of the trie's byte array.
    offset: usize,
    /// Length of the key (string) from before the node.
    key_length: usize,
    /// Remaining length of the branch of the trie.
    remaining_branch_length: usize,
}

// Node lead byte values.

// 00..0f: Branch node. If node!=0 then the length is node+1, otherwise
// the length is one more than the next byte.

// For a branch sub-node with at most this many entries, we drop down
// to a linear search.
const MAX_BRANCH_LINEAR_SUB_NODE_LENGTH: i32 = 5;

// 10..1f: Linear-match node, match 1..16 bytes and continue reading the next node.
const MIN_LINEAR_MATCH: i32 = 0x10;
const MAX_LINEAR_MATCH_LENGTH: i32 = 0x10;

// 20..ff: Variable-length value node.
// If odd, the value is final. (Otherwise, intermediate value or jump delta.)
// Then shift-right by 1 bit.
// The remaining lead byte value indicates the number of following bytes (0..4)
// and contains the value's top bits.

/// It is a final value if bit 0 is set.
const VALUE_IS_FINAL: i32 = 0x1;

trait CompactInt {
    const RIGHT_SHIFT: i32;

    const MAX_ONE_BYTE: i32;
    const MAX_TWO_BYTE: i32;
    const MAX_THREE_BYTE: i32;

    const MIN_ONE_BYTE_LEAD: i32;
    const MIN_TWO_BYTE_LEAD: i32;
    const MIN_THREE_BYTE_LEAD: i32;
    const FOUR_BYTE_LEAD: i32;
    const FIVE_BYTE_LEAD: i32;

    /// Reads the compact integer starting at the given position in the byte array.
    fn read_int(bytes: &Vec<u8>, offset: usize) -> i32 {
        let lead_byte: i32 = bytes[offset] as i32 >> Self::RIGHT_SHIFT;
        if lead_byte < Self::MIN_TWO_BYTE_LEAD {
            lead_byte - Self::MIN_ONE_BYTE_LEAD
        } else if lead_byte < Self::MIN_THREE_BYTE_LEAD {
            ((lead_byte - Self::MIN_TWO_BYTE_LEAD) << 8) | bytes[offset + 1] as i32
        } else if lead_byte < Self::FOUR_BYTE_LEAD {
            ((lead_byte - Self::MIN_THREE_BYTE_LEAD) << 16)
                | ((bytes[offset + 1] as i32) << 8)
                | (bytes[offset + 2] as i32)
        } else if lead_byte == Self::FOUR_BYTE_LEAD {
            ((bytes[pos + 1] as i32) << 16)
                | ((bytes[pos + 2] as i32) << 8)
                | (bytes[pos + 3] as i32)
        } else {
            ((bytes[pos + 1] as i32) << 24)
                | ((bytes[pos + 2] as i32) << 16)
                | ((bytes[pos + 3] as i32) << 8)
                | (bytes[pos + 4] as i32)
        }
    }

    /// Calculates the width in bytes of the compact integer starting at the given position in the
    /// byte array.
    fn get_width(bytes: &Vec<u8>, offset: usize) -> usize;
}

/// Compact value: After testing bit 0, shift right by 1 and then use the following thresholds.
struct CompactValue {}

impl CompactValue {
    /// 0x20
    const MIN_LEAD: i32 = MIN_LINEAR_MATCH + MAX_LINEAR_MATCH_LENGTH;

    fn is_final(byte: u8) -> bool {
        (byte & VALUE_IS_FINAL as u8) != 0
    }
}

impl CompactInt for CompactValue {
    const RIGHT_SHIFT: i32 = 1;

    /// At least 6 bits in the first byte.
    const MAX_ONE_BYTE: i32 = 0x40;
    const MAX_TWO_BYTE: i32 = 0x1aff;
    /// A little more than Unicode code points. (0x11ffff)
    const MAX_THREE_BYTE: i32 =
        ((Self::FOUR_BYTE_LEAD as i32 - Self::MIN_THREE_BYTE_LEAD) << 16) - 1;

    /// 0x10
    const MIN_ONE_BYTE_LEAD: i32 = Self::MIN_LEAD / 2;
    /// 0x51
    const MIN_TWO_BYTE_LEAD: i32 = Self::MIN_ONE_BYTE_LEAD + Self::MAX_ONE_BYTE;
    /// 0x6c
    const MIN_THREE_BYTE_LEAD: i32 = Self::MIN_TWO_BYTE_LEAD as i32 + (Self::MAX_TWO_BYTE >> 8) + 1;
    const FOUR_BYTE_LEAD: i32 = 0x7e;
    const FIVE_BYTE_LEAD: i32 = 0x7f;

    fn get_width(bytes: &Vec<u8>, offset: usize) -> usize {
        let lead_byte = bytes[offset] as i32;
        assert!(lead_byte >= Self::MIN_LEAD);

        if lead_byte < (Self::MIN_TWO_BYTE_LEAD << Self::RIGHT_SHIFT) {
            1
        } else if lead_byte < (Self::MIN_THREE_BYTE_LEAD << Self::RIGHT_SHIFT) {
            2
        } else if lead_byte < (Self::FOUR_BYTE_LEAD << Self::RIGHT_SHIFT) {
            3
        } else {
            4 + ((lead_byte >> Self::RIGHT_SHIFT) & 1)
        }
        unimplemented!()
    }
}

/// Compact delta integers.
struct CompactDelta {}

impl CompactDelta {}

impl CompactInt for CompactDelta {
    const RIGHT_SHIFT: i32 = 0;

    const MAX_ONE_BYTE: i32 = 0xbf;
    /// 0x2fff
    const MAX_TWO_BYTE: i32 =
        ((Self::MIN_THREE_BYTE_LEAD - Self::MIN_TWO_BYTE_LEAD as i32) << 8) - 1;
    const MAX_THREE_BYTE: i32 =
        ((Self::FOUR_BYTE_LEAD as i32 - Self::MIN_THREE_BYTE_LEAD) << 16) - 1;

    /// Does not apply for `CompactDelta`.
    const MIN_ONE_BYTE_LEAD: i32 = 0;
    /// 0xc0
    const MIN_TWO_BYTE_LEAD: i32 = Self::MAX_ONE_BYTE + 1;
    const MIN_THREE_BYTE_LEAD: i32 = 0xf0;
    const FOUR_BYTE_LEAD: i32 = 0xfe;
    const FIVE_BYTE_LEAD: i32 = 0xff;

    fn get_width(bytes: &Vec<u8>, offset: usize) -> usize {
        let lead_byte = bytes[offset] as i32;
        if lead_byte < Self::MIN_TWO_BYTE_LEAD {
            1
        } else if lead_byte < Self::MIN_THREE_BYTE_LEAD {
            2
        } else if lead_byte < Self::FOUR_BYTE_LEAD {
            3
        } else {
            4 + (lead_byte & 1)
        }
    }
}
