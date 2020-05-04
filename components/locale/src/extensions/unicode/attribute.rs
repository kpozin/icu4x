use std::ops::RangeInclusive;
use std::str::FromStr;

use crate::parser::errors::ParserError;
use tinystr::TinyStr8;

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Copy)]
pub struct Attribute(TinyStr8);

const ATTR_LENGTH: RangeInclusive<usize> = 3..=8;

impl Attribute {
    /// A constructor which takes a utf8 slice, parses it and
    /// produces a well-formed `Variant`.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_locale::extensions::unicode::Attribute;
    ///
    /// let attribute = Attribute::from_bytes(b"foobar")
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(attribute, "foobar");
    /// ```
    pub fn from_bytes(v: &[u8]) -> Result<Self, ParserError> {
        if !ATTR_LENGTH.contains(&v.len()) {
            return Err(ParserError::InvalidExtension);
        }

        let s = TinyStr8::from_bytes(v).map_err(|_| ParserError::InvalidExtension)?;

        if !s.is_ascii_alphanumeric() {
            return Err(ParserError::InvalidExtension);
        }

        Ok(Self(s.to_ascii_lowercase()))
    }

    /// A helper function for displaying
    /// a `Attribute` subtag as a `&str`.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_locale::extensions::unicode::Attribute;
    ///
    /// let attribute = Attribute::from_bytes(b"foobar")
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(attribute.as_str(), "foobar");
    /// ```
    ///
    /// `Notice`: For many use cases, such as comparison,
    /// `Attribute` implements `PartialEq<&str>` which allows for direct comparisons.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Attribute {
    type Err = ParserError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(source.as_bytes())
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<&str> for Attribute {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<str> for Attribute {
    fn eq(&self, other: &str) -> bool {
        *self.as_str() == *other
    }
}

impl<'l> From<&'l Attribute> for &'l str {
    fn from(input: &'l Attribute) -> Self {
        input.as_str()
    }
}
