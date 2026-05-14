use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::NdfDocument;
use crate::NormaxisPdfError;

/// Lifecycle status of a persisted NDF document record.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NdfRecordStatus {
    #[default]
    Active,
    Superseded,
    Archived,
}

impl NdfRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Superseded => "superseded",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for NdfRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A persisted NDF document record ready for storage in any database.
///
/// Index fields map to individual DB columns for efficient querying.
/// The `payload` field holds the full canonical NDF JSON for retrieval and regeneration.
///
/// # Suggested SQL schema
///
/// ```sql
/// CREATE TABLE ndf_documents (
///     document_id    TEXT PRIMARY KEY,
///     title          TEXT NOT NULL,
///     entity         TEXT NOT NULL DEFAULT '',
///     classification TEXT NOT NULL DEFAULT 'public',
///     created_at     TEXT NOT NULL,
///     template_id    TEXT,
///     document_ref   TEXT,
///     document_type  TEXT,
///     status         TEXT NOT NULL DEFAULT 'active',
///     integrity_hash TEXT NOT NULL,
///     payload        TEXT NOT NULL  -- TEXT (SQLite) or JSONB (PostgreSQL)
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdfRecord {
    // ── Index fields (DB columns) ──────────────────────────────────────────────
    pub document_id: String,
    pub title: String,
    pub entity: String,
    pub classification: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    pub status: NdfRecordStatus,
    /// Canonical hash from the document's own `integrity.ndf_hash` field.
    pub integrity_hash: String,
    // ── Full payload ───────────────────────────────────────────────────────────
    /// Canonical NDF JSON (RFC 8785 / JCS). Store as TEXT / JSONB / longtext.
    pub payload: String,
}

impl NdfRecord {
    /// Build an [`NdfRecord`] from a live [`NdfDocument`].
    ///
    /// Serialises to canonical JSON and extracts index fields.
    pub fn from_ndf(ndf: &NdfDocument) -> crate::Result<Self> {
        let status = if ndf.is_superseded() {
            NdfRecordStatus::Superseded
        } else {
            NdfRecordStatus::Active
        };
        Ok(Self {
            document_id: ndf.audit.document_id.clone(),
            title: ndf.meta.title.clone(),
            entity: ndf.meta.entity.clone(),
            classification: ndf.meta.classification.clone(),
            created_at: ndf.meta.created_at.clone(),
            template_id: ndf.origin.ndt_template_id.clone(),
            document_ref: ndf.meta.document_ref.clone(),
            document_type: ndf.meta.document_type.clone(),
            status,
            integrity_hash: ndf.integrity.ndf_hash.clone(),
            payload: ndf.to_canonical_json()?,
        })
    }

    /// Recover the [`NdfDocument`] from this record's payload.
    pub fn to_ndf(&self) -> crate::Result<NdfDocument> {
        serde_json::from_str(&self.payload)
            .map_err(|e| NormaxisPdfError::SerdeError(e.to_string()))
    }

    /// SHA-256 of the raw payload bytes (independent of NDF integrity hashes).
    pub fn payload_sha256(&self) -> String {
        let hash = Sha256::digest(self.payload.as_bytes());
        format!("sha256:{}", hex::encode(hash))
    }

    /// Build a lightweight summary from this record.
    pub fn summary(&self) -> NdfRecordSummary {
        NdfRecordSummary {
            document_id: self.document_id.clone(),
            title: self.title.clone(),
            entity: self.entity.clone(),
            classification: self.classification.clone(),
            created_at: self.created_at.clone(),
            template_id: self.template_id.clone(),
            document_ref: self.document_ref.clone(),
            document_type: self.document_type.clone(),
            status: self.status.clone(),
            integrity_hash: self.integrity_hash.clone(),
        }
    }
}

/// Lightweight summary of an NDF record for listing operations (no payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdfRecordSummary {
    pub document_id: String,
    pub title: String,
    pub entity: String,
    pub classification: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    pub status: NdfRecordStatus,
    pub integrity_hash: String,
}

/// Filter criteria for listing NDF records.
///
/// All fields are optional; unset fields are not applied.
#[derive(Debug, Clone, Default)]
pub struct NdfFilter {
    /// Return only records for this entity.
    pub entity: Option<String>,
    /// Return only records with this classification.
    pub classification: Option<String>,
    /// Return only records generated from this template UUID.
    pub template_id: Option<String>,
    /// Return only records of this document type.
    pub document_type: Option<String>,
    /// Return only records with this lifecycle status.
    pub status: Option<NdfRecordStatus>,
    /// Return only records created on or after this ISO 8601 timestamp.
    pub created_after: Option<String>,
    /// Return only records created before this ISO 8601 timestamp.
    pub created_before: Option<String>,
}

/// DB-agnostic interface for an institutional NDF document archive.
///
/// Implementors provide the actual persistence layer — SQLite, PostgreSQL,
/// MongoDB, or an in-memory store. The serialisation format is always canonical
/// JSON (RFC 8785 / JCS) via [`NdfRecord::payload`], so records can migrate
/// between backends without conversion.
///
/// # Regeneration guarantee
///
/// Any document stored via `save` can be re-rendered to an identical PDF by:
/// ```rust,ignore
/// let record = registry.record(document_id)?;
/// let ndf = record.to_ndf()?;
/// let pdf = normordis_pdf::render_ndf(&ndf.to_canonical_json()?)?;
/// ```
///
/// For documents with custom fonts, use [`render_ndf_with_fonts`] or ensure
/// font bytes were embedded before saving via [`NdfDocument::embed_font`].
///
/// # Example (in-memory implementation sketch)
///
/// ```rust,ignore
/// struct MemoryNdfRegistry { records: Vec<NdfRecord> }
///
/// impl NdfRegistry for MemoryNdfRegistry {
///     type Error = String;
///
///     fn save(&mut self, ndf: &NdfDocument) -> Result<NdfRecord, String> {
///         let record = NdfRecord::from_ndf(ndf).map_err(|e| e.to_string())?;
///         self.records.push(record.clone());
///         Ok(record)
///     }
///     // …
/// }
/// ```
pub trait NdfRegistry {
    /// The error type returned by all operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Persist an NDF document and return the resulting indexed record.
    ///
    /// Implementations should reject duplicate `document_id` values with an error.
    fn save(&mut self, ndf: &NdfDocument) -> Result<NdfRecord, Self::Error>;

    /// Load a document by its stable `document_id`.
    fn load(&self, document_id: &str) -> Result<NdfDocument, Self::Error>;

    /// Load the indexed record (including payload) by `document_id`.
    fn record(&self, document_id: &str) -> Result<NdfRecord, Self::Error>;

    /// Return lightweight summaries matching `filter`.
    ///
    /// Results should be ordered by `created_at` descending.
    fn list(&self, filter: NdfFilter) -> Result<Vec<NdfRecordSummary>, Self::Error>;

    /// Check whether a document exists without loading the full payload.
    fn exists(&self, document_id: &str) -> Result<bool, Self::Error>;

    /// Mark a document as `archived` (soft delete — payload is preserved).
    fn archive(&mut self, document_id: &str) -> Result<(), Self::Error>;
}
