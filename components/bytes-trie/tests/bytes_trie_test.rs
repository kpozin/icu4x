use {
    assert_matches::assert_matches,
    icu_bytes_trie::{BytesTrie, BytesTrieBuilder},
};

struct Entry {
    s: String,
    value: Option<i32>,
}

impl Entry {
    fn new(s: String, value: Option<i32>) -> Self {
        Self { s, value }
    }

    fn string(&self) -> &str {
        &self.s[..]
    }

    fn bytes(&self) -> &[u8] {
        self.s.as_bytes()
    }

    fn value(&self) -> Option<i32> {
        self.value
    }
}

#[test]
fn test_builder_empty() {
    let builder = BytesTrieBuilder::new();
    let result = builder.build_fast();
    assert_matches!(result, Err(_));
}
