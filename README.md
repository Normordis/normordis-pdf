# normordis-pdf

Pure-Rust institutional PDF generation for the NORMAXIS mini-app framework.

This repository now contains the standalone `normordis-pdf` library crate along with two helper tools:
- `tools/dotx2ndt`
- `tools/ndt-tools`

A backup/restore helper is also available in `scripts/`.

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
normordis-pdf = "1.0.0"
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
normordis-pdf = "1.0.0"
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

### NCRTF rich text

```rust
use normordis_pdf::DocumentBuilder;

let ncrtf = r#"{
  "ncrtf": "1.0",
  "blocks": [
    {"type":"heading","level":1,"children":[{"type":"text","text":"Título","marks":[]}]},
    {"type":"paragraph","alignment":"justify","children":[
      {"type":"text","text":"Texto com ","marks":[]},
      {"type":"text","text":"negrito","marks":["bold"]},
      {"type":"text","text":" e itálico.","marks":["italic"]}
    ]}
  ]
}"#;

let pdf = DocumentBuilder::new("Documento")
    .push_ncrtf(ncrtf)?
    .render_to_bytes()?;
```

### NDT template

```rust
use normordis_pdf::DocumentBuilder;

const TEMPLATE: &str = include_str!("templates/oficio-nacional.ndt.json");

let data = serde_json::json!({
    "ndt_data": "1.0.0",
    "data": {
        "entidade": "Câmara Municipal de Exemplo",
        "numero": "2025/001",
        "data": "25 de Abril de 2025",
        "assunto": "Resposta a pedido de informação",
        "mensagem": "{\"ncrtf\":\"1.0\",\"blocks\":[{\"type\":\"paragraph\",\"alignment\":\"justify\",\"children\":[{\"type\":\"text\",\"text\":\"Informamos que o pedido foi recebido.\",\"marks\":[]}]}]}"
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

### NCRTF v1.0 — rich text format

NCRTF (NORMAXIS Canonical Rich Text Format) is a JSON schema for inline-styled paragraphs. It is the interchange format between editors (such as `@normaxis/nx-doc`) and this renderer.

```json
{
  "ncrtf": "1.0",
  "blocks": [
    {
      "type": "paragraph",
      "alignment": "justify",
      "children": [
        {"type": "text", "text": "Normal, ", "marks": []},
        {"type": "text", "text": "bold", "marks": ["bold"]},
        {"type": "text", "text": " and italic.", "marks": ["italic"]}
      ]
    },
    {
      "type": "list",
      "list_type": "bullet",
      "children": [
        {"indent": 0, "children": [{"type": "text", "text": "Item", "marks": []}]}
      ]
    }
  ]
}
```

Supported block types: `paragraph`, `heading` (levels 1–4), `list` (bullet / ordered / checklist).  
Supported inline marks: `bold`, `italic`, `underline`, `strikethrough`, `code`.

### NDT v1.0.0 — document templates

NDT (NORMAXIS Document Template) is a JSON-driven template format for institutional documents. Templates define a layout schema; runtime data is injected at render time.

**Template file** (`*.ndt.json`):

```json
{
  "ndt": "1.0.0",
  "meta": { "title": "Relatório", "description": "…" },
  "placeholders": {
    "entity_name": { "type": "string", "required": true },
    "body":        { "type": "ncrtf",  "required": false }
  },
  "body": [
    { "type": "heading", "text": "{{entity_name}}", "level": 1 },
    {
      "type": "conditional",
      "condition": "body", "operator": "exists",
      "then": [{ "type": "rich_text", "content": "{{body}}", "source": "placeholder" }],
      "else": [{ "type": "paragraph", "text": "Sem conteúdo." }]
    }
  ]
}
```

**Data file** (`*.ndt-data.json`):

```json
{
  "ndt_data": "1.0.0",
  "data": {
    "entity_name": "Câmara Municipal de Exemplo",
    "body": "{\"ncrtf\":\"1.0\",\"blocks\":[]}"
  }
}
```

Supported body element types: `paragraph`, `heading`, `rich_text`, `table`, `list`, `image`,
`spacer`, `horizontal_rule`, `page_break`, `fixed_text`, `fixed_image`, `fixed_line`,
`fixed_box`, `zone_ref`, `conditional`, `repeat`, `include`.

Supported conditional operators: `exists`, `empty`, `eq`, `neq`, `gt`, `lt`.

## Bundled templates

Ready-to-use NDT templates are provided under `examples/templates/`:

| File | Description |
|---|---|
| `relatorio-simples.ndt.json` | Simple institutional report |
| `oficio-nacional.ndt.json` | Official letter (ofício) |
| `certidao-generica.ndt.json` | Generic certificate (certidão) |
| `formulario-generico.ndt.json` | Generic two-section form |

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
normordis_pdf::VERSION        // "1.0.0" — crate version
normordis_pdf::NDT_VERSION    // "1.0.0" — NDT engine version
normordis_pdf::NCRTF_VERSION  // "1.0"   — NCRTF parser version
```

## API stability

All public items re-exported from `normordis_pdf::*` are considered stable from v1.0.0 onwards. Internal modules (`normordis_pdf::template::*`, `normordis_pdf::richtext::*`, etc.) are not stable and may change between minor versions.

## License

EUPL-1.2 — see [LICENSE](../../LICENSE) or [https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12).

