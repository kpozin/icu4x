use std::ops::RangeInclusive;
use std::str::FromStr;

use crate::parser::errors::ParserError;
use tinystr::TinyStr8;

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Copy)]
pub struct Key(TinyStr8);

const KEY_LENGTH: RangeInclusive<usize> = 1..=8;

impl Key {
    /// A constructor which takes a utf8 slice, parses it and
    /// produces a well-formed `Variant`.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_locale::extensions::private::Key;
    ///
    /// let attribute = Key::from_bytes(b"foobar")
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(attribute.as_str(), "foobar");
    /// ```
    pub fn from_bytes(v: &[u8]) -> Result<Self, ParserError> {
        if !KEY_LENGTH.contains(&v.len()) {
            return Err(ParserError::InvalidExtension);
        }

        let s = TinyStr8::from_bytes(v).map_err(|_| ParserError::InvalidExtension)?;

        if !s.is_ascii_alphanumeric() {
            return Err(ParserError::InvalidExtension);
        }

        Ok(Self(s.to_ascii_lowercase()))
    }

    /// A helper function for displaying
    /// a `Key` subtag as a `&str`.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_locale::extensions::private::Key;
    ///
    /// let attribute = Key::from_bytes(b"foobar")
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(attribute.as_str(), "foobar");
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
