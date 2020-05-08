use std::str::FromStr;

use crate::parser::errors::ParserError;
use tinystr::TinyStr4;

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Copy)]
pub struct Key(TinyStr4);

const KEY_LENGTH: usize = 2;

impl Key {
    /// A constructor which takes a utf8 slice, parses it and
    /// produces a well-formed `Key`.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_locale::extensions::unicode::Key;
    ///
    /// let key = Key::from_bytes(b"ca")
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(key, "ca");
    /// ```
    pub fn from_bytes(key: &[u8]) -> Result<Self, ParserError> {
        if key.len() != KEY_LENGTH
            || !key[0].is_ascii_alphanumeric()
            || !key[1].is_ascii_alphabetic()
        {
            return Err(ParserError::InvalidExtension);
        }

        let key = TinyStr4::from_bytes(key).map_err(|_| ParserError::InvalidSubtag)?;
        Ok(Self(key.to_ascii_lowercase()))
    }

    /// A helper function for displaying
    /// a `Key` subtag as a `&str`.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_locale::extensions::unicode::Key;
    ///
    /// let key: Key = "ca".parse().expect("Parsing failed.");
    ///
    /// assert_eq!(key.as_str(), "ca");
    /// ```
    ///
    /// `Notice`: For many use cases, such as comparison,
    /// `Key` implements `PartialEq<&str>` which allows for direct comparisons.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Key {
    type Err = ParserError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(source.as_bytes())
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<&str> for Key {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<str> for Key {
    fn eq(&self, other: &str) -> bool {
        *self.as_str() == *other
    }
}

impl<'l> From<&'l Key> for &'l str {
    fn from(input: &'l Key) -> Self {
        input.as_str()
    }
}
