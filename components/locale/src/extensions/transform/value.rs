use crate::parser::ParserError;
use std::ops::RangeInclusive;
use std::str::FromStr;
use tinystr::TinyStr8;

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct Value(Box<[TinyStr8]>);

const TYPE_LENGTH: RangeInclusive<usize> = 3..=8;
const TRUE_TVALUE: TinyStr8 = unsafe { TinyStr8::new_unchecked(1_702_195_828u64) }; // "true"

impl Value {
    pub fn from_bytes(input: &[u8]) -> Result<Self, ParserError> {
        let mut v = vec![];
        for subtag in input.split(|c| *c == b'-' || *c == b'_') {
            if !Self::is_type_subtag(subtag) {
                return Err(ParserError::InvalidExtension);
            }
            v.push(TinyStr8::from_bytes(subtag).map_err(|_| ParserError::InvalidExtension)?);
        }
        Ok(Self(v.into_boxed_slice()))
    }

    pub fn from_vec_unchecked(input: Vec<TinyStr8>) -> Self {
        Self(input.into_boxed_slice())
    }

    pub fn is_type_subtag(t: &[u8]) -> bool {
        TYPE_LENGTH.contains(&t.len()) && !t.iter().any(|c: &u8| !c.is_ascii_alphanumeric())
    }

    pub fn parse_subtag(t: &[u8]) -> Result<Option<TinyStr8>, ParserError> {
        let s = TinyStr8::from_bytes(t).map_err(|_| ParserError::InvalidSubtag)?;
        if !TYPE_LENGTH.contains(&t.len()) || !s.is_ascii_alphanumeric() {
            return Err(ParserError::InvalidExtension);
        }

        let s = s.to_ascii_lowercase();

        if s == TRUE_TVALUE {
            Ok(None)
        } else {
            Ok(Some(s))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromStr for Value {
    type Err = ParserError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(source.as_bytes())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for subtag in self.0.iter() {
            write!(f, "-{}", subtag)?;
        }
        Ok(())
    }
}
