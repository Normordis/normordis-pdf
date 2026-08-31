use base64::Engine as _;
use serde_json::Value;

use super::{
    TemplateError,
    data::NdtData,
    model::{NdtDocument, legacy_body::BodyElement},
    resolver,
};
use crate::{
    elements::{
        Element,
        fixed_image::{FixedImageBox, ImageFit},
        fixed_line::FixedLineElement as FixedLine,
        fixed_text::{FixedTextBox, VerticalAlign},
        image::{ImageAlignment, ImageElement},
        list::{BulletList, ListItemElement, OrderedList},
        page_break::PageBreakElement,
        paragraph::{Paragraph, ParagraphContent, TextRun},
        section::Section,
        spacer::Spacer,
        table::Table,
    },
    layout::{BorderStyle, BoxBorder, FixedBox, OverflowPolicy, TextAlign},
    richtext::{self},
    styles::{DocumentStyle, RgbColor},
};

/// Resolve a template src string to raw image bytes.
///
/// Handles `{{placeholder}}` expansion followed by data-URL decoding:
/// `data:image/<fmt>;base64,<base64>` → decoded bytes.
fn resolve_image_src(src: &str, data: &NdtData) -> Vec<u8> {
    let resolved = resolver::resolve_string(src, data);
    let s = resolved.trim();
    if let Some(rest) = s.strip_prefix("data:image/")
        && let Some((_, b64)) = rest.split_once(";base64,")
    {
        return base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .unwrap_or_default();
    }
    Vec::new()
}

/// NDT 2.0.0 render entry point (positioned-layout renderer not yet implemented).
pub fn render_template(
    _doc: &NdtDocument,
    _data: &NdtData,
    _style: &DocumentStyle,
) -> Result<Vec<Box<dyn Element>>, TemplateError> {
    Err(TemplateError::RenderError(
        "NDT 2.0.0 positioned-layout renderer not yet implemented".into(),
    ))
}

/// Render a legacy body element list (used by the render archive pipeline internally).
pub(crate) fn render_body_elements(
    body: &[BodyElement],
    data: &NdtData,
    style: &DocumentStyle,
) -> Result<Vec<Box<dyn Element>>, TemplateError> {
    let mut elements: Vec<Box<dyn Element>> = Vec::new();

    for item in body {
        match item {
            BodyElement::Paragraph(p) => {
                let text = resolver::resolve_string(&p.text, data);
                let alignment = parse_alignment(p.alignment.as_deref());
                let mut para = Paragraph::new(text).align(alignment);
                if p.bold.unwrap_or(false) {
                    para = para.bold();
                }
                if p.italic.unwrap_or(false) {
                    para = para.italic();
                }
                if let Some(fs) = p.font_size {
                    para = para.font_size(fs);
                }
                if let Some(indent) = p.indent_mm {
                    para.indent_left_mm = indent;
                }
                elements.push(Box::new(para));
            }

            BodyElement::Heading(h) => {
                let text = resolver::resolve_string(&h.text, data);
                let level = h.level.unwrap_or(1).clamp(1, 3);
                elements.push(Box::new(Section::new(text, level)));
            }

            BodyElement::RichText(rt) => {
                let json = if rt.source.as_deref() == Some("placeholder") {
                    match resolver::resolve_value(&rt.content, data) {
                        Some(Value::String(s)) => s,
                        _ => resolver::resolve_string(&rt.content, data),
                    }
                } else {
                    resolver::resolve_string(&rt.content, data)
                };

                let ncrtf_doc = richtext::parse_ncrtf(&json)
                    .map_err(|e| TemplateError::RenderError(e.to_string()))?;
                let mut els = richtext::ncrtf_to_elements(&ncrtf_doc, style);
                elements.append(&mut els);
            }

            BodyElement::Table(t) => {
                let headers: Vec<String> = t
                    .headers
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .map(|h| resolver::resolve_string(h, data))
                    .collect();
                let rows: Vec<crate::elements::table::TableRow> = t
                    .rows
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .map(|row| {
                        let cells: Vec<String> = row
                            .iter()
                            .map(|c| resolver::resolve_string(c, data))
                            .collect();
                        crate::elements::table::TableRow::plain(cells)
                    })
                    .collect();
                let mut table = Table::new(headers, rows);
                if let Some(widths) = &t.col_widths {
                    table = table.col_widths(widths.clone());
                }
                elements.push(Box::new(table));
            }

            BodyElement::List(l) => {
                let list_type = l.list_type.as_deref().unwrap_or("bullet");
                let items: Vec<ListItemElement> = l
                    .items
                    .iter()
                    .map(|text| ListItemElement {
                        indent: 0,
                        runs: vec![TextRun::plain(resolver::resolve_string(text, data))],
                    })
                    .collect();
                match list_type {
                    "ordered" => elements.push(Box::new(OrderedList { start: 1, items })),
                    _ => elements.push(Box::new(BulletList { items })),
                }
            }

            BodyElement::Image(img) => {
                let bytes = resolve_image_src(&img.src, data);
                let mut el = ImageElement::new(bytes);
                if let Some(pct) = img.width_percent {
                    el.width_percent = Some(pct);
                }
                if let Some(ref cap) = img.caption {
                    el = el.caption(resolver::resolve_string(cap, data));
                }
                el.alignment = match img.alignment.as_deref() {
                    Some("left") => ImageAlignment::Left,
                    Some("right") => ImageAlignment::Right,
                    _ => ImageAlignment::Center,
                };
                elements.push(Box::new(el));
            }

            BodyElement::Spacer(s) => {
                elements.push(Box::new(Spacer::new(s.height_mm)));
            }

            BodyElement::HorizontalRule => {
                elements.push(Box::new(Spacer::new(2.0)));
            }

            BodyElement::PageBreak => {
                elements.push(Box::new(PageBreakElement));
            }

            BodyElement::FixedText(ft) => {
                let text = resolver::resolve_string(&ft.text, data);
                let overflow = parse_overflow(ft.overflow.as_deref());
                let alignment = parse_alignment(ft.alignment.as_deref());
                elements.push(Box::new(FixedTextBox {
                    text_box: FixedBox {
                        x_mm: ft.x_mm,
                        y_mm: ft.y_mm,
                        width_mm: ft.width_mm,
                        height_mm: ft.height_mm,
                        overflow,
                        border: None,
                        background: None,
                        padding_mm: ft.padding_mm.unwrap_or(2.0),
                        z_index: 0,
                        ua_role: None,
                        ua_alt: None,
                    },
                    content: ParagraphContent::Plain(text),
                    alignment,
                    font_size: ft.font_size,
                    vertical_align: VerticalAlign::Top,
                }));
            }

            BodyElement::FixedImage(fi) => {
                let bytes = resolve_image_src(&fi.src, data);
                let fit = match fi.fit.as_deref() {
                    Some("cover") => ImageFit::Cover,
                    Some("stretch") => ImageFit::Stretch,
                    Some("original") => ImageFit::Original,
                    _ => ImageFit::Contain,
                };
                elements.push(Box::new(FixedImageBox {
                    image_box: FixedBox {
                        x_mm: fi.x_mm,
                        y_mm: fi.y_mm,
                        width_mm: fi.width_mm,
                        height_mm: fi.height_mm,
                        overflow: OverflowPolicy::Truncate,
                        border: None,
                        background: None,
                        padding_mm: 0.0,
                        z_index: 0,
                        ua_role: None,
                        ua_alt: None,
                    },
                    data: bytes,
                    fit,
                }));
            }

            BodyElement::FixedLine(fl) => {
                let color = fl
                    .color
                    .as_deref()
                    .and_then(RgbColor::from_hex)
                    .unwrap_or_else(|| RgbColor::new(0.0, 0.0, 0.0));
                elements.push(Box::new(FixedLine::new(
                    fl.x1_mm, fl.y1_mm, fl.x2_mm, fl.y2_mm, color,
                )));
            }

            BodyElement::FixedBox(fb) => {
                let text = resolver::resolve_string(fb.content.as_deref().unwrap_or(""), data);
                let overflow = parse_overflow(fb.overflow.as_deref());
                let alignment = parse_alignment(fb.alignment.as_deref());
                let border = fb
                    .border_color
                    .as_deref()
                    .and_then(RgbColor::from_hex)
                    .map(|c| BoxBorder {
                        width_mm: fb.border_width_mm.unwrap_or(0.3),
                        color: c,
                        style: BorderStyle::Solid,
                    });
                let background = fb.background.as_deref().and_then(RgbColor::from_hex);
                elements.push(Box::new(FixedTextBox {
                    text_box: FixedBox {
                        x_mm: fb.x_mm,
                        y_mm: fb.y_mm,
                        width_mm: fb.width_mm,
                        height_mm: fb.height_mm,
                        overflow,
                        border,
                        background,
                        padding_mm: fb.padding_mm.unwrap_or(2.0),
                        z_index: 0,
                        ua_role: None,
                        ua_alt: None,
                    },
                    content: ParagraphContent::Plain(text),
                    alignment,
                    font_size: None,
                    vertical_align: VerticalAlign::Top,
                }));
            }

            BodyElement::FootnoteRef(fref) => {
                use crate::elements::footnote::{FootnoteMarkStyle, FootnoteRef};
                let style = match fref.mark_style.as_deref() {
                    Some("alpha") => FootnoteMarkStyle::Alpha,
                    Some("symbol") => FootnoteMarkStyle::Symbol,
                    _ => FootnoteMarkStyle::Numeric,
                };
                elements.push(Box::new(FootnoteRef::new(fref.number).with_style(style)));
            }

            BodyElement::Toc(toc_el) => {
                use crate::elements::toc::TableOfContents;
                let mut toc = TableOfContents::new();
                if let Some(ref t) = toc_el.title {
                    toc = toc.title(t.clone());
                }
                if let Some(lvl) = toc_el.max_level {
                    toc = toc.max_level(lvl);
                }
                if let Some(ref lc) = toc_el.leader_char
                    && let Some(c) = lc.chars().next()
                {
                    toc = toc.dot_leader(c);
                }
                elements.push(Box::new(toc));
            }

            BodyElement::AcroformField(af) => {
                use crate::elements::form::{
                    CheckBoxDef, ComboBoxDef, FieldRect, FormField, TextFieldDef,
                };
                let rect = FieldRect {
                    x_mm: af.rect.x_mm,
                    y_mm: af.rect.y_mm,
                    width_mm: af.rect.width_mm,
                    height_mm: af.rect.height_mm,
                };
                let field = match af.field_type.as_str() {
                    "check_box" => FormField::CheckBox(CheckBoxDef {
                        name: af.name.clone(),
                        checked_by_default: af.checked_by_default.unwrap_or(false),
                        tooltip: af.tooltip.clone(),
                        rect,
                    }),
                    "combo_box" => FormField::ComboBox(ComboBoxDef {
                        name: af.name.clone(),
                        options: af.options.clone().unwrap_or_default(),
                        default_value: None,
                        editable: false,
                        tooltip: af.tooltip.clone(),
                        rect,
                        font_size: af.font_size.unwrap_or(11.0),
                    }),
                    _ => FormField::TextField(TextFieldDef {
                        name: af.name.clone(),
                        default_value: None,
                        tooltip: af.tooltip.clone(),
                        multiline: false,
                        max_length: af.max_length,
                        readonly: false,
                        required: af.required.unwrap_or(false),
                        rect,
                        font_size: af.font_size.unwrap_or(11.0),
                    }),
                };
                elements.push(Box::new(field));
            }
        }
    }

    Ok(elements)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_alignment(s: Option<&str>) -> TextAlign {
    match s {
        Some("center") => TextAlign::Center,
        Some("justify") => TextAlign::Justify,
        _ => TextAlign::Left,
    }
}

fn parse_overflow(s: Option<&str>) -> OverflowPolicy {
    match s {
        Some("clip") => OverflowPolicy::Clip,
        Some("shrink") => OverflowPolicy::Shrink,
        Some("overflow") => OverflowPolicy::Overflow,
        _ => OverflowPolicy::Truncate,
    }
}
