pub mod audit;
pub mod integrity;
pub mod jcs;
pub mod registry;
pub mod revision;

pub use audit::{Actor, AuditEvent, EventType, ArchiveAudit};
pub use integrity::{IntegrityFailure, IntegrityReport, ArchiveIntegrity, canonical_hash};
pub use registry::{ArchiveFilter, ArchiveRecord, ArchiveRecordStatus, ArchiveRecordSummary, ArchiveRegistry};
pub use revision::ArchiveRevision;

use base64::Engine as _;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::NormordisPdfError;

/// Render archive format version produced by this engine.
pub const ARCHIVE_VERSION: &str = "1.1.0";

/// A fully resolved NORMORDIS render archive.
///
/// Not to be confused with the NDF (NORMORDIS Document Format) specification:
/// this is the engine's self-contained render bundle — template provenance,
/// resolved data, styles, integrity, audit chain, outputs and signatures.
///
/// Immutable fields after creation: `origin`, `revision`, `meta`, `output`,
/// `styles`, `content`, `integrity`.
/// Append-only fields: `audit.events`, `outputs`, `signatures`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderArchive {
    /// Render archive format version. Always "1.1.0" for archives created by this engine.
    /// Serialised as `archive`; `archive` is accepted on read for archives written by
    /// releases before the rename (see CHANGELOG, R14).
    #[serde(rename = "archive", alias = "archive")]
    pub archive: String,
    /// Generation traceability — engine, template, actor. Immutable.
    pub origin: ArchiveOrigin,
    /// Revision reference. None for original documents. Immutable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<ArchiveRevisionRef>,
    /// Document metadata with resolved values. Immutable.
    pub meta: ArchiveMeta,
    /// PDF output options from the NDT template. Immutable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    /// Fully resolved styles as canonical JSON. Immutable.
    pub styles: Value,
    /// Resolved document body (all placeholders substituted) as canonical JSON. Immutable.
    pub content: Value,
    /// Integrity hashes over canonical JSON. Immutable.
    pub integrity: ArchiveIntegrity,
    /// Append-only audit chain.
    pub audit: ArchiveAudit,
    /// Append-only list of rendered outputs.
    #[serde(default)]
    pub outputs: Vec<ArchiveOutput>,
    /// Append-only list of digital signatures.
    #[serde(default)]
    pub signatures: Vec<ArchiveSignature>,
    /// NDT page configuration (header/footer). Stored for historical regeneration. Immutable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<Value>,
    /// Custom font families embedded as base64 for self-contained historical regeneration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embedded_fonts: Vec<ArchiveEmbeddedFont>,
}

impl RenderArchive {
    /// Serialises to canonical JSON per RFC 8785 / JCS.
    pub fn to_canonical_json(&self) -> crate::Result<String> {
        let value =
            serde_json::to_value(self).map_err(|e| NormordisPdfError::SerdeError(e.to_string()))?;
        let canonical = jcs::canonicalise(&value);
        serde_json::to_string(&canonical).map_err(|e| NormordisPdfError::SerdeError(e.to_string()))
    }

    /// Serialises to pretty-printed JSON. Use only for debugging; not for hashing.
    pub fn to_pretty_json(&self) -> crate::Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| NormordisPdfError::SerdeError(e.to_string()))
    }

    /// Appends an audit event, verifying content_hash for documentary events.
    pub fn add_event(&mut self, event: AuditEvent) -> crate::Result<()> {
        if let Some(ref hash) = event.content_hash && hash != &self.integrity.content_hash {
            return Err(NormordisPdfError::ArchiveAuditError(format!(
                "content_hash mismatch at event seq {} — content has been modified",
                self.audit.next_seq()
            )));
        }
        self.audit.append(event)
    }

    /// Appends an output record.
    pub fn add_output(&mut self, output: ArchiveOutput) -> crate::Result<()> {
        self.outputs.push(output);
        Ok(())
    }

    /// Appends a signature record.
    pub fn add_signature(&mut self, sig: ArchiveSignature) -> crate::Result<()> {
        self.signatures.push(sig);
        Ok(())
    }

    /// Verifies all integrity hashes and the audit chain.
    pub fn verify_integrity(&self) -> crate::Result<IntegrityReport> {
        integrity::verify(self)
    }

    pub fn is_signed(&self) -> bool {
        !self.signatures.is_empty()
    }

    pub fn is_approved(&self) -> bool {
        self.audit
            .events
            .iter()
            .any(|e| e.event_type == EventType::DocumentApproved)
    }

    pub fn is_superseded(&self) -> bool {
        self.audit
            .events
            .iter()
            .any(|e| e.event_type == EventType::DocumentSuperseded)
    }

    pub fn is_revision(&self) -> bool {
        self.revision.is_some()
    }

    /// Embed a custom font family into this archive for self-contained historical regeneration.
    ///
    /// Call this after [`compile_ndt`] for each non-built-in font used in the template.
    /// Built-in fonts (Liberation Sans/Serif/Mono, Libertinus Serif) do not need embedding.
    pub fn embed_font(
        &mut self,
        family: &str,
        regular: &[u8],
        bold: Option<&[u8]>,
        italic: Option<&[u8]>,
        bold_italic: Option<&[u8]>,
    ) {
        self.embedded_fonts.push(ArchiveEmbeddedFont::from_bytes(
            family,
            regular,
            bold,
            italic,
            bold_italic,
        ));
    }
}

// ── ArchiveEmbeddedFont ───────────────────────────────────────────────────────────

/// A custom font family embedded in a render archive as base64-encoded TTF/OTF bytes.
///
/// Store in [`RenderArchive::embedded_fonts`] so the document is self-contained
/// for historical regeneration without external font files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEmbeddedFont {
    /// Font family name as used in the template (e.g. `"Roboto"`, `"FiraSans"`).
    pub family: String,
    /// Regular variant — base64-encoded TTF/OTF. Required.
    pub regular: String,
    /// Bold variant — base64-encoded TTF/OTF.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<String>,
    /// Italic variant — base64-encoded TTF/OTF.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<String>,
    /// Bold-italic variant — base64-encoded TTF/OTF.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold_italic: Option<String>,
}

impl ArchiveEmbeddedFont {
    /// Encode raw font bytes into an [`ArchiveEmbeddedFont`] record.
    pub fn from_bytes(
        family: &str,
        regular: &[u8],
        bold: Option<&[u8]>,
        italic: Option<&[u8]>,
        bold_italic: Option<&[u8]>,
    ) -> Self {
        let enc = base64::engine::general_purpose::STANDARD;
        Self {
            family: family.to_string(),
            regular: enc.encode(regular),
            bold: bold.map(|b| enc.encode(b)),
            italic: italic.map(|b| enc.encode(b)),
            bold_italic: bold_italic.map(|b| enc.encode(b)),
        }
    }
}

// ── Supporting types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveOrigin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndt_template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndt_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndt_template_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndt_data_hash: Option<String>,
    pub engine_version: String,
    pub engine_backend: String,
    pub generated_at: String,
    pub generated_by: Actor,
}

fn default_lang() -> String {
    "pt-PT".into()
}

fn default_classification() -> String {
    "public".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMeta {
    pub title: String,
    #[serde(default)]
    pub entity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    #[serde(default = "default_classification")]
    pub classification: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(default)]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat_mode: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numbering: Option<ArchiveMetaNumbering>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetaNumbering {
    pub numbering_ref: String,
    pub document_number: String,
    pub sequence_id: String,
    pub assigned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRevisionRef {
    pub revision_of: String,
    pub revision_reason: String,
    pub revision_seq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveOutput {
    pub format: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub generated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSignature {
    pub algorithm: String,
    pub signer: String,
    pub signed_at: String,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
