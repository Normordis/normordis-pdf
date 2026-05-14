use super::model::{NdtTemplateSummary, NdtTemplateRecord, TemplateStatus};

/// Filter criteria for listing templates in a registry.
///
/// All fields are optional; unset fields are not applied.  The default
/// (all `None` / empty `tags`) returns every record in the registry.
#[derive(Debug, Clone, Default)]
pub struct TemplateFilter {
    /// Return only templates in this namespace.
    pub namespace: Option<String>,
    /// Return only templates with this lifecycle status.
    pub status: Option<TemplateStatus>,
    /// Return only templates in this category.
    pub category: Option<String>,
    /// Return only templates that carry ALL of these tags.
    pub tags: Vec<String>,
    /// Return only templates with this locale (BCP-47).
    pub locale: Option<String>,
}

/// DB-agnostic interface for an institutional NDT template registry.
///
/// Implementors provide the actual persistence layer — SQLite, PostgreSQL,
/// MongoDB, or an in-memory store.  The serialisation format is always JSON
/// (via [`NdtTemplateRecord::to_json`]) so records can migrate between
/// backends without conversion.
///
/// # Versioning model
///
/// Each template has a stable `id` (UUID) and a monotonically increasing
/// `version_number`.  The `(id, version_number)` pair is the unique key.
/// Creating a new version via [`NdtTemplateRecord::bump_version`] populates
/// `parent_id` and `parent_version`, preserving the full lineage.
///
/// # Example (in-memory implementation sketch)
///
/// ```rust,ignore
/// struct MemoryRegistry { records: Vec<NdtTemplateRecord> }
///
/// impl NdtRegistry for MemoryRegistry {
///     type Error = String;
///
///     fn save(&mut self, record: &NdtTemplateRecord) -> Result<(), String> {
///         self.records.push(record.clone());
///         Ok(())
///     }
///     // … other methods
/// }
/// ```
pub trait NdtRegistry {
    /// The error type returned by all operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Persist a template record.
    ///
    /// Implementations **should** reject duplicate `(id, version_number)`
    /// pairs with an error rather than silently overwriting.
    fn save(&mut self, record: &NdtTemplateRecord) -> Result<(), Self::Error>;

    /// Load a specific version of a template by its stable `id`.
    ///
    /// When `version` is `None`, returns the latest active version, falling
    /// back to the latest record of any status if no active version exists.
    fn load(&self, id: &str, version: Option<u32>) -> Result<NdtTemplateRecord, Self::Error>;

    /// Load by human-readable `slug` within a `namespace`.
    ///
    /// Same `version = None` semantics as [`load`].
    fn load_by_slug(
        &self,
        namespace: &str,
        slug: &str,
        version: Option<u32>,
    ) -> Result<NdtTemplateRecord, Self::Error>;

    /// Return lightweight summaries matching `filter`.
    ///
    /// Results should be ordered by `(namespace, slug, version_number)`.
    fn list(&self, filter: TemplateFilter) -> Result<Vec<NdtTemplateSummary>, Self::Error>;

    /// Check whether a record exists without loading the full document.
    fn exists(&self, id: &str, version: Option<u32>) -> Result<bool, Self::Error>;

    /// Mark a specific version as `deprecated`.
    fn deprecate(&mut self, id: &str, version: u32) -> Result<(), Self::Error>;

    /// Mark a specific version as `archived`.
    fn archive(&mut self, id: &str, version: u32) -> Result<(), Self::Error>;
}
