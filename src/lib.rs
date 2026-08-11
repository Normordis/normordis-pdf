//! # normordis-pdf
//!
//! Institutional PDF generation library for NORMAXIS mini-apps.
//!
//! Generates professional PDF documents for Portuguese public administration,
//! with support for:
//! - Flow and Fixed Box layout modes
//! - NORMORDIS Canonical Rich Text Format (NCRTF)
//! - NORMORDIS Document Template format (NDT v2.0.0)
//! - Named paragraph/table styles with inheritance (equivalent to Word Styles)
//! - Tab stops (left, right, center, decimal) with leader characters
//! - TTF/OTF font loading with real glyph metrics (rustybuzz + ttf-parser)
//! - Left, Justify, Center, Right text alignment
//!
//! ## Quick Start
//!
//! ```rust
//! use normordis_pdf::{DocumentBuilder, Section, Paragraph, TextAlign};
//!
//! let pdf = DocumentBuilder::new("Annual Report")
//!     .push(Section::new("1. Introduction", 1))
//!     .push(Paragraph::new("Document body text.").align(TextAlign::Justify))
//!     .render_to_bytes()?;
//! # Ok::<(), normordis_pdf::NormordisPdfError>(())
//! ```
//!
//! ## Named Styles
//!
//! ```rust
//! use normordis_pdf::{DocumentBuilder, Paragraph, Section};
//!
//! let pdf = DocumentBuilder::new("Styled Document")
//!     .push(Section::new("Introduction", 1))
//!     .push(Paragraph::new("Caption text.").style("caption"))
//!     .render_to_bytes()?;
//! # Ok::<(), normordis_pdf::NormordisPdfError>(())
//! ```
//!
//! ## NDT Templates (v2.0.0)
//!
//! ```rust,no_run
//! use normordis_pdf::DocumentBuilder;
//!
//! let data = r#"{"ndt_data":"1.0.0","data":{"entity":"Câmara Municipal"}}"#;
//! let template = r#"{
//!     "ndt_version": "2.0.0",
//!     "schema_id": "urn:normordis:ndt:example",
//!     "versao_ndt": "1.0.0",
//!     "paginas_def": [{"id": "p1"}],
//!     "sequencia": [{"pagina_def": "p1", "repeticao": "unica"}]
//! }"#;
//!
//! let result = DocumentBuilder::new("Ofício")
//!     .push_ndt(template, data);
//! ```

// ── Modules ───────────────────────────────────────────────────────────────────

pub mod backend;
pub mod builder;
pub mod compliance;
pub mod document;
pub mod elements;
pub mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod fonts;
pub mod layout;
pub mod archive;
pub mod page;
pub mod richtext;
pub mod signing;
pub mod styles;
pub mod template;
pub mod tsa;

// ── Error handling ────────────────────────────────────────────────────────────

pub use error::{NormordisPdfError, Result};
#[allow(deprecated)]
pub use error::NormaxisPdfError;

// ── Digital signing ───────────────────────────────────────────────────────────

pub use signing::{PreparedPdf, SignatureConfig, SignatureField, SignatureOptions, sign_pdf};
pub use tsa::{embed_timestamp, extract_signature_value};
#[cfg(feature = "tsa")]
pub use tsa::{request_timestamp, timestamp_pkcs7};

// ── Styles ────────────────────────────────────────────────────────────────────

pub use styles::{
    DocumentStyle, NamedStyle, Orientation, PageSize, ResolvedStyle, RgbColor,
    SecurityClassification, StyleResolver, TraceabilityMetadata, Watermark, default_named_styles,
};

// ── Fonts ─────────────────────────────────────────────────────────────────────

pub use fonts::{
    FontData,
    FontFallbackChain,
    // v1.3.x backward-compatibility aliases
    FontFamily,
    FontRegistry,
    FontVariant,
    FontVariants,
    ShapedGlyph,
    liberation_mono_family,
    liberation_sans_family,
    liberation_serif_family,
};

// ── Page ─────────────────────────────────────────────────────────────────────

pub use page::PageLayout;

// ── Layout ───────────────────────────────────────────────────────────────────

pub use layout::{
    AppliedStyle, BorderStyle, BoxBorder, DecorationLine, FixedBox, GlyphUsageTracker,
    HighlightColor, KnuthPlassOptimizer, LayoutResult, LineBox, LineBreakingMode, LineSegment,
    OpenTypeFeatures, OverflowPolicy, PageFlow, TabStop, TabStopAlign, TextAlign, TextDecoration,
    TextLayoutEngine, TextRun, WordBox,
};

// ── Builder / Document ───────────────────────────────────────────────────────

pub use backend::pdf_writer_backend::{
    encode_for_identity_h, generate_to_unicode_cmap, subset_font, to_cff_if_possible,
};
pub use backend::{FontRef, ImageRef, PdfBackend};
pub use builder::{DocumentBuilder, SigningBuilder};
pub use document::{CompressionLevel, Document, PdfStandard};

// ── Elements — Flow ──────────────────────────────────────────────────────────

pub use elements::{
    Element, LayoutMode, RenderContext, RenderResult,
    footer::{PageFooter, SectionedFooter},
    footnote::{FOOTNOTE_SEPARATOR_THICKNESS_MM, FootnoteMarkStyle, FootnoteRef},
    form::{
        CheckBoxDef, ComboBoxDef, FieldRect, FormField, ListBoxDef, RadioButtonDef, TextFieldDef,
    },
    header::{InstitutionalHeader, SectionedHeader},
    image::ImageElement,
    list::{BulletList, CheckList, CheckListItem, ListItem, ListItemElement, OrderedList},
    page_break::PageBreakElement,
    paragraph::{Paragraph, ParagraphBorder, ParagraphContent},
    section::Section,
    section_break::{Orientation as SectionOrientation, SectionBreak, SectionMargins},
    spacer::{HorizontalRuleElement, Spacer},
    table::{
        BorderLineStyle, CellBorder, CellBorders, CellPadding, RowHeight, Table, TableBuilder,
        TableCell, TableRow, TableStyle,
    },
    toc::{TableOfContents, TocEntry},
};

// ── Elements — Fixed ─────────────────────────────────────────────────────────

pub use elements::fixed::{FixedImageBox, FixedLineElement, FixedTextBox, ImageFit, VerticalAlign};

// ── Rich text ────────────────────────────────────────────────────────────────

pub use richtext::{NcrtfDocument, ncrtf_to_elements, parse_ncrtf};

// ── Templates ────────────────────────────────────────────────────────────────

pub use template::{
    ENGINE_NDT_DATA_VERSION, ENGINE_NDT_VERSION, NdtDocument, RuntimeContext, TemplateError,
    check_version_compatibility, parse_ndt, parse_ndt_data, render as render_ndt,
    resolve_runtime_fields, serialize_ndt_json, serialize_ndt_toml,
};

// ── NDT 2.0.0 types ───────────────────────────────────────────────────────────

pub use template::model::{NdtOutput, NdtSignature, NdtSignatureField};

// ── NDT template filter ───────────────────────────────────────────────────────

pub use template::TemplateFilter;

// ── Render archive pipeline ─────────────────────────────────────────────────────────────

pub use template::{
    CompileOptions, compile_ndt, parse_archive, render_archive, render_archive_prepared_for_signing,
    render_archive_prepared_for_signing_with_fonts, render_archive_with_fonts, verify_archive,
};

// ── Render archive types ─────────────────────────────────────────────────────────────────

pub use archive::{
    Actor, AuditEvent, EventType, IntegrityFailure, IntegrityReport, ArchiveAudit, RenderArchive,
    ArchiveEmbeddedFont, ArchiveIntegrity, ArchiveMeta, ArchiveMetaNumbering, ArchiveOrigin, ArchiveOutput, ArchiveRevision,
    ArchiveRevisionRef, ArchiveSignature, canonical_hash,
};

// ── Render archive registry ────────────────────────────────────────────────────

pub use archive::{ArchiveFilter, ArchiveRecord, ArchiveRecordStatus, ArchiveRecordSummary, ArchiveRegistry};

// ── NCRTF 2.0.0 types ────────────────────────────────────────────────────────

pub use richtext::marks::MarkValue as NcrtfMark;

// ── Version constants ─────────────────────────────────────────────────────────

/// Version of the normordis-pdf library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Accessibility / PDF/UA-2 ─────────────────────────────────────────────────

pub use compliance::ua::{
    AccessibilityConfig, ArtifactType, StructEvent, StructTag, StructureTree, UaError, UaValidator,
    UaWarning,
};

/// NDT format version supported by this release.
pub const NDT_VERSION: &str = "2.0.0";

/// PDF backend crate powering the output engine.
pub const PDF_BACKEND: &str = "pdf-writer";

/// Render archive format version produced by this release.
pub const ARCHIVE_VERSION: &str = archive::ARCHIVE_VERSION;

/// NCRTF format version supported by this release.
pub const NCRTF_VERSION: &str = richtext::NCRTF_VERSION;
