# Changelog

All notable changes to `normordis-pdf` are documented here.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) — versions follow [Semantic Versioning](https://semver.org/).

---

## [1.3.0] — 2026-04-26

### Added

- **Named styles** — `NamedStyle` struct with optional fields; style inheritance chain via `extends`; cycle detection returns `StyleCycleError`; unknown name returns `UnknownStyle`
- **7 built-in styles** — `normal`, `heading_1`, `heading_2`, `heading_3`, `caption`, `table_header`, `table_body`; accessible without declaring in `DocumentStyle.named_styles`
- **`StyleResolver`** — resolves a named style to a fully-populated `ResolvedStyle` (no `Option` fields); `StyleResolver::new(styles, doc_style).resolve("name")`
- **`DocumentStyle.named_styles`** — `HashMap<String, NamedStyle>` for user-defined styles; user styles override built-ins with the same name
- **`Paragraph` — named style support** — `.style("caption")` builder method; `style_ref` field; resolved values used as defaults for `font_size`, `bold`, `italic`, `alignment`, `indent_*`, `space_before/after`
- **`space_before_mm` / `space_after_mm`** — per-paragraph spacing in mm; suppressed at top of page (matching Word behaviour); builder methods `.space_before(mm)` and `.space_after(mm)`
- **Tab stops** — `TabStop { position_mm, alignment, leader }` with factory methods `.left()`, `.right()`, `.center()`, `.decimal()`, `.with_leader(char)`; `TabStopAlign { Left, Right, Center, Decimal }`; added to `Paragraph` via `.tab_stop(stop)` builder; `\t` characters in `TextRun` text are processed by the layout engine
- **`PageFlow::is_top_of_page()`** — returns `true` when cursor is within 0.5 mm of the top margin; used to suppress `space_before`
- **`Section` — named style support** — `.style("heading_1")` builder method; `style_ref` field; defaults to `heading_1/2/3` built-ins by level
- **`TableStyle`** — named table style struct with `outer_border`, `inner_border`, `header_background`, `stripe_color`; factory methods `TableStyle::grid()`, `.bordered()`, `.striped()`, `.plain()`
- **`CellPadding`** — per-edge cell insets `{ top_mm, bottom_mm, left_mm, right_mm }`; default 1/1/2/2 mm; factory methods `.uniform(mm)`, `.horizontal_vertical(h, v)`; `TableCell.padding` field; `.padding(p)` builder method
- **`Table::with_table_style()`** — builder method to apply a `TableStyle`
- **Example `07_named_styles`** — demonstrates all v1.3.0 features
- **30 new tests** in `tests/v130_styles.rs`
- **`tools/dotx2ndt`** — CLI tool that extracts Word `.dotx` style definitions and generates an NDT-compatible named-styles JSON skeleton

### Changed

- `TextLayoutEngine::layout_runs` gains a `tab_stops: &[TabStop]` parameter; all internal callers pass `&[]` — no behaviour change for existing code
- `Section::render` now uses `StyleResolver` instead of hardcoded level-based sizing; output is visually identical for the default built-in styles
- `NDT_VERSION` bumped to `"1.3.0"`; `NCRTF_VERSION` bumped to `"1.1.0"`

---

## [1.2.0] — 2026-04-26

### Added

- **Table pagination** — tables spanning multiple pages no longer silently truncate rows; header rows are re-printed on each continuation page
- **List pagination** — `BulletList`, `OrderedList`, and `CheckList` span multiple pages correctly
- **Kerning** — optional pair kerning via `ab_glyph`; enable with feature flag `kerning`
- **`col_span` / `row_span`** in `TableCell` — merged-header tables; builder methods `.col_span(n)` and `.row_span(n)`
- **Z-index for Fixed Box** — `FixedBox::z_index: i32` (default 0); higher z-index renders on top; sorted before each page flush
- **Character spacing** — `TextRun::letter_spacing_mm: f64` (default 0.0); added to glyph-advance measurement and line wrapping
- **Per-cell borders** — `CellBorders`, `CellBorder`, `BorderLineStyle` (Solid / Dashed / Dotted / None) per table cell edge
- **Paragraph indentation** — `indent_left_mm`, `indent_right_mm`, `indent_first_line_mm`; builder methods `.indent_left()`, `.indent_right()`, `.indent_first_line()`
- **`TextAlign::Right`** reintroduced — right-aligned dates, numeric columns, letter headings; no breaking changes
- **`RenderResult`** — `Element::render` now returns `crate::Result<RenderResult>` with `has_more` flag; enables multi-page element continuation
- **`TableBuilder`** — fluent builder via `Table::builder()` with `header_row()`, `row()`, `stripe()`, `col_widths()`, `build()`
- **Example `06_advanced_layout`** — demonstrates all v1.2.0 features (indentation, Right alignment, col_span, multi-page table and list)
- **25 new tests** in `tests/v120_advanced.rs` covering all feature areas

### Fixed

- **`Paragraph::estimated_height_mm`** — replaced hardcoded `10.0` with character-width heuristic based on actual font size and content length

### Changed

- `Element::render` signature: `fn render(&self, ctx: &mut RenderContext) -> crate::Result<RenderResult>` (was `-> crate::Result<()>`); all simple elements return `RenderResult::done()` — no behaviour change
- `NDT_VERSION` bumped to `"1.2.0"` (backwards compatible with 1.0.0 and 1.1.0)

---

## [1.1.0] — 2026-04-25

### Added

- **`SectionedHeader`** — per-page-type institutional headers (`first_page`, `odd_pages`, `even_pages`) via `DocumentBuilder::sectioned_header()`
- **`SectionedFooter`** — per-page-type footers (`first_page`, `odd_pages`, `even_pages`) via `DocumentBuilder::sectioned_footer()`; `all_pages()` convenience builder
- **`PageFooter` text columns** — `.left()`, `.center()`, `.right()` builder methods; all columns accept runtime fields; separator line and column layout now rendered
- **`RowHeight` enum** — `Auto` / `AtLeast(f64)` / `Exact(f64)` row height control for `TableRow`; builder methods `height_exact()` and `height_at_least()`
- **`TableRow` / `TableCell` types** — `TableRow::plain(Vec<String>)` for simple construction; `TableRow::new(Vec<TableCell>)` for rich construction
- **Runtime calculated fields** — `{{page}}`, `{{total_pages}}`, `{{today}}`, `{{now}}` resolved in all footer text columns via `RuntimeContext` and `resolve_runtime_fields()`
- **`Watermark` struct** — diagonal text watermark on every page via `DocumentBuilder::watermark()`; configurable `text`, `opacity`, `color`, `font_size`, `angle_deg`
- **Two-pass page count** — `render_to_bytes` now estimates total pages before the main render so `{{total_pages}}` is accurate
- **`RenderContext` page tracking** — `page_number` and `total_pages` fields propagated to all elements
- **Example `05_fidelity`** — demonstrates all v1.1.0 features (sectioned header/footer, watermark, exact row heights, runtime fields)
- **13 new tests** in `tests/v110_fidelity.rs` covering all five feature areas

### Changed

- `Table::new` rows parameter changed from `Vec<Vec<String>>` to `Vec<TableRow>` — use `TableRow::plain()` for migration
- `NDT_VERSION` bumped to `"1.1.0"` (`ENGINE_NDT_VERSION` constant)
- `chrono` added as a workspace dependency (used by `RuntimeContext` for date/time fields)

---

## [1.0.0] — 2026-04-25

First stable release. API is considered stable from this version onwards.

### Added

- **NDT v1.0.0 template engine** (`src/template/`) — parse, validate, resolve, and render JSON-driven document templates with 16 body element types
  - `BodyElement` enum: `Paragraph`, `Heading`, `RichText`, `Table`, `List`, `Image`, `Spacer`, `HorizontalRule`, `PageBreak`, `FixedText`, `FixedImage`, `FixedLine`, `FixedBox`, `ZoneRef`, `Conditional`, `Repeat`, `Include`
  - `ConditionalElement` with operators: `exists`, `empty`, `eq`, `neq`, `gt`, `lt`
  - `RepeatElement` for list-driven repetition
  - Nested key resolution (`obj.field` syntax in data)
  - Placeholder type validation (`string`, `ncrtf`)
  - `TemplateError` enum with 8 variants for structured error reporting
- **`DocumentBuilder::push_ndt(template, data)`** — renders an NDT template into the document builder pipeline
- **`DocumentBuilder::push_ncrtf(json)`** — renders NCRTF rich text directly
- **Liberation Sans fonts** embedded at compile time (Regular, Bold, Italic, Bold Italic) — no system fonts required
- **`ab_glyph` integration** — real glyph advance-width metrics for accurate text layout
- **`Orientation` enum** (`Portrait` / `Landscape`) added to `DocumentStyle`
- **`elements::fixed` module** — unified re-export for all fixed-position element types (`FixedTextBox`, `FixedImageBox`, `FixedLineElement`, `VerticalAlign`, `ImageFit`)
- **`ListItem` type alias** for `ListItemElement`
- **Version constants**: `VERSION`, `NDT_VERSION`, `NCRTF_VERSION` available from crate root
- **4 runnable examples** with matching NDT templates:
  - `01_basic_document` — flow document
  - `02_ncrtf_document` — NCRTF rich text
  - `03_ndt_template` — NDT template with runtime data
  - `04_mixed_layout` — flow + fixed box (office letter)
- **4 bundled NDT templates**: `relatorio-simples`, `oficio-nacional`, `certidao-generica`, `formulario-generico`
- Comprehensive crate-level doc comments with Quick Start and NDT examples

### Changed

- `TextAlign` (canonical enum) replaces the previous split `Alignment` (layout) + `TextAlignment` (paragraph/fixed) — all public APIs now use `TextAlign`
- `TextLayoutEngine` methods now take `&FontRegistry` as first parameter, eliminating self-referential struct issues
- `FontRegistry` implements `Default` and embeds Liberation Sans automatically
- `FontVariant` implements `Clone` via re-parsing from stored bytes
- `NormaxisPdfError` gains a `Template(String)` variant
- Crate version is now managed independently (`version = "1.0.0"`) rather than inheriting workspace version
- Publication metadata added to `Cargo.toml` (repository, keywords, categories, description)

### Removed

- `Alignment` enum (replaced by `TextAlign`)
- `TextAlignment` enum (replaced by `TextAlign`)
- Local `TextAlign` in `richtext::model` (consolidated into `layout::TextAlign`)

---

## [0.7.0] — 2026-04-24

### Added

- `FixedBox` layout type with `OverflowPolicy::Shrink` (auto-reduce font size to fit)
- `VerticalAlign` for fixed text boxes (Top / Middle / Bottom)
- `BorderStyle` / `BoxBorder` for table and fixed box borders
- `PageFlow` struct for cursor and page management

---

## [0.6.0] — 2026-04-24

### Added

- NCRTF v1.0 parser (`push_ncrtf`) with inline marks (bold, italic, underline, strikethrough, code)
- `RichText` flow element
- `TextRun` / `AppliedStyle` types for styled inline text

---

## [0.5.0] — 2026-04-24

### Added

- `Table` flow element with configurable headers, rows, and column widths
- `List` flow element (bullet / ordered / checklist) with `ListItem` / `ListItemElement`

---

## [0.4.0] — 2026-04-24

### Added

- `FixedTextBox`, `FixedImageBox`, `FixedLineElement` for absolute-coordinate positioning
- Mixed layout mode (flow + fixed in one document)

---

## [0.3.0] — 2026-04-23

### Added

- `TextLayoutEngine` with real glyph metrics via `FontRegistry`
- Word-wrap, line-break, and justification logic
- `LayoutResult` / `LineBox` / `LineSegment` types

---

## [0.2.0] — 2026-04-23

### Added

- `FontRegistry` with `FontFamily` / `FontVariant`
- TTF loading via `printpdf` + `ab_glyph` glyph advance metrics

---

## [0.1.0] — 2026-04-22

### Added

- Initial scaffold: `DocumentBuilder`, `Document`, `PageLayout`
- `Paragraph`, `Section` (heading), `Spacer`, `PageBreak`, `HorizontalRule` flow elements
- `DocumentStyle` with `PageSize`, `RgbColor`
- printpdf 0.9 Op-based renderer

