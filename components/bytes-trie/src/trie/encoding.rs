use std::convert::TryInto;

/// Unfortunately, after serialization, there is no way to distinguish between an empty value and an
/// actual value of `-1`.
pub(crate) const EMPTY_VALUE: i32 = -1;

// Node lead byte values.

// 00..0f: Branch node. If node!=0 then the length is node+1, otherwise
// the length is one more than the next byte.

// For a branch sub-node with at most this many entries, we drop down
// to a linear search.
pub(crate) const MAX_BRANCH_LINEAR_SUB_NODE_LENGTH: u8 = 5;

// 10..1f: Linear-match node, match 1..16 bytes and continue reading the next node.
pub(crate) const MIN_LINEAR_MATCH: u8 = 0x10;
pub(crate) const MAX_LINEAR_MATCH_LENGTH: u8 = 0x10;

// 20..ff: Variable-length value node.
// If odd, the value is final. (Otherwise, intermediate value or jump delta.)
// Then shift-right by 1 bit.
// The remaining lead byte value indicates the number of following bytes (0..4)
// and contains the value's top bits.

/// It is a final value if bit 0 is set.
pub(crate) const VALUE_IS_FINAL: u8 = 0x1;

pub(crate) trait CompactInt {
    const RIGHT_SHIFT: u8;

    const MAX_ONE_BYTE: u8;
    const MAX_TWO_BYTE: i32;
    const MAX_THREE_BYTE: i32;

    const MIN_ONE_BYTE_LEAD: u8;
    const MIN_TWO_BYTE_LEAD: u8;
    const MIN_THREE_BYTE_LEAD: u8;
    const FOUR_BYTE_LEAD: u8;
    const FIVE_BYTE_LEAD: u8;

    /// Reads the compact integer starting at the given position in the byte array.
    fn read_int(bytes: &[u8], offset: usize) -> i32 {
        let lead_byte: u8 = bytes[offset] >> Self::RIGHT_SHIFT;
        if lead_byte < Self::MIN_TWO_BYTE_LEAD {
            (lead_byte - Self::MIN_ONE_BYTE_LEAD) as i32
        } else if lead_byte < Self::MIN_THREE_BYTE_LEAD {
            ((lead_byte as i32 - Self::MIN_TWO_BYTE_LEAD as i32) << 8) | (bytes[offset + 1] as i32)
        } else if lead_byte < Self::FOUR_BYTE_LEAD {
            ((lead_byte as i32 - Self::MIN_THREE_BYTE_LEAD as i32) << 16)
                | ((bytes[offset + 1] as i32) << 8)
                | (bytes[offset + 2] as i32)
        } else if lead_byte == Self::FOUR_BYTE_LEAD {
            ((bytes[offset + 1] as i32) << 16)
                | ((bytes[offset + 2] as i32) << 8)
                | (bytes[offset + 3] as i32)
        } else {
            ((bytes[offset + 1] as i32) << 24)
                | ((bytes[offset + 2] as i32) << 16)
                | ((bytes[offset + 3] as i32) << 8)
                | (bytes[offset + 4] as i32)
        }
    }

    /// Calculates the width in bytes of the compact integer starting at the given position in the
    /// byte array.
    fn get_width(bytes: &[u8], offset: usize) -> usize;
}

/// Compact value: After testing bit 0, shift right by 1 and then use the following thresholds.
pub(crate) struct CompactValue {}

impl CompactValue {
    /// 0x20
    pub(crate) const MIN_LEAD: u8 = MIN_LINEAR_MATCH + MAX_LINEAR_MATCH_LENGTH;

    pub(crate) fn is_final(byte: u8) -> bool {
        (byte & VALUE_IS_FINAL as u8) != 0
    }
}

impl CompactInt for CompactValue {
    const RIGHT_SHIFT: u8 = 1;

    /// At least 6 bits in the first byte.
    const MAX_ONE_BYTE: u8 = 0x40;
    const MAX_TWO_BYTE: i32 = 0x1aff;
    /// A little more than Unicode code points. (0x11ffff)
    const MAX_THREE_BYTE: i32 =
        ((Self::FOUR_BYTE_LEAD as i32 - Self::MIN_THREE_BYTE_LEAD as i32) << 16) - 1;

    /// 0x10
    const MIN_ONE_BYTE_LEAD: u8 = Self::MIN_LEAD / 2;
    /// 0x51
    const MIN_TWO_BYTE_LEAD: u8 = Self::MIN_ONE_BYTE_LEAD + Self::MAX_ONE_BYTE;
    /// 0x6c
    const MIN_THREE_BYTE_LEAD: u8 = Self::MIN_TWO_BYTE_LEAD + (Self::MAX_TWO_BYTE >> 8) as u8 + 1;
    const FOUR_BYTE_LEAD: u8 = 0x7e;
    const FIVE_BYTE_LEAD: u8 = 0x7f;

    fn get_width(bytes: &[u8], offset: usize) -> usize {
        let lead_byte = bytes[offset];
        assert!(lead_byte >= Self::MIN_LEAD);

        if lead_byte < (Self::MIN_TWO_BYTE_LEAD << Self::RIGHT_SHIFT) {
            1
        } else if lead_byte < (Self::MIN_THREE_BYTE_LEAD << Self::RIGHT_SHIFT) {
            2
        } else if lead_byte < (Self::FOUR_BYTE_LEAD << Self::RIGHT_SHIFT) {
            3
        } else {
            (4 + ((lead_byte >> Self::RIGHT_SHIFT) & 1))
                .try_into()
                .unwrap()
        }
    }
}

/// Compact delta integers.
pub(crate) struct CompactDelta {}

impl CompactDelta {}

impl CompactInt for CompactDelta {
    const RIGHT_SHIFT: u8 = 0;

    const MAX_ONE_BYTE: u8 = 0xbf;
    /// 0x2fff
    const MAX_TWO_BYTE: i32 =
        ((Self::MIN_THREE_BYTE_LEAD as i32 - Self::MIN_TWO_BYTE_LEAD as i32) << 8) - 1;
    const MAX_THREE_BYTE: i32 =
        ((Self::FOUR_BYTE_LEAD as i32 - Self::MIN_THREE_BYTE_LEAD as i32) << 16) - 1;

    /// Does not apply for `CompactDelta`.
    const MIN_ONE_BYTE_LEAD: u8 = 0;
    /// 0xc0
    const MIN_TWO_BYTE_LEAD: u8 = Self::MAX_ONE_BYTE + 1;
    const MIN_THREE_BYTE_LEAD: u8 = 0xf0;
    const FOUR_BYTE_LEAD: u8 = 0xfe;
    const FIVE_BYTE_LEAD: u8 = 0xff;

    fn get_width(bytes: &[u8], offset: usize) -> usize {
        let lead_byte = bytes[offset];
        if lead_byte < Self::MIN_TWO_BYTE_LEAD {
            1
        } else if lead_byte < Self::MIN_THREE_BYTE_LEAD {
            2
        } else if lead_byte < Self::FOUR_BYTE_LEAD {
            3
        } else {
            (4 + (lead_byte & 1)) as usize
        }
    }
}
