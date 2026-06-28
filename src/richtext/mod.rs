pub mod converter;
pub mod marks;
pub mod model;

pub use model::NcrtfDocument;

/// NCRTF format version supported by this release.
pub const NCRTF_VERSION: &str = "2.0.0";

use crate::{NormordisPdfError, Result, elements::Element, styles::DocumentStyle};

/// Parse a JSON string as an NCRTF v2.0.0 document.
///
/// # Example
///
/// ```rust
/// use normordis_pdf::parse_ncrtf;
///
/// let json = r#"{"ncrtf_version":"2.0.0","content":[]}"#;
/// let doc = parse_ncrtf(json).unwrap();
/// assert_eq!(doc.ncrtf_version, "2.0.0");
/// ```
pub fn parse_ncrtf(json: &str) -> Result<NcrtfDocument> {
    serde_json::from_str(json).map_err(|e| NormordisPdfError::ParseError(e.to_string()))
}

/// Convert a parsed `NcrtfDocument` into renderable `normordis-pdf` elements.
pub fn ncrtf_to_elements(doc: &NcrtfDocument, style: &DocumentStyle) -> Vec<Box<dyn Element>> {
    converter::ncrtf_to_elements(doc, style)
}
