use std::ops::Deref;

use super::Key;
use super::Value;
#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, PartialOrd, Ord)]
pub struct Keywords(Box<[(Key, Value)]>);

impl Keywords {
    pub fn from_vec_unchecked(input: Vec<(Key, Value)>) -> Self {
        Self(input.into_boxed_slice())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: Key) -> bool {
        self.0.binary_search_by_key(&key, |(key, _)| *key).is_ok()
    }

    pub fn get(&self, key: Key) -> Option<&Value> {
        if let Ok(idx) = self.0.binary_search_by_key(&key, |(key, _)| *key) {
            self.0.get(idx).map(|(_, v)| v)
        } else {
            None
        }
    }
}

impl Deref for Keywords {
    type Target = [(Key, Value)];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
