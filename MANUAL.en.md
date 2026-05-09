# normordis-pdf — Developer Guide

This repository is bilingual, with the main programming guide currently available in Portuguese in `MANUAL.md`.

## English guide status

A complete English version of the manual is under construction. For now, please use the Portuguese manual at `MANUAL.md` and refer to `README.md` for bilingual project documentation.

## Quick start

```toml
# Cargo.toml
[dependencies]
normordis-pdf = "1.0.0"
```

```rust
use normordis_pdf::{DocumentBuilder, Paragraph, Section, Spacer, TextAlign};

let pdf = DocumentBuilder::new("Monthly Report")
    .push(Section::new("1. Introduction", 1))
    .push(Paragraph::new("This report describes…").align(TextAlign::Justify))
    .push(Spacer::new(6.0))
    .render_to_bytes()?;

std::fs::write("output.pdf", pdf)?;
```

## Project structure

- `Cargo.toml` — crate manifest and workspace definition
- `src/` — core library implementation
- `examples/` — sample renderers using the crate API
- `tools/` — helper CLIs such as `dotx2ndt` and `ndt-tools`
- `MANUAL.md` — detailed developer guide in Portuguese
- `README.md` — bilingual project introduction and quick start
