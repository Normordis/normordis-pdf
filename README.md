# normordis-pdf

Compliance-grade institutional PDF generation in pure Rust — PDF/A, PDF/UA-2 and PAdES — reference implementation of the [NORMORDIS formats](https://github.com/Normordis/normordis-formats).

[![Crate](https://img.shields.io/crates/v/normordis-pdf.svg)](https://crates.io/crates/normordis-pdf)
[![Docs](https://docs.rs/normordis-pdf/badge.svg)](https://docs.rs/normordis-pdf)
[![License: EUPL-1.2](https://img.shields.io/badge/license-EUPL--1.2-blue.svg)](LICENSE)
[![CI](https://github.com/Normordis/normordis-pdf/actions/workflows/ci.yml/badge.svg)](https://github.com/Normordis/normordis-pdf/actions/workflows/ci.yml)
[![veraPDF](https://github.com/Normordis/normordis-pdf/actions/workflows/verapdf.yml/badge.svg)](https://github.com/Normordis/normordis-pdf/actions/workflows/verapdf.yml)

Este repositório contém a crate `normordis-pdf` e três ferramentas auxiliares:

| Diretório | Função |
|---|---|
| `tools/dotx2ndt` | converte `.docx`/`.dotx` Word em templates NDT |
| `tools/ndt-tools` | utilitários CLI para ficheiros NDT |
| `tools/verify-pdf` | invoca o veraPDF e reporta conformidade PDF/A e PDF/UA-2 |

**Documentação** · [MANUAL.md](MANUAL.md) (pt-PT) · [MANUAL.en.md](MANUAL.en.md) · [docs/man/ncrtf.md](docs/man/ncrtf.md) · [CHANGELOG.md](CHANGELOG.md) · [SECURITY.md](SECURITY.md) · [scripts/README.md](scripts/README.md)

**Arquitetura e proveniência** · [docs/architecture/](docs/architecture/) (decisões) · [AGENTS.md](AGENTS.md) · [AI_USAGE.md](AI_USAGE.md)

---

## Português (pt-PT)

### O que é

`normordis-pdf` gera documentos formais — ofícios, relatórios, certidões, formulários — diretamente em Rust, sem dependências externas (sem LaTeX, sem Typst, sem Chromium, sem Java em tempo de execução). Produz PDF conforme às normas que a lei exige — **PDF/A** (arquivo, ISO 19005), **PDF/UA-2** (acessibilidade, ISO 14289-2:2024) e **PAdES** (assinatura digital) — e essa conformidade é verificada por um validador independente (veraPDF) em integração contínua, não apenas afirmada.

O alvo primário é a administração pública, sujeita ao princípio da legalidade. Qualquer setor com obrigações de conformidade documental — financeiro, saúde, seguros — pode usá-lo nos mesmos termos (EUPL-1.2).

### Conformidade

| `PdfStandard` | Norma | Notas |
|---|---|---|
| `PdfA4Ua2` **(omissão)** | ISO 19005-4 (PDF/A-4f) + ISO 14289-2 | PDF 2.0; arquivo e acessibilidade em simultâneo |
| `PdfA1b` | ISO 19005-1 | recusa transparência (§6.4) em vez de emitir ficheiro não conforme |
| `PdfA2b` | ISO 19005-2 | permite transparência (marcas de água translúcidas) |
| `PdfUa2` | ISO 14289-2:2024 | acessibilidade isolada, sem PDF/A |
| `Pdf17` | PDF 1.7 | sem requisitos de conformidade |

**O que a CI verifica com o veraPDF 1.30.2:** PDF/A-1b, PDF/A-2b e PDF/UA-2, a cada alteração, como *check* obrigatório para entrar em `main`. A validação independente do perfil PDF/A-4f está pendente.

Uma exigência de desenho: quando dois requisitos são mutuamente exclusivos (ex.: PDF/A-1b com transparência), a biblioteca **recusa** e explica, citando a norma — nunca emite silenciosamente um ficheiro que declara uma conformidade que não tem.

### Início rápido

```toml
[dependencies]
normordis-pdf = "3.0.0"
```

```rust
use normordis_pdf::{DocumentBuilder, Paragraph, Section, Spacer, TextAlign};

let pdf = DocumentBuilder::new("Relatório Mensal")
    .push(Section::new("1. Introdução", 1))
    .push(Paragraph::new("Este relatório descreve…").align(TextAlign::Justify))
    .push(Spacer::new(6.0))
    .render_to_bytes()?;   // PDF/A-4f + PDF/UA-2 por omissão

std::fs::write("output.pdf", pdf)?;
```

Com metadados de rastreabilidade e classificação de segurança (marca de água automática quando a classificação não é `Public`):

```rust
use normordis_pdf::{
    DocumentBuilder, NDT_VERSION, PdfStandard, SecurityClassification, TraceabilityMetadata, VERSION,
};

let pdf = DocumentBuilder::new("Despacho")
    .standard(PdfStandard::PdfA2b)
    .traceability(TraceabilityMetadata {
        engine_version: VERSION.into(),
        framework_version: None,
        entity_id: "cm-lisboa".into(),
        document_ref: Some("DESP/2026/042".into()),
        classification: SecurityClassification::Internal,
        generated_at: "2026-09-04T10:00:00Z".into(),
        ndt_version: NDT_VERSION.into(),
    })
    .render_to_bytes()?;
```

### Assinatura digital (PAdES)

A biblioteca prepara o documento e expõe os *byte ranges*; a assinatura PKCS#7 é produzida fora (HSM, cartão de cidadão, serviço qualificado) e depois incorporada. Os bytes assinados nunca são reserializados.

```rust
use normordis_pdf::{DocumentBuilder, SignatureConfig};

let prepared = DocumentBuilder::new("Contrato")
    .sign(SignatureConfig::default())
    .render_prepared_for_signing()?;

let pkcs7_der = my_hsm.sign(&prepared.bytes_to_sign())?;   // fora da biblioteca
let signed_pdf = prepared.embed_signature(&pkcs7_der)?;
```

Com a feature `tsa`, `timestamp_pkcs7` acrescenta um carimbo temporal RFC 3161 ao PKCS#7.

### Formatos NORMORDIS

As especificações normativas vivem em [`normordis-formats`](https://github.com/Normordis/normordis-formats): [NDF 1.0.0](https://github.com/Normordis/normordis-formats/blob/main/specs/ndf/SPEC.md) (instância documental), [NDT 2.0.0](https://github.com/Normordis/normordis-formats/blob/main/specs/ndt/SPEC.md) (template de apresentação) e [NCRTF 2.0.0](https://github.com/Normordis/normordis-formats/blob/main/specs/ncrtf/SPEC.md) (texto rico canónico).

- **NCRTF** — `DocumentBuilder::push_ncrtf(json)` converte um documento NCRTF 2.0.0 em elementos. Blocos: `paragraph`, `heading`, `list` (bullet / ordered / checklist), `blockquote`, `table`, `image`. Marcas: `bold`, `italic`, `underline`, `strikethrough`, `superscript`, `subscript`, `code`.
- **NDT** — `DocumentBuilder::push_ndt(template, data)` faz *parsing* e validação de templates NDT 2.0.0. **O renderizador de layout posicionado ainda não está implementado**: `push_ndt` devolve erro com qualquer template válido. É trabalho em curso, não um defeito escondido — o exemplo `03_ndt_template` falha por desenho até o renderizador existir.
- **Arquivo de renderização** (`RenderArchive`, `compile_ndt`, `verify_archive`, `ArchiveRegistry`) — formato interno da crate, com JSON canónico, hashes e trilho de auditoria. Não confundir com o NDF da especificação.

A 3.0.0 é a última versão em que a crate define os formatos por si própria. A partir da 4.0.0, `normordis-formats` é a referência exclusiva e a crate valida contra os schemas normativos aí publicados — ver `docs/architecture/`.

### Elementos

Três modelos de composição, combináveis no mesmo documento: **Flow** (empilhamento vertical, quebras automáticas, cabeçalho re-injetado), **Fixed Box** (coordenadas absolutas em mm) e **NDT** (template + dados).

| Flow | Tipo |
|---|---|
| Parágrafo (simples ou rico) | `Paragraph` |
| Título de secção | `Section::new(texto, nível)` |
| Listas | `BulletList`, `OrderedList`, `CheckList` |
| Tabela | `Table::new(headers, rows)` ou `Table::builder()` |
| Imagem | `ImageElement::new(bytes).alt(texto)` |
| Espaçador / régua / quebra de página | `Spacer`, `HorizontalRuleElement`, `PageBreakElement` |
| Quebra de secção (orientação, margens) | `SectionBreak` |
| Índice | `TableOfContents` |
| Nota de rodapé | `FootnoteRef` + `DocumentBuilder::add_footnote` |
| Campos de formulário | `FormField` (texto, checkbox, rádio, combo, lista) |
| Cabeçalho / rodapé | `InstitutionalHeader`, `SectionedHeader`, `PageFooter`, `SectionedFooter` |

| Fixed Box | Método |
|---|---|
| Texto em posição absoluta | `DocumentBuilder::fixed_text` |
| Imagem em posição absoluta | `DocumentBuilder::fixed_image` |
| Linha decorativa | `DocumentBuilder::fixed_line` |

Estilos nomeados com herança (`DocumentStyle`, `NamedStyle`), tabulações com *leaders*, decorações, bordas e alinhamento justificado com quebra de linha Knuth–Plass opcional.

### Fontes

Quatro famílias embutidas em tempo de compilação — Liberation Sans, Liberation Serif, Liberation Mono e Libertinus Serif — com métricas reais de glifos (rustybuzz + ttf-parser) e *subsetting* na saída. Nenhuma fonte de sistema é necessária. Nomes comuns do Word (`Arial`, `Calibri`, `Times New Roman`, `Cambria`, `Consolas`, …) resolvem para o equivalente Liberation.

```rust
let pdf = DocumentBuilder::new("Doc")
    .font_from_bytes("MyFont", include_bytes!("MyFont-Regular.ttf"),
                     Some(include_bytes!("MyFont-Bold.ttf")), None, None)?
    .push(Paragraph::new("Texto.").font_family("MyFont"))
    .render_to_bytes()?;
```

`font_from_file`, `fonts_from_dir`, `default_font`, `FontFallbackChain` e, com a feature `system-fonts`, `fonts_from_system`.

### Features Cargo

| Feature | Efeito |
|---|---|
| `system-fonts` | `fonts_from_system()` via `fontdb` |
| `hyphenation` | hifenização (dicionários embutidos) no quebrador de linha |
| `optimal_wrap` | quebra de linha Knuth–Plass por parágrafo (`LineBreakingMode`) |
| `tsa` | carimbo temporal RFC 3161 (`request_timestamp`, `timestamp_pkcs7`) — requer rede |
| `ffi` | ABI C (`generate_pdf_from_json`, `generate_pdf_from_ndt`, `free_pdf_result`) para uso a partir de outras linguagens; a crate compila também como `cdylib` |

### Exemplos

```bash
cargo run --example <nome> -p normordis-pdf
```

| Exemplo | Demonstra |
|---|---|
| `01_basic_document` | documento Flow com títulos, parágrafos, tabela, lista |
| `02_ncrtf_document` | documento construído a partir de NCRTF 2.0.0 |
| `03_ndt_template` | template NDT 2.0.0 + dados — **falha por desenho até o renderizador existir** |
| `04_mixed_layout` | Flow + Fixed Box (ofício) |
| `05_fidelity` | cabeçalho/rodapé por secção, marca de água, campos de runtime |
| `06_advanced_layout` | indentação, `col_span`, tabelas multipágina |
| `07_named_styles` | estilos nomeados com herança |
| `08_portuguese_spacing` | hifenização e quebra de linha em português |
| `09_typography` | decorações, tabulações, bordas |
| `10_advanced_elements` | formulários, notas de rodapé, índice |
| `11_size_benchmark` | dimensão de documentos grandes |
| `12_compliance` | PDF/A-1b e PDF/A-2b com rastreabilidade — validado pelo veraPDF na CI |
| `13_accessibility` | PDF/UA-2 com árvore de estrutura — validado pelo veraPDF na CI |
| `14_custom_fonts` | fontes TTF/OTF próprias, `font_family` por parágrafo, cadeia de *fallback* |
| `15_fonts_from_dir` | carregar todas as fontes de um diretório |

### Constantes de versão

```rust
normordis_pdf::VERSION          // "3.0.0" — versão da crate
normordis_pdf::NDT_VERSION      // "2.0.0" — NDT suportado
normordis_pdf::NCRTF_VERSION    // "2.0.0" — NCRTF suportado
normordis_pdf::ARCHIVE_VERSION  // formato do arquivo de renderização
normordis_pdf::PDF_BACKEND      // "pdf-writer"
```

### Estabilidade da API

Os itens re-exportados em `normordis_pdf::*` são considerados estáveis dentro de um *major*. Os módulos internos (`template::*`, `richtext::*`, `layout::*`, `backend::*`, …) não são estáveis.

---

## English

### What it is

`normordis-pdf` generates formal documents — official letters, reports, certificates, forms — directly from Rust, with no external dependency at run time (no LaTeX, no Typst, no Chromium, no Java). Output conforms to the standards the law requires — **PDF/A** (archival, ISO 19005), **PDF/UA-2** (accessibility, ISO 14289-2:2024) and **PAdES** (digital signatures) — and that conformance is checked by an independent validator (veraPDF) in CI rather than merely claimed.

Its primary target is public administration, bound by the principle of legality. Any sector with document-compliance obligations — finance, health, insurance — can use it on the same terms (EUPL-1.2).

### Conformance

| `PdfStandard` | Standard | Notes |
|---|---|---|
| `PdfA4Ua2` **(default)** | ISO 19005-4 (PDF/A-4f) + ISO 14289-2 | PDF 2.0; archival and accessible at once |
| `PdfA1b` | ISO 19005-1 | rejects transparency (§6.4) instead of emitting a non-conformant file |
| `PdfA2b` | ISO 19005-2 | allows transparency (translucent watermarks) |
| `PdfUa2` | ISO 14289-2:2024 | accessibility alone, no PDF/A |
| `Pdf17` | PDF 1.7 | no conformance requirements |

**What CI verifies with veraPDF 1.30.2:** PDF/A-1b, PDF/A-2b and PDF/UA-2, on every change, as a required check for `main`. Independent validation of the PDF/A-4f profile is pending.

A design rule: when two requirements are mutually exclusive (e.g. PDF/A-1b with transparency), the library **refuses** and explains, citing the standard — it never silently emits a file that claims a conformance it does not have.

### Quick start

```toml
[dependencies]
normordis-pdf = "3.0.0"
```

```rust
use normordis_pdf::{DocumentBuilder, Paragraph, Section, Spacer, TextAlign};

let pdf = DocumentBuilder::new("Monthly Report")
    .push(Section::new("1. Introduction", 1))
    .push(Paragraph::new("This report describes…").align(TextAlign::Justify))
    .push(Spacer::new(6.0))
    .render_to_bytes()?;   // PDF/A-4f + PDF/UA-2 by default

std::fs::write("output.pdf", pdf)?;
```

With traceability metadata and a security classification (an automatic watermark is applied when the classification is not `Public`):

```rust
use normordis_pdf::{
    DocumentBuilder, NDT_VERSION, PdfStandard, SecurityClassification, TraceabilityMetadata, VERSION,
};

let pdf = DocumentBuilder::new("Decision")
    .standard(PdfStandard::PdfA2b)
    .traceability(TraceabilityMetadata {
        engine_version: VERSION.into(),
        framework_version: None,
        entity_id: "cm-lisboa".into(),
        document_ref: Some("DEC/2026/042".into()),
        classification: SecurityClassification::Internal,
        generated_at: "2026-09-04T10:00:00Z".into(),
        ndt_version: NDT_VERSION.into(),
    })
    .render_to_bytes()?;
```

### Digital signatures (PAdES)

The library prepares the document and exposes the byte ranges; the PKCS#7 signature is produced outside (HSM, smart card, qualified service) and then embedded. Signed bytes are never re-serialised.

```rust
use normordis_pdf::{DocumentBuilder, SignatureConfig};

let prepared = DocumentBuilder::new("Contract")
    .sign(SignatureConfig::default())
    .render_prepared_for_signing()?;

let pkcs7_der = my_hsm.sign(&prepared.bytes_to_sign())?;   // outside the library
let signed_pdf = prepared.embed_signature(&pkcs7_der)?;
```

With the `tsa` feature, `timestamp_pkcs7` adds an RFC 3161 timestamp to the PKCS#7.

### NORMORDIS formats

The normative specifications live in [`normordis-formats`](https://github.com/Normordis/normordis-formats): [NDF 1.0.0](https://github.com/Normordis/normordis-formats/blob/main/specs/ndf/SPEC.md) (document instance), [NDT 2.0.0](https://github.com/Normordis/normordis-formats/blob/main/specs/ndt/SPEC.md) (presentation template) and [NCRTF 2.0.0](https://github.com/Normordis/normordis-formats/blob/main/specs/ncrtf/SPEC.md) (canonical rich text).

- **NCRTF** — `DocumentBuilder::push_ncrtf(json)` converts an NCRTF 2.0.0 document into elements. Blocks: `paragraph`, `heading`, `list` (bullet / ordered / checklist), `blockquote`, `table`, `image`. Marks: `bold`, `italic`, `underline`, `strikethrough`, `superscript`, `subscript`, `code`.
- **NDT** — `DocumentBuilder::push_ndt(template, data)` parses and validates NDT 2.0.0 templates. **The positioned-layout renderer is not implemented yet**: `push_ndt` returns an error for any valid template. This is work in progress, not a hidden defect — example `03_ndt_template` fails by design until the renderer exists.
- **Render archive** (`RenderArchive`, `compile_ndt`, `verify_archive`, `ArchiveRegistry`) — the crate's internal archive format, with canonical JSON, hashes and an audit trail. Not to be confused with the NDF of the specification.

3.0.0 is the last version in which the crate defines the formats by itself. From 4.0.0 on, `normordis-formats` is the exclusive reference and the crate validates against the normative schemas published there — see `docs/architecture/`.

### Elements

Three composition models, mixable in one document: **Flow** (vertical stacking, automatic page breaks, header re-injection), **Fixed Box** (absolute coordinates in mm) and **NDT** (template + data).

| Flow | Type |
|---|---|
| Paragraph (plain or rich) | `Paragraph` |
| Section heading | `Section::new(text, level)` |
| Lists | `BulletList`, `OrderedList`, `CheckList` |
| Table | `Table::new(headers, rows)` or `Table::builder()` |
| Image | `ImageElement::new(bytes).alt(text)` |
| Spacer / rule / page break | `Spacer`, `HorizontalRuleElement`, `PageBreakElement` |
| Section break (orientation, margins) | `SectionBreak` |
| Table of contents | `TableOfContents` |
| Footnote | `FootnoteRef` + `DocumentBuilder::add_footnote` |
| Form fields | `FormField` (text, checkbox, radio, combo, list) |
| Header / footer | `InstitutionalHeader`, `SectionedHeader`, `PageFooter`, `SectionedFooter` |

| Fixed Box | Method |
|---|---|
| Text at an absolute position | `DocumentBuilder::fixed_text` |
| Image at an absolute position | `DocumentBuilder::fixed_image` |
| Decorative line | `DocumentBuilder::fixed_line` |

Named styles with inheritance (`DocumentStyle`, `NamedStyle`), tab stops with leaders, decorations, borders, and justified text with optional Knuth–Plass line breaking.

### Fonts

Four families are embedded at compile time — Liberation Sans, Liberation Serif, Liberation Mono and Libertinus Serif — with real glyph metrics (rustybuzz + ttf-parser) and output subsetting. No system fonts are required. Common Word font names (`Arial`, `Calibri`, `Times New Roman`, `Cambria`, `Consolas`, …) resolve to their Liberation equivalents.

```rust
let pdf = DocumentBuilder::new("Doc")
    .font_from_bytes("MyFont", include_bytes!("MyFont-Regular.ttf"),
                     Some(include_bytes!("MyFont-Bold.ttf")), None, None)?
    .push(Paragraph::new("Text.").font_family("MyFont"))
    .render_to_bytes()?;
```

Also `font_from_file`, `fonts_from_dir`, `default_font`, `FontFallbackChain` and, with the `system-fonts` feature, `fonts_from_system`.

### Cargo features

| Feature | Effect |
|---|---|
| `system-fonts` | `fonts_from_system()` via `fontdb` |
| `hyphenation` | hyphenation (embedded dictionaries) in the line breaker |
| `optimal_wrap` | per-paragraph Knuth–Plass line breaking (`LineBreakingMode`) |
| `tsa` | RFC 3161 timestamps (`request_timestamp`, `timestamp_pkcs7`) — needs network |
| `ffi` | C ABI (`generate_pdf_from_json`, `generate_pdf_from_ndt`, `free_pdf_result`) for use from other languages; the crate also builds as `cdylib` |

### Examples

```bash
cargo run --example <name> -p normordis-pdf
```

| Example | Shows |
|---|---|
| `01_basic_document` | Flow document with headings, paragraphs, table, list |
| `02_ncrtf_document` | document built from NCRTF 2.0.0 |
| `03_ndt_template` | NDT 2.0.0 template + data — **fails by design until the renderer exists** |
| `04_mixed_layout` | Flow + Fixed Box (office letter) |
| `05_fidelity` | sectioned header/footer, watermark, runtime fields |
| `06_advanced_layout` | indentation, `col_span`, multi-page tables |
| `07_named_styles` | named styles with inheritance |
| `08_portuguese_spacing` | Portuguese hyphenation and line breaking |
| `09_typography` | decorations, tab stops, borders |
| `10_advanced_elements` | forms, footnotes, table of contents |
| `11_size_benchmark` | large-document size benchmark |
| `12_compliance` | PDF/A-1b and PDF/A-2b with traceability — veraPDF-validated in CI |
| `13_accessibility` | PDF/UA-2 with structure tree — veraPDF-validated in CI |
| `14_custom_fonts` | custom TTF/OTF fonts, per-paragraph `font_family`, fallback chain |
| `15_fonts_from_dir` | load every font in a directory |

### Version constants

```rust
normordis_pdf::VERSION          // "3.0.0" — crate version
normordis_pdf::NDT_VERSION      // "2.0.0" — supported NDT
normordis_pdf::NCRTF_VERSION    // "2.0.0" — supported NCRTF
normordis_pdf::ARCHIVE_VERSION  // render-archive format
normordis_pdf::PDF_BACKEND      // "pdf-writer"
```

### API stability

Items re-exported from `normordis_pdf::*` are considered stable within a major version. Internal modules (`template::*`, `richtext::*`, `layout::*`, `backend::*`, …) are not.

---

## Utilização de IA / AI-assisted development

Este projeto é desenvolvido com assistência de IA generativa (Claude Code) sob
controlo arquitetural humano. A política, o que é e não é delegado, e a
convenção de registo por commit estão em [AI_USAGE.md](AI_USAGE.md) e
[docs/ai-provenance.md](docs/ai-provenance.md).

This project is developed with generative-AI assistance (Claude Code) under
human architectural control. The policy, the division of responsibility and
the per-commit provenance convention are in [AI_USAGE.md](AI_USAGE.md) and
[docs/ai-provenance.md](docs/ai-provenance.md).

---

## License

EUPL-1.2 — see [LICENSE](LICENSE) or [https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12).

### Bundled fonts

This repository redistributes third-party font software, unmodified, under its
own licences — not under EUPL-1.2. Liberation Sans and Libertinus Serif are
covered by the SIL Open Font License 1.1; Liberation Serif and Liberation Mono
are covered by the Liberation Fonts License (GPLv2 with an embedding
exception). Per-family attribution, versions and licence texts are in
[assets/fonts/NOTICE.md](assets/fonts/NOTICE.md).

The embedding exception means PDFs produced with these fonts are not made
subject to the GPL by the embedding itself.
