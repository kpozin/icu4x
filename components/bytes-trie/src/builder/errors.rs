use thiserror::Error;

#[derive(Debug, Error)]
pub enum BytesTrieBuilderError {
    #[error("Duplicate string")]
    DuplicateString,
}
