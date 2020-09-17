use thiserror::Error;

#[derive(Debug, Error)]
pub enum BytesTrieBuilderError {
    #[error("Duplicate string")]
    DuplicateString,

    #[error("Illegal state: Cannot add entries after `build()`")]
    AddAfterBuild,

    #[error("The maximum string length is 0xffff.")]
    KeyTooLong,
}
