use crate::parser::errors::ParserError;
use std::str::FromStr;
use tinystr::TinyStr4;

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Copy)]
pub struct Key(TinyStr4);

impl Key {
    pub fn from_bytes(key: &[u8]) -> Result<Self, ParserError> {
        if key.len() != 2 || !key[0].is_ascii_alphabetic() || !key[1].is_ascii_digit() {
            return Err(ParserError::InvalidExtension);
        }
        let tkey = TinyStr4::from_bytes(key).map_err(|_| ParserError::InvalidExtension)?;
        Ok(Self(tkey.to_ascii_lowercase()))
    }

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
