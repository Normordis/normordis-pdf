use thiserror::Error;

/// All errors that can occur during PDF generation.
#[derive(Debug, Error)]
pub enum NormordisPdfError {
    #[error("font load error: {0}")]
    FontLoadError(String),

    #[error("image load error: {0}")]
    ImageLoadError(String),

    #[error("render error: {0}")]
    RenderError(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("template error: {0}")]
    Template(String),

    #[error("cycle detected in style inheritance chain: '{0}'")]
    StyleCycleError(String),

    #[error("unknown style name: '{0}'")]
    UnknownStyle(String),

    #[error("render archive integrity error: {0}")]
    ArchiveIntegrityError(String),

    #[error("render archive audit chain error: {0}")]
    ArchiveAuditError(String),

    #[error("render archive revision error: {0}")]
    ArchiveRevisionError(String),

    #[error("render archive compile error: {0}")]
    ArchiveCompileError(String),

    #[error("PDF/UA-2 accessibility error: {0}")]
    AccessibilityError(String),

    #[error("serialisation error: {0}")]
    SerdeError(String),

    #[error("TSA timestamp error: {0}")]
    TsaError(String),
}

pub type Result<T> = std::result::Result<T, NormordisPdfError>;

/// Deprecated alias — use [`NormordisPdfError`] instead.
#[deprecated(since = "3.0.0", note = "renamed to NormordisPdfError")]
pub type NormaxisPdfError = NormordisPdfError;
