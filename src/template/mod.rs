pub mod data;
pub mod model;
pub mod registry;
pub mod renderer;
pub mod resolver;
pub mod validator;

pub use data::NdtData;
pub use model::NdtDocument;
pub use registry::TemplateFilter;
pub use renderer::render_template;

pub use archive_pipeline::{
    CompileOptions, compile_ndt, parse_archive, render_archive,
    render_archive_prepared_for_signing, render_archive_prepared_for_signing_with_fonts,
    render_archive_with_fonts, verify_archive,
};
mod archive_pipeline;

pub const ENGINE_NDT_VERSION: &str = "2.0.0";
pub const ENGINE_NDT_DATA_VERSION: &str = "1.0.0";

pub use resolver::{RuntimeContext, resolve_runtime_fields};

use crate::elements::Element;
use crate::styles::DocumentStyle;

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("NDT version {template} incompatible with engine {engine}")]
    IncompatibleVersion { template: String, engine: String },
    #[error("Include not found: {path}")]
    IncludeNotFound { path: String },
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("TOML error: {0}")]
    TomlError(String),
    #[error("Render error: {0}")]
    RenderError(String),
}

/// Parse an NDT template string.  Auto-detects format: strings starting with
/// `{` are parsed as JSON; everything else is treated as TOML.
pub fn parse_ndt(input: &str) -> Result<NdtDocument, TemplateError> {
    if input.trim_start().starts_with('{') {
        serde_json::from_str(input).map_err(TemplateError::JsonError)
    } else {
        toml::from_str(input).map_err(|e| TemplateError::TomlError(e.to_string()))
    }
}

/// Parse an NdtData JSON string.
pub fn parse_ndt_data(json: &str) -> Result<NdtData, TemplateError> {
    serde_json::from_str(json).map_err(TemplateError::JsonError)
}

/// Serialize an [`NdtDocument`] to a pretty-printed JSON string.
pub fn serialize_ndt_json(doc: &NdtDocument) -> Result<String, TemplateError> {
    serde_json::to_string_pretty(doc).map_err(TemplateError::JsonError)
}

/// Serialize an [`NdtDocument`] to a TOML string.
///
/// # Limitations
///
/// Fields containing `serde_json::Value::Null` are silently dropped, as TOML
/// has no null type.  Use [`serialize_ndt_json`] for lossless round-trips.
pub fn serialize_ndt_toml(doc: &NdtDocument) -> Result<String, TemplateError> {
    toml::to_string_pretty(doc).map_err(|e| TemplateError::TomlError(e.to_string()))
}

/// Validate that the template version is compatible with this engine.
pub fn check_version_compatibility(template_version: &str) -> Result<(), TemplateError> {
    let engine_major: u32 = ENGINE_NDT_VERSION
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    let template_major: u32 = template_version
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if template_major > engine_major {
        Err(TemplateError::IncompatibleVersion {
            template: template_version.to_string(),
            engine: ENGINE_NDT_VERSION.to_string(),
        })
    } else {
        Ok(())
    }
}

/// Full pipeline: parse NDT + render to elements.
pub fn render(
    template_json: &str,
    data_json: &str,
    style: &DocumentStyle,
) -> Result<Vec<Box<dyn Element>>, TemplateError> {
    let doc = parse_ndt(template_json)?;
    let data = parse_ndt_data(data_json)?;
    check_version_compatibility(&doc.ndt_version)?;
    render_template(&doc, &data, style)
}
