use thiserror::Error;

#[derive(Debug, Error)]
pub enum BytesTrieBuilderError {
    #[error("Can't build without any entries")]
    EmptyBuilder,

    #[error("Duplicate string")]
    DuplicateString,

    #[error("Illegal state: Cannot add entries after `build()`")]
    AddAfterBuild,

    #[error("The maximum string length is 0xffff.")]
    KeyTooLong,
}
