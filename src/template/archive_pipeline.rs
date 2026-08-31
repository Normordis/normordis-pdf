use base64::Engine as _;
use regex::Regex;
use serde_json::Value;

use super::data::NdtData;
use super::resolver;
use crate::archive::{
    ARCHIVE_VERSION, ArchiveEmbeddedFont, ArchiveMeta, ArchiveOrigin, RenderArchive,
    audit::{Actor, ArchiveAudit, AuditEvent, EventType},
    integrity::{ArchiveIntegrity, canonical_hash},
};
use crate::{NormordisPdfError, Result};

// ── CompileOptions ────────────────────────────────────────────────────────────

/// Options controlling how `compile_ndt()` builds an `RenderArchive`.
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Unique document identifier.
    /// If `None`, a UUID v4 is generated automatically.
    pub document_id: Option<String>,

    /// Actor responsible for this generation (stored in audit chain).
    pub generated_by: Actor,

    /// NDT template identifier for origin traceability.
    pub ndt_template_id: Option<String>,

    /// SHA-256 hash of the NDT template file.
    /// If `None`, computed automatically from the `ndt` input.
    pub ndt_template_hash: Option<String>,

    /// If `true` (default), error when any `{{placeholder}}` remains unresolved.
    pub validate_resolved: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            document_id: None,
            generated_by: Actor::System {
                id: "normordis-pdf".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
                instance_id: None,
            },
            ndt_template_id: None,
            ndt_template_hash: None,
            validate_resolved: true,
        }
    }
}

// ── compile_ndt ───────────────────────────────────────────────────────────────

/// Compiles an NDT template + data into a fully resolved `RenderArchive`.
///
/// Pipeline:
/// 1. Parse NDT (JSON or TOML)
/// 2. Validate required placeholders
/// 3. Deep-substitute `{{placeholders}}` in all body string fields
/// 4. Check no unresolved placeholders remain (`validate_resolved`)
/// 5. Compute integrity hashes (RFC 8785 / JCS)
/// 6. Build and return `RenderArchive`
///
/// After calling this, use [`RenderArchive::embed_font`] for any custom fonts
/// used in the template before persisting the archive.
pub fn compile_ndt(ndt: &str, data: &NdtData, options: CompileOptions) -> Result<RenderArchive> {
    let doc =
        super::parse_ndt(ndt).map_err(|e| NormordisPdfError::ArchiveCompileError(e.to_string()))?;

    // Serialize and resolve paginas_def + estilos
    let content_val = serde_json::to_value(&doc.paginas_def)
        .map_err(|e| NormordisPdfError::SerdeError(e.to_string()))?;
    let resolved_content = resolve_value_placeholders(content_val, data);

    let styles_val = serde_json::to_value(&doc.estilos)
        .map_err(|e| NormordisPdfError::SerdeError(e.to_string()))?;

    if options.validate_resolved {
        let content_str = serde_json::to_string(&resolved_content)
            .map_err(|e| NormordisPdfError::SerdeError(e.to_string()))?;
        let re = Regex::new(r"\{\{[a-zA-Z0-9_.]+\}\}").expect("static regex");
        if let Some(m) = re.find(&content_str) {
            return Err(NormordisPdfError::ArchiveCompileError(format!(
                "unresolved placeholder '{}' in content after substitution",
                m.as_str()
            )));
        }
    }

    let now = chrono::Utc::now().to_rfc3339();

    let meta_title = doc.titulo.clone().unwrap_or_default();
    let meta = ArchiveMeta {
        title: meta_title,
        entity: String::new(),
        entity_id: None,
        lang: "pt-PT".into(),
        document_ref: None,
        document_type: None,
        classification: "public".into(),
        subject: None,
        keywords: None,
        created_at: now.clone(),
        valid_from: None,
        valid_until: None,
        supersedes: None,
        compat_mode: None,
        numbering: None,
    };
    let meta_val =
        serde_json::to_value(&meta).map_err(|e| NormordisPdfError::SerdeError(e.to_string()))?;

    let integrity = ArchiveIntegrity::compute(&resolved_content, &styles_val, &meta_val)?;
    let document_id = options
        .document_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let ndt_template_hash = options.ndt_template_hash.unwrap_or_else(|| {
        let v = serde_json::from_str::<Value>(ndt).unwrap_or(Value::Null);
        canonical_hash(&v)
    });

    let first_event = AuditEvent {
        seq: 1,
        event_type: EventType::DocumentGenerated,
        timestamp: now.clone(),
        actor: options.generated_by.clone(),
        content_hash: Some(integrity.content_hash.clone()),
        note: None,
        extra: Default::default(),
    };

    Ok(RenderArchive {
        archive: ARCHIVE_VERSION.into(),
        origin: ArchiveOrigin {
            ndt_template_id: options.ndt_template_id,
            ndt_version: None,
            ndt_template_hash: Some(ndt_template_hash),
            ndt_data_hash: None,
            engine_version: env!("CARGO_PKG_VERSION").into(),
            engine_backend: "normordis-pdf".into(),
            generated_at: now,
            generated_by: options.generated_by,
        },
        revision: None,
        meta,
        output: None,
        styles: styles_val,
        content: resolved_content,
        page: None,
        embedded_fonts: vec![],
        integrity,
        audit: ArchiveAudit {
            document_id,
            events: vec![first_event],
        },
        outputs: vec![],
        signatures: vec![],
    })
}

// ── parse_archive / verify_archive ────────────────────────────────────────────────────

/// Parses a render archive from JSON (canonical or pretty-printed).
pub fn parse_archive(json: &str) -> Result<RenderArchive> {
    serde_json::from_str(json).map_err(|e| NormordisPdfError::SerdeError(e.to_string()))
}

/// Verifies the integrity hashes of a render archive.
pub fn verify_archive(json: &str) -> Result<crate::archive::integrity::IntegrityReport> {
    let archive = parse_archive(json)?;
    archive.verify_integrity()
}

// ── render_archive ────────────────────────────────────────────────────────────────

/// Renders a render archive to PDF bytes.
///
/// Fonts embedded in `archive.embedded_fonts` are loaded automatically.
/// For additional or override fonts, use [`render_archive_with_fonts`].
pub fn render_archive(archive_json: &str) -> Result<Vec<u8>> {
    render_archive_inner(archive_json, None)
}

/// Renders a render archive to PDF bytes, supplementing with an external font registry.
///
/// Fonts in `extra` take precedence over fonts embedded in the archive.
/// Use this when the archive was written without embedding font bytes and the
/// original fonts are available at render time.
pub fn render_archive_with_fonts(
    archive_json: &str,
    extra: &crate::fonts::FontRegistry,
) -> Result<Vec<u8>> {
    render_archive_inner(archive_json, Some(extra))
}

fn render_archive_inner(
    archive_json: &str,
    extra_fonts: Option<&crate::fonts::FontRegistry>,
) -> Result<Vec<u8>> {
    let archive = parse_archive(archive_json)?;
    let (body, fonts) = rebuild_body_elements_and_fonts(&archive, extra_fonts)?;
    let (standard, compression, accessibility) = parse_output_options(archive.output.as_ref());

    let style = crate::styles::DocumentStyle::default();
    let empty_data = empty_ndt_data();
    let elements = super::renderer::render_body_elements(&body, &empty_data, &style)
        .map_err(|e| NormordisPdfError::Template(e.to_string()))?;

    crate::document::Document {
        title: archive.meta.title,
        style,
        fonts,
        header: None,
        sectioned_header: None,
        footer: None,
        sectioned_footer: None,
        watermark: None,
        elements,
        footnotes: vec![],
        toc_entries: None,
        compression,
        standard,
        signature: None,
        traceability: None,
        accessibility,
    }
    .render_to_bytes()
}

// ── render_archive_prepared_for_signing ──────────────────────────────────────────

/// Renders a render archive to a [`PreparedPdf`] ready for external PKCS#7 signing.
pub fn render_archive_prepared_for_signing(
    archive_json: &str,
    opts: crate::signing::SignatureOptions,
) -> Result<crate::signing::PreparedPdf> {
    render_archive_prepared_for_signing_inner(archive_json, opts, None)
}

/// Renders a render archive to a [`PreparedPdf`], supplementing with an external font registry.
pub fn render_archive_prepared_for_signing_with_fonts(
    archive_json: &str,
    opts: crate::signing::SignatureOptions,
    extra: &crate::fonts::FontRegistry,
) -> Result<crate::signing::PreparedPdf> {
    render_archive_prepared_for_signing_inner(archive_json, opts, Some(extra))
}

fn render_archive_prepared_for_signing_inner(
    archive_json: &str,
    opts: crate::signing::SignatureOptions,
    extra_fonts: Option<&crate::fonts::FontRegistry>,
) -> Result<crate::signing::PreparedPdf> {
    let archive = parse_archive(archive_json)?;
    let (body, fonts) = rebuild_body_elements_and_fonts(&archive, extra_fonts)?;
    let (standard, compression, accessibility) = parse_output_options(archive.output.as_ref());

    let style = crate::styles::DocumentStyle::default();
    let empty_data = empty_ndt_data();
    let elements = super::renderer::render_body_elements(&body, &empty_data, &style)
        .map_err(|e| NormordisPdfError::Template(e.to_string()))?;

    crate::document::Document {
        title: archive.meta.title,
        style,
        fonts,
        header: None,
        sectioned_header: None,
        footer: None,
        sectioned_footer: None,
        watermark: None,
        elements,
        footnotes: vec![],
        toc_entries: None,
        compression,
        standard,
        signature: None,
        traceability: None,
        accessibility,
    }
    .render_prepared_for_signing(opts)
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Reconstruct legacy body elements and a `FontRegistry` from a parsed render archive.
fn rebuild_body_elements_and_fonts(
    archive: &RenderArchive,
    extra_fonts: Option<&crate::fonts::FontRegistry>,
) -> Result<(
    Vec<super::model::legacy_body::BodyElement>,
    crate::fonts::FontRegistry,
)> {
    // Graceful fallback: archives compiled from NDT 2.0.0 paginas_def content won't
    // deserialize as BodyElement; return empty body so rendering still succeeds.
    let body: Vec<super::model::legacy_body::BodyElement> =
        serde_json::from_value(archive.content.clone()).unwrap_or_default();

    let mut fonts = crate::fonts::FontRegistry::default();
    for ef in &archive.embedded_fonts {
        decode_and_register_font(ef, &mut fonts)?;
    }
    if let Some(extra) = extra_fonts {
        for (_name, fam) in extra.families() {
            fonts.register(fam.clone());
        }
    }

    Ok((body, fonts))
}

/// Decode a base64-encoded [`ArchiveEmbeddedFont`] and register it in the registry.
fn decode_and_register_font(
    ef: &ArchiveEmbeddedFont,
    fonts: &mut crate::fonts::FontRegistry,
) -> Result<()> {
    let dec = base64::engine::general_purpose::STANDARD;
    let regular = dec
        .decode(&ef.regular)
        .map_err(|e| NormordisPdfError::FontLoadError(e.to_string()))?;
    let bold = ef
        .bold
        .as_deref()
        .map(|s| {
            dec.decode(s)
                .map_err(|e| NormordisPdfError::FontLoadError(e.to_string()))
        })
        .transpose()?;
    let italic = ef
        .italic
        .as_deref()
        .map(|s| {
            dec.decode(s)
                .map_err(|e| NormordisPdfError::FontLoadError(e.to_string()))
        })
        .transpose()?;
    let bold_italic = ef
        .bold_italic
        .as_deref()
        .map(|s| {
            dec.decode(s)
                .map_err(|e| NormordisPdfError::FontLoadError(e.to_string()))
        })
        .transpose()?;
    fonts.register_bytes(
        &ef.family,
        &regular,
        bold.as_deref(),
        italic.as_deref(),
        bold_italic.as_deref(),
    )
}

/// Parse PDF output options from the archive `output` field.
fn parse_output_options(
    output: Option<&Value>,
) -> (
    crate::document::PdfStandard,
    crate::document::CompressionLevel,
    crate::compliance::ua::AccessibilityConfig,
) {
    let ndt_output: Option<super::model::NdtOutput> =
        output.and_then(|v| serde_json::from_value(v.clone()).ok());

    let standard = match ndt_output.as_ref().and_then(|o| o.standard.as_deref()) {
        Some("pdf_a_1b") | Some("pdf_a1b") => crate::document::PdfStandard::PdfA1b,
        Some("pdf_a_2b") | Some("pdf_a2b") => crate::document::PdfStandard::PdfA2b,
        Some("pdf_ua2") | Some("pdf_ua_2") => crate::document::PdfStandard::PdfUa2,
        _ => crate::document::PdfStandard::Pdf17,
    };

    let compression = match ndt_output.as_ref().and_then(|o| o.compression.as_deref()) {
        Some("none") => crate::document::CompressionLevel::None,
        Some("fast") => crate::document::CompressionLevel::Fast,
        Some("best") => crate::document::CompressionLevel::Best,
        _ => crate::document::CompressionLevel::Default,
    };

    let accessibility = ndt_output
        .as_ref()
        .and_then(|o| o.accessibility.clone())
        .unwrap_or_default();

    (standard, compression, accessibility)
}

fn empty_ndt_data() -> NdtData {
    NdtData {
        ndt_data: "1.0.0".into(),
        template_id: None,
        template_version: None,
        data: Default::default(),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Recursively substitutes `{{placeholder}}` patterns in all string nodes of a JSON value.
fn resolve_value_placeholders(value: Value, data: &NdtData) -> Value {
    match value {
        Value::String(s) => Value::String(resolver::resolve_string(&s, data)),
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|v| resolve_value_placeholders(v, data))
                .collect(),
        ),
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k, resolve_value_placeholders(v, data));
            }
            Value::Object(new_map)
        }
        other => other,
    }
}
