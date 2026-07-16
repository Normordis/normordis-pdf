# normordis-pdf

Pure-Rust institutional PDF generation for the NORMAXIS mini-app framework.

This repository now contains the standalone `normordis-pdf` library crate along with two helper tools:
- `tools/dotx2ndt`
- `tools/ndt-tools`

Bash-first build, backup and restore helpers are available in [`scripts/`](scripts/README.md). O fluxo 3-2-1 mantém cópias no SSD e no Google Drive; equivalentes PowerShell permanecem disponíveis para Windows.

[![Crate](https://img.shields.io/badge/crate-normordis--pdf-blue)](https://crates.io/crates/normordis-pdf)
[![License: EUPL-1.2](https://img.shields.io/badge/license-EUPL--1.2-blue.svg)](LICENSE)

## Documentação bilingue / Bilingual documentation

Este repositório oferece documentação em português e inglês.
- Português: introdução e início rápido abaixo.
- English: see the English sections below.
- Manual: `MANUAL.md` é atualmente em português; `MANUAL.en.md` fornece um ponto de entrada em inglês.

## Português (pt-PT)

`normordis-pdf` gera documentos formais — relatórios, cartas, certificados e formulários — diretamente em Rust, sem dependências externas (sem LaTeX, sem Typst, sem Chromium). Está orientado para padrões de administração pública portuguesa e inclui [Liberation Sans](https://github.com/liberationfonts/liberation-fonts) para renderização pronta a usar.

Três modelos de composição, todos combináveis num único documento:

| Modelo | Descrição |
|---|---|
| **Flow** | Elementos empilham-se verticalmente; quebras de página automáticas; cabeçalho re-injectado |
| **Fixed Box** | Elementos posicionados em coordenadas absolutas; não afeta o cursor |
| **Templates NDT** | Templates JSON com injeção de dados em tempo de execução |

## Início rápido

```toml
# Cargo.toml
[dependencies]
normordis-pdf = "3.0.0"
```

### Documento Flow

```rust
use normordis_pdf::{DocumentBuilder, Paragraph, Section, Spacer, TextAlign};

let pdf = DocumentBuilder::new("Relatório Mensal")
    .push(Section::new("1. Introdução", 1))
    .push(Paragraph::new("Este relatório descreve…").align(TextAlign::Justify))
    .push(Spacer::new(6.0))
    .render_to_bytes()?;

std::fs::write("output.pdf", pdf)?;
```

## English

## Overview

`normordis-pdf` generates formal documents — reports, letters, certificates, forms — directly from Rust, with no external binary dependency (no LaTeX, no Typst, no Chromium). It targets Portuguese public administration document standards and embeds [Liberation Sans](https://github.com/liberationfonts/liberation-fonts) for out-of-the-box rendering with real glyph metrics.

Three composition models, all mixable in a single document:

| Model | Description |
|---|---|
| **Flow** | Elements stack vertically; automatic page breaks; header re-injection |
| **Fixed Box** | Elements placed at absolute coordinates; no cursor effect |
| **NDT Templates** | JSON-driven document templates with runtime data injection |

## Quick Start

```toml
# Cargo.toml
[dependencies]
normordis-pdf = "3.0.0"
```

### Flow document

```rust
use normordis_pdf::{DocumentBuilder, Paragraph, Section, Spacer, TextAlign};

let pdf = DocumentBuilder::new("Relatório Mensal")
    .push(Section::new("1. Introdução", 1))
    .push(Paragraph::new("Este relatório descreve…").align(TextAlign::Justify))
    .push(Spacer::new(6.0))
    .render_to_bytes()?;

std::fs::write("output.pdf", pdf)?;
```

### NCRTF v2.0.0 rich text

```rust
use normordis_pdf::DocumentBuilder;

let ncrtf = r#"{
  "ncrtf_version": "2.0.0",
  "content": [
    {"type":"heading","level":1,"content":[{"type":"text","text":"Título"}]},
    {"type":"paragraph","alignment":"justify","content":[
      {"type":"text","text":"Texto com "},
      {"type":"text","text":"negrito","marks":["bold"]},
      {"type":"text","text":" e itálico.","marks":["italic"]}
    ]}
  ]
}"#;

let pdf = DocumentBuilder::new("Documento")
    .push_ncrtf(ncrtf)?
    .render_to_bytes()?;
```

### NDT v2.0.0 template

```rust
use normordis_pdf::DocumentBuilder;

const TEMPLATE: &str = include_str!("templates/oficio-nacional.ndt.json");

let data = serde_json::json!({
    "ndt_data": "1.0.0",
    "data": {
        "entidade": "Câmara Municipal de Exemplo",
        "numero": "2025/001",
        "data": "25 de Abril de 2025"
    }
}).to_string();

let pdf = DocumentBuilder::new("Ofício")
    .push_ndt(TEMPLATE, &data)?
    .render_to_bytes()?;
```

## Features

### Flow elements

| Type | Struct / method |
|---|---|
| Paragraph (plain or rich) | `Paragraph::new(text)` |
| Section heading | `Section::new(text, level)` |
| Ordered / bullet / checklist | `List::new(items, ListType::Bullet)` |
| Table | `Table::new(headers, rows)` |
| Image | `FlowImage::new(bytes, width_mm)` |
| Spacer | `Spacer::new(height_mm)` |
| Horizontal rule | `HorizontalRule::new()` |
| Page break | `PageBreak` |

### Fixed Box elements

| Type | Builder method |
|---|---|
| Text at absolute position | `DocumentBuilder::fixed_text(box, text, align)` |
| Image at absolute position | `DocumentBuilder::fixed_image(box, bytes, fit)` |
| Decorative line | `DocumentBuilder::fixed_line(x1, y1, x2, y2, color)` |

### NCRTF v2.0.0 — rich text format

NCRTF (NORMORDIS Canonical Rich Text Format) is a JSON schema for inline-styled paragraphs. It is the interchange format between editors (such as `@normaxis/nx-doc`) and this renderer.

```json
{
  "ncrtf_version": "2.0.0",
  "content": [
    {
      "type": "paragraph",
      "alignment": "justify",
      "content": [
        {"type": "text", "text": "Normal, "},
        {"type": "text", "text": "bold", "marks": ["bold"]},
        {"type": "text", "text": " and italic.", "marks": ["italic"]}
      ]
    },
    {
      "type": "list",
      "list_type": "bullet",
      "content": [
        {"type": "list_item", "content": [{"type": "text", "text": "Item"}]}
      ]
    }
  ]
}
```

Supported block types: `paragraph`, `heading` (levels 1–3), `list` (bullet / ordered / checklist), `blockquote`, `table`, `image`.  
Supported inline marks: `bold`, `italic`, `underline`, `strikethrough`, `superscript`, `subscript`, `code`.

### NDT v2.0.0 — document templates

NDT (NORMORDIS Document Template) is a JSON/TOML-driven template format for institutional documents. Templates define a positioned layout; runtime data is injected at render time.

**Template file** (`*.ndt.json`):

```json
{
  "ndt_version": "2.0.0",
  "schema_id": "urn:normordis:ndt:oficio-nacional",
  "versao_ndt": "1.0.0",
  "titulo": "Ofício Nacional",
  "paginas_def": [
    {
      "id": "pagina-principal",
      "graficos": [
        {
          "tipo": "texto_fixo",
          "x_mm": 25, "y_mm": 20, "largura_mm": 160, "altura_mm": 10,
          "conteudo": "{{entidade}}"
        }
      ]
    }
  ],
  "sequencia": [
    {"pagina_def": "pagina-principal", "repeticao": "unica"}
  ]
}
```

**Data file** (`*.ndt-data.json`):

```json
{
  "ndt_data": "1.0.0",
  "data": {
    "entidade": "Câmara Municipal de Exemplo",
    "numero": "2025/001"
  }
}
```

> **Note:** The NDT v2.0.0 positioned-layout renderer is currently in development. `push_ndt` parses and validates the template but returns an error until the renderer is complete.

## Examples

Run any example with:

```bash
cargo run --example <name> -p normordis-pdf
```

| Example | Description |
|---|---|
| `01_basic_document` | Flow document with headings, paragraphs, table, list |
| `02_ncrtf_document` | Document built from NCRTF rich text JSON |
| `03_ndt_template` | Document rendered from an NDT template + runtime data |
| `04_mixed_layout` | Flow + Fixed Box mixed (office letter style) |
| `05_fidelity` | Sectioned header/footer, watermark, runtime fields |
| `06_advanced_layout` | Indentation, col_span, multi-page tables |
| `07_named_styles` | Named paragraph styles with inheritance |
| `08_portuguese_spacing` | Portuguese hyphenation and line breaking |
| `09_typography` | Text decorations, tab stops, borders |
| `10_advanced_elements` | Forms, footnotes, TOC |
| `11_size_benchmark` | Large document size benchmark |
| `12_compliance` | PDF/A-1b + traceability |
| `13_accessibility` | PDF/UA-2 tagged document |
| `14_custom_fonts` | Custom TTF/OTF fonts via `font_from_bytes`, per-paragraph `.font_family()`, fallback chain |
| `15_fonts_from_dir` | Load all fonts from a directory with `fonts_from_dir` |

## Fonts

Four font families are embedded at compile time — Liberation Sans, Liberation Serif, Liberation Mono, and Libertinus Serif. No system fonts are required.

```rust
// Register any TTF/OTF font family — from bytes or from disk
let pdf = DocumentBuilder::new("Doc")
    .font_from_bytes(
        "MyFont",
        include_bytes!("assets/MyFont-Regular.ttf"),
        Some(include_bytes!("assets/MyFont-Bold.ttf")),
        None, None,
    )?
    .push(Paragraph::new("Custom font paragraph.").font_family("MyFont"))
    .render_to_bytes()?;
```

Key font APIs:

| API | Description |
|---|---|
| `DocumentBuilder::font_from_bytes(name, regular, bold?, italic?, bold_italic?)` | Register from `&[u8]` (e.g. `include_bytes!`) |
| `DocumentBuilder::font_from_file(name, regular, bold?, italic?, bold_italic?)` | Register from TTF/OTF file paths |
| `DocumentBuilder::fonts_from_dir(path)` | Scan a directory; groups files by `-Bold` / `-Italic` suffix |
| `DocumentBuilder::default_font(name)` | Change the document default family |
| `Paragraph::font_family(name)` | Per-paragraph font override |
| `FontRegistry::register_bytes` / `register_file` / `load_dir` | Direct registry manipulation |
| `DocumentStyle::font_fallback` | `FontFallbackChain` — tried in order when the requested font is not registered |

Common Word font names (`Arial`, `Calibri`, `Times New Roman`, `Cambria`, `Consolas`, etc.) are pre-registered as aliases to their Liberation equivalents.

## Version constants

```rust
normordis_pdf::VERSION        // "3.0.0" — crate version
normordis_pdf::NDT_VERSION    // "2.0.0" — NDT format version
normordis_pdf::NCRTF_VERSION  // "2.0.0" — NCRTF format version
```

## API stability

All public items re-exported from `normordis_pdf::*` are considered stable. Internal modules (`normordis_pdf::template::*`, `normordis_pdf::richtext::*`, etc.) are not stable and may change between minor versions.

## License

EUPL-1.2 — see [LICENSE](../../LICENSE) or [https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12).
