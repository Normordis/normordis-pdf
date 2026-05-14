# normordis-pdf — Developer Guide

This repository is bilingual. The authoritative programming reference is `MANUAL.md` (Portuguese). This file covers the most commonly asked topics in English.

## Quick start

```toml
# Cargo.toml
[dependencies]
normordis-pdf = "2.5.0"
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

## Fonts

### Embedded families

Four font families ship pre-compiled into the crate binary — no system fonts or external files needed:

| Name | Word equivalent | Typical use |
|---|---|---|
| `LiberationSans` | Arial / Calibri / Helvetica | Body text (default) |
| `LiberationSerif` | Times New Roman / Cambria | Formal serif body |
| `LiberationMono` | Courier New / Consolas | Code, references |
| `LibertinusSerif` | — | Alternative serif body |

Common Word font names are pre-registered as aliases:  
`Arial`, `Calibri`, `Helvetica` → `LiberationSans`  
`Times New Roman`, `Cambria`, `Georgia` → `LiberationSerif`  
`Courier New`, `Consolas` → `LiberationMono`

### Loading custom fonts

Register any TTF/OTF font via the `DocumentBuilder` fluent API:

```rust
use normordis_pdf::{DocumentBuilder, Paragraph};

// From bytes (e.g. include_bytes! for bundled fonts)
let pdf = DocumentBuilder::new("Document")
    .font_from_bytes(
        "GilSans",
        include_bytes!("assets/GilSans-Regular.ttf"),
        Some(include_bytes!("assets/GilSans-Bold.ttf")),
        None, None,
    )?
    .push(Paragraph::new("Text in GilSans.").font_family("GilSans"))
    .render_to_bytes()?;

// From files on disk
let pdf = DocumentBuilder::new("Document")
    .font_from_file(
        "FiraCode",
        "assets/FiraCode-Regular.ttf",
        None::<&str>, None::<&str>, None::<&str>,
    )?
    .render_to_bytes()?;

// Scan a whole directory
let pdf = DocumentBuilder::new("Document")
    .fonts_from_dir("assets/fonts/")?
    .render_to_bytes()?;

// Change the default font
let pdf = DocumentBuilder::new("Document")
    .default_font("LiberationSerif")?
    .render_to_bytes()?;
```

### Per-paragraph font override

```rust
use normordis_pdf::Paragraph;

Paragraph::new("Serif paragraph.").font_family("LiberationSerif")
Paragraph::new("Mono paragraph.").font_family("LiberationMono")
// Unknown name → fallback chain → warning on stderr, never an error
Paragraph::new("Fallback paragraph.").font_family("UnknownFont")
```

### `FontRegistry` direct manipulation

```rust
use normordis_pdf::FontRegistry;

let mut reg = FontRegistry::default();

reg.register_bytes("Crimson", include_bytes!("Crimson-Regular.ttf"), None, None, None)?;
reg.register_file("Montserrat", "Montserrat-Regular.ttf", None::<&str>, None::<&str>, None::<&str>)?;
reg.register_single("Icons", "icons.ttf")?;
let n = reg.load_dir("assets/fonts/")?;   // groups by -Bold / -Italic suffix
reg.add_alias("Helvetica Neue", "Montserrat");
```

### Font fallback chain

When a requested font is not registered, the engine tries the fallback chain in order. Defaults to `["LiberationSans", "LiberationSerif", "LiberationMono"]`.

```rust
use normordis_pdf::{DocumentStyle, FontFallbackChain};

let mut style = DocumentStyle::default();
style.font_fallback = FontFallbackChain::new(vec!["Crimson", "LiberationSans"]);
```

### System fonts (optional feature)

```toml
normordis-pdf = { version = "...", features = ["system-fonts"] }
```

```rust
#[cfg(feature = "system-fonts")]
let reg = normordis_pdf::FontRegistry::from_system()?;
```

## Project structure

- `Cargo.toml` — crate manifest and workspace definition
- `src/` — core library implementation
- `examples/` — runnable examples (`cargo run --example <name>`)
- `tools/` — helper CLIs: `dotx2ndt`, `ndt-tools`
- `MANUAL.md` — full developer guide (Portuguese)
- `README.md` — bilingual project introduction and quick start
- `CHANGELOG.md` — version history
