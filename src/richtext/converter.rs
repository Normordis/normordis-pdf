use crate::{
    elements::{
        Element,
        image::ImageElement,
        list::{BulletList, CheckList, CheckListItem, ListItemElement, OrderedList},
        paragraph::{Paragraph, TextRun},
        section::Section,
        table::{Table, TableCell, TableStyle},
    },
    layout::TextAlign,
    richtext::{
        marks::{AppliedStyle, MarkValue},
        model::{
            Block, HeadingBlock, Inline, ListItemContent, ListType, NcrtfDocument, ParagraphBlock,
            TableBlock, TextNode,
        },
    },
    styles::{DocumentStyle, RgbColor},
};

const LINK_COLOR: RgbColor = RgbColor { r: 0.0, g: 0.2, b: 0.6 };
const BLOCKQUOTE_COLOR: RgbColor = RgbColor { r: 0.47, g: 0.47, b: 0.47 };

/// Convert a parsed `NcrtfDocument` into a flat list of `normordis-pdf` elements.
pub fn ncrtf_to_elements(doc: &NcrtfDocument, _style: &DocumentStyle) -> Vec<Box<dyn Element>> {
    let mut elements: Vec<Box<dyn Element>> = Vec::new();

    for block in &doc.content {
        match block {
            Block::Heading(h) => elements.push(heading_to_section(h)),
            Block::Paragraph(p) => elements.push(paragraph_block_to_element(p)),
            Block::List(l) => match l.list_type {
                ListType::Bullet => {
                    let items = l.content.iter().map(|li| ListItemElement {
                        indent: 0,
                        runs: list_item_runs(&li.content),
                    }).collect();
                    elements.push(Box::new(BulletList { items }));
                }
                ListType::Ordered => {
                    let items = l.content.iter().map(|li| ListItemElement {
                        indent: 0,
                        runs: list_item_runs(&li.content),
                    }).collect();
                    elements.push(Box::new(OrderedList { start: 1, items }));
                }
                ListType::Checklist => {
                    let items = l.content.iter().map(|li| CheckListItem {
                        checked: li.checked.unwrap_or(false),
                        indent: 0,
                        runs: list_item_runs(&li.content),
                    }).collect();
                    elements.push(Box::new(CheckList { items }));
                }
            },
            Block::Table(t) => table_block_to_elements(t, &mut elements),
            Block::Blockquote(bq) => {
                let runs = inlines_to_runs_colored(&bq.content, &BLOCKQUOTE_COLOR);
                let mut p = Paragraph::from_runs(runs, TextAlign::Left, None);
                p.indent_left_mm = 8.0;
                p.indent_right_mm = 8.0;
                elements.push(Box::new(p));
            }
            Block::Image(img) => {
                // NCRTF v2.0.0 images reference assets by key, not data URIs.
                // Without a resolved asset registry, emit an empty placeholder.
                let mut element = ImageElement::new(Vec::new()).alt(img.alt.clone());
                if let Some(cap) = &img.caption {
                    element = element.caption(cap.clone());
                }
                if let Some(pct) = img.width_percent {
                    element.width_percent = Some(pct as f64);
                }
                elements.push(Box::new(element));
            }
        }
    }

    elements
}

// ── Block helpers ─────────────────────────────────────────────────────────────

fn heading_to_section(h: &HeadingBlock) -> Box<dyn Element> {
    let text = inlines_to_text(&h.content);
    let level = h.level.clamp(1, 3);
    Box::new(Section::new(text, level))
}

fn paragraph_block_to_element(p: &ParagraphBlock) -> Box<dyn Element> {
    let runs = inlines_to_runs(&p.content);
    let alignment = convert_alignment(p.alignment.as_ref());
    let mut para = Paragraph::from_runs(runs, alignment, None);
    if let Some(level) = p.indent {
        para.indent_left_mm = level as f64 * 10.0;
    }
    Box::new(para)
}

fn table_block_to_elements(t: &TableBlock, out: &mut Vec<Box<dyn Element>>) {
    let mut builder = Table::builder().table_style(TableStyle::grid());

    for hrow in &t.head {
        let cells: Vec<TableCell> = hrow.cells.iter().map(|s| {
            let mut c = TableCell::new(s.clone());
            c.style_ref = Some("table_header".into());
            c
        }).collect();
        builder = builder.header_row(cells);
    }

    for brow in &t.body {
        let cells: Vec<TableCell> = brow.cells.iter()
            .map(|s| TableCell::new(s.clone()))
            .collect();
        builder = builder.row(cells);
    }

    out.push(Box::new(builder.build()));
}

// ── Inline helpers ────────────────────────────────────────────────────────────

/// Extract plain text from a slice of inline nodes.
pub fn inlines_to_text(inlines: &[Inline]) -> String {
    inlines.iter().map(|i| match i {
        Inline::Text(t) => t.text.clone(),
        Inline::Link(l) => l.content.iter().map(|t| t.text.clone()).collect(),
        Inline::HardBreak => "\n".to_string(),
    }).collect()
}

/// Convert a slice of inline nodes into `TextRun`s.
pub fn inlines_to_runs(inlines: &[Inline]) -> Vec<TextRun> {
    inlines_to_runs_colored(inlines, &RgbColor::new(0.0, 0.0, 0.0))
}

/// Convert inline nodes to `TextRun`s with an optional base color override.
fn inlines_to_runs_colored(inlines: &[Inline], base_color: &RgbColor) -> Vec<TextRun> {
    let is_black = base_color.r == 0.0 && base_color.g == 0.0 && base_color.b == 0.0;
    let color_hex = if is_black {
        None
    } else {
        Some(format!(
            "#{:02X}{:02X}{:02X}",
            (base_color.r * 255.0) as u8,
            (base_color.g * 255.0) as u8,
            (base_color.b * 255.0) as u8,
        ))
    };

    let mut runs = Vec::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => runs.push(text_node_to_run(t, color_hex.as_deref())),
            Inline::Link(l) => {
                for t in &l.content {
                    let mut run = text_node_to_run(t, None);
                    run.style.underline = true;
                    if run.style.color.is_none() {
                        run.style.color = Some(format!(
                            "#{:02X}{:02X}{:02X}",
                            (LINK_COLOR.r * 255.0) as u8,
                            (LINK_COLOR.g * 255.0) as u8,
                            (LINK_COLOR.b * 255.0) as u8,
                        ));
                    }
                    runs.push(run);
                }
            }
            Inline::HardBreak => runs.push(TextRun { text: "\n".into(), ..Default::default() }),
        }
    }
    runs
}

fn text_node_to_run(t: &TextNode, base_color: Option<&str>) -> TextRun {
    let marks: &[MarkValue] = t.marks.as_deref().unwrap_or(&[]);
    let mut style = AppliedStyle::from(marks);
    if style.color.is_none() {
        style.color = base_color.map(|s| s.to_owned());
    }
    if style.font_family.is_none() {
        style.font_family = t.font_family.clone();
    }
    TextRun { text: t.text.clone(), style, ..Default::default() }
}

/// Extract `TextRun`s from list item content, skipping nested lists.
fn list_item_runs(content: &[ListItemContent]) -> Vec<TextRun> {
    let mut runs = Vec::new();
    for item in content {
        match item {
            ListItemContent::Text(t) => runs.push(text_node_to_run(t, None)),
            ListItemContent::Link(l) => {
                for t in &l.content {
                    let mut run = text_node_to_run(t, None);
                    run.style.underline = true;
                    runs.push(run);
                }
            }
            ListItemContent::HardBreak => {
                runs.push(TextRun { text: "\n".into(), ..Default::default() });
            }
            ListItemContent::List(_) => {} // nested lists not yet rendered in body-flow path
        }
    }
    runs
}

fn convert_alignment(a: Option<&TextAlign>) -> TextAlign {
    match a {
        Some(align) => *align,
        None => TextAlign::Left,
    }
}
