# normordis-pdf — Developer Guide

This repository is bilingual. The authoritative programming reference is `MANUAL.md` (Portuguese). This file covers the most commonly asked topics in English.

## Quick start

```toml
# Cargo.toml
[dependencies]
normordis-pdf = "3.0.0"
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

## Implementing a custom element

Any struct that implements `Element` can be added via `builder.push()`.

Drawing calls go through `ctx.backend` (the `PdfBackend` trait). **There is no `ctx.ops` or direct printpdf integration.** `render()` returns `RenderResult`, not `()`.

```rust
use normordis_pdf::{Element, LayoutMode, RenderContext, elements::RenderResult};
use normordis_pdf::styles::RgbColor;

struct ColorBanner {
    height_mm: f64,
    color: RgbColor,
}

impl Element for ColorBanner {
    fn estimated_height_mm(&self) -> f64 {
        self.height_mm
    }

    fn render(&self, ctx: &mut RenderContext) -> normordis_pdf::Result<RenderResult> {
        let x = ctx.layout.content_x_mm;
        let y = ctx.flow.cursor_y_mm - self.height_mm;
        let w = ctx.layout.content_width_mm;

        // Draw via ctx.backend — never push ops directly:
        ctx.backend.draw_rect(x, y, w, self.height_mm, &self.color)?;

        // Advance the flow cursor (required for flow elements):
        ctx.flow.advance(self.height_mm);

        Ok(RenderResult::done())
    }
}
```

For a fixed element, override `layout_mode()` and **do not call** `ctx.flow.advance()`:

```rust
use normordis_pdf::FixedBox;

fn layout_mode(&self) -> LayoutMode {
    LayoutMode::Fixed(FixedBox {
        x_mm: 10.0, y_mm: 50.0,
        width_mm: 80.0, height_mm: 20.0,
        ..Default::default()
    })
}
```

### Key `ctx.backend` methods

| Method | Description |
|---|---|
| `draw_rect(x, y, w, h, fill)` | Filled rectangle |
| `draw_rect_stroked(x, y, w, h, fill, stroke, pt)` | Rectangle with border |
| `draw_line(x1, y1, x2, y2, width_pt, color)` | Line segment |
| `draw_text(text, x, y, size_pt, font_ref, color, spacing)` | Text at absolute position |
| `set_opacity(0.0–1.0)` | Opacity via ExtGState |
| `save_state()` / `restore_state()` | Graphics state stack |

Convenience wrappers directly on `ctx`:

| Method | Description |
|---|---|
| `ctx.draw_hline(x0, x1, y, width_pt, color)` | Horizontal line |
| `ctx.draw_vline(x, y0, y1, width_pt, color)` | Vertical line |
| `ctx.draw_text(text, x, y, size_pt, font_ref, color, spacing)` | Text |

## Project structure

- `Cargo.toml` — crate manifest and workspace definition
- `src/` — core library implementation
- `examples/` — runnable examples (`cargo run --example <name>`)
- `tools/` — helper CLIs: `dotx2ndt`, `ndt-tools`
- `MANUAL.md` — full developer guide (Portuguese)
- `README.md` — bilingual project introduction and quick start
- `CHANGELOG.md` — version history
