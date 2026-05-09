use thiserror::Error;

#[derive(Debug, Error)]
pub enum Dotx2NdtError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("ZIP error: {0}")]
    Zip(String),

    #[error("missing ZIP entry: {0}")]
    MissingEntry(String),

    #[error("XML parse error: {0}")]
    Xml(String),

}
