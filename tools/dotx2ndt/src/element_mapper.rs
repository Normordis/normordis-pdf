use std::collections::HashMap;

use base64::Engine as _;
use serde_json::{json, Value};

use crate::error::Dotx2NdtError;
use crate::numbering_mapper::ListType;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_NS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

// ── Public types ──────────────────────────────────────────────────────────────

/// Controls whether document text is preserved or replaced with `{{placeholders}}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    #[default]
    Document,
    Template,
}

/// Context shared across the entire body-mapping pass.
pub struct MappingContext {
    /// Word `styleId` → NDT slug (e.g. `"Heading1"` → `"heading_1"`).
    pub style_id_map: HashMap<String, String>,
    /// `numId` → list type, from `word/numbering.xml`.
    pub numbering_types: HashMap<u32, ListType>,
    /// Media bytes keyed by path relative to `word/` (e.g. `"media/image1.png"`).
    pub media: HashMap<String, Vec<u8>>,
    /// rId → target path relative to `word/`.
    pub relationships: HashMap<String, String>,
    pub mode: OutputMode,
    /// Accumulated placeholder definitions `(key, description)` for template mode.
    pub placeholders: Vec<(String, String)>,
    counter: u32,
}

impl Default for MappingContext {
    fn default() -> Self {
        Self {
            style_id_map: HashMap::new(),
            numbering_types: HashMap::new(),
            media: HashMap::new(),
            relationships: HashMap::new(),
            mode: OutputMode::Document,
            placeholders: Vec::new(),
            counter: 0,
        }
    }
}

impl MappingContext {
    fn next_key(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{}_{}", prefix, self.counter)
    }

    fn placeholder(&mut self, prefix: &str, description: &str) -> String {
        let key = self.next_key(prefix);
        self.placeholders.push((key.clone(), description.to_string()));
        format!("{{{{{}}}}}", key)
    }
}

// ── Intermediate types ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct RawPara {
    style_id: Option<String>,
    alignment: Option<String>,
    num_id: Option<u32>,
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
enum Segment {
    Text { text: String, bold: bool, italic: bool, underline: bool, strike: bool },
    Image { r_id: String },
    LineBreak,
}

#[derive(Debug, Clone)]
enum RawItem {
    Para(RawPara),
    Table(Vec<Vec<String>>),
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Parses a Word `word/document.xml` body into NDT body element JSON values.
///
/// Handles: headings, paragraphs (with style and inline formatting), lists,
/// tables, and embedded images.  Dummy anchor paragraphs between consecutive
/// tables are suppressed automatically.
pub fn map_body_from_xml(
    document_xml: &str,
    ctx: &mut MappingContext,
) -> Result<Vec<Value>, Dotx2NdtError> {
    let doc = roxmltree::Document::parse(document_xml)
        .map_err(|e| Dotx2NdtError::Xml(format!("{e}")))?;

    let body = doc
        .descendants()
        .find(|n| n.tag_name().name() == "body")
        .ok_or_else(|| Dotx2NdtError::Xml("missing w:body".into()))?;

    let children: Vec<roxmltree::Node> =
        body.children().filter(|n| n.is_element()).collect();

    // First pass: parse into raw items (with dummy-paragraph suppression)
    let mut raw_items: Vec<RawItem> = Vec::new();
    for (i, node) in children.iter().enumerate() {
        let prev_table = i > 0 && children[i - 1].tag_name().name() == "tbl";
        let next_table =
            i + 1 < children.len() && children[i + 1].tag_name().name() == "tbl";

        if is_dummy_paragraph(node, prev_table, next_table) {
            continue;
        }

        match node.tag_name().name() {
            "p" => raw_items.push(RawItem::Para(parse_paragraph(node))),
            "tbl" => raw_items.push(RawItem::Table(parse_table(node))),
            _ => {}
        }
    }

    // Second pass: group list paragraphs, render to Value
    let mut result = Vec::new();
    let mut i = 0;
    while i < raw_items.len() {
        match &raw_items[i] {
            RawItem::Para(p) if p.num_id.is_some() => {
                let list_num_id = p.num_id.unwrap();
                let mut list_paras: Vec<RawPara> = Vec::new();
                while i < raw_items.len() {
                    if let RawItem::Para(p2) = &raw_items[i] {
                        if p2.num_id == Some(list_num_id) {
                            list_paras.push(p2.clone());
                            i += 1;
                            continue;
                        }
                    }
                    break;
                }
                let lt = ctx
                    .numbering_types
                    .get(&list_num_id)
                    .copied()
                    .unwrap_or(ListType::Bullet);
                result.push(render_list(lt, list_paras, ctx));
            }
            RawItem::Para(_) => {
                // Clone to avoid borrowing raw_items while calling ctx
                if let RawItem::Para(p) = raw_items[i].clone() {
                    if let Some(v) = render_para(p, ctx) {
                        result.push(v);
                    }
                }
                i += 1;
            }
            RawItem::Table(_) => {
                if let RawItem::Table(rows) = raw_items[i].clone() {
                    result.push(render_table(rows, ctx));
                }
                i += 1;
            }
        }
    }

    Ok(result)
}

// ── Dummy-paragraph suppression (preserved from v0.2.0) ──────────────────────

/// Returns `true` if `node` is an empty anchor paragraph between two tables.
pub(crate) fn is_dummy_paragraph(
    node: &roxmltree::Node,
    prev_is_table: bool,
    next_is_table: bool,
) -> bool {
    if node.tag_name().name() != "p" {
        return false;
    }
    if !prev_is_table || !next_is_table {
        return false;
    }
    let has_visible = node
        .descendants()
        .filter(|n| n.tag_name().name() == "t")
        .any(|n| !n.text().unwrap_or("").trim().is_empty());
    !has_visible
}

// ── Parsing ───────────────────────────────────────────────────────────────────

fn parse_paragraph(node: &roxmltree::Node) -> RawPara {
    let mut style_id: Option<String> = None;
    let mut alignment: Option<String> = None;
    let mut num_id: Option<u32> = None;
    let mut segments: Vec<Segment> = Vec::new();

    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        match child.tag_name().name() {
            "pPr" => {
                for ppr in child.children() {
                    if !ppr.is_element() {
                        continue;
                    }
                    match ppr.tag_name().name() {
                        "pStyle" => {
                            style_id = ppr.attribute((W_NS, "val")).map(|s| s.to_string());
                        }
                        "numPr" => {
                            let mut nid: Option<u32> = None;
                            for np in ppr.children() {
                                match np.tag_name().name() {
                                    "numId" => {
                                        nid = np
                                            .attribute((W_NS, "val"))
                                            .and_then(|v| v.parse().ok());
                                    }
                                    _ => {}
                                }
                            }
                            if let Some(n) = nid {
                                if n != 0 {
                                    num_id = Some(n);
                                }
                            }
                        }
                        "jc" => {
                            alignment = ppr
                                .attribute((W_NS, "val"))
                                .map(|v| map_jc(v).to_string());
                        }
                        _ => {}
                    }
                }
            }
            "r" => {
                parse_run(&child, &mut segments);
            }
            "hyperlink" => {
                // Collect runs inside hyperlinks as plain text (no href in NDT body)
                for hchild in child.children() {
                    if hchild.is_element() && hchild.tag_name().name() == "r" {
                        parse_run(&hchild, &mut segments);
                    }
                }
            }
            _ => {}
        }
    }

    RawPara { style_id, alignment, num_id, segments }
}

fn parse_run(node: &roxmltree::Node, segments: &mut Vec<Segment>) {
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut strike = false;

    // Parse run properties
    if let Some(rpr) = node.children().find(|n| n.tag_name().name() == "rPr") {
        for prop in rpr.children() {
            if !prop.is_element() {
                continue;
            }
            match prop.tag_name().name() {
                "b" => {
                    bold = prop.attribute((W_NS, "val")).map_or(true, |v| v != "false" && v != "0");
                }
                "i" => {
                    italic =
                        prop.attribute((W_NS, "val")).map_or(true, |v| v != "false" && v != "0");
                }
                "u" => {
                    let val = prop.attribute((W_NS, "val")).unwrap_or("single");
                    underline = val != "none";
                }
                "strike" => {
                    strike =
                        prop.attribute((W_NS, "val")).map_or(true, |v| v != "false" && v != "0");
                }
                _ => {}
            }
        }
    }

    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        match child.tag_name().name() {
            "t" => {
                let text = child.text().unwrap_or("").to_string();
                if !text.is_empty() {
                    segments.push(Segment::Text { text, bold, italic, underline, strike });
                }
            }
            "br" => {
                segments.push(Segment::LineBreak);
            }
            "drawing" => {
                if let Some(r_id) = find_blip_embed(&child) {
                    segments.push(Segment::Image { r_id });
                }
            }
            _ => {}
        }
    }
}

/// Searches a `<w:drawing>` subtree for `<a:blip r:embed="...">` and returns the rId.
fn find_blip_embed(drawing: &roxmltree::Node) -> Option<String> {
    for node in drawing.descendants() {
        if node.tag_name().name() == "blip" {
            if let Some(id) = node.attribute((R_NS, "embed")) {
                return Some(id.to_string());
            }
        }
    }
    None
}

fn parse_table(node: &roxmltree::Node) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for child in node.children() {
        if !child.is_element() || child.tag_name().name() != "tr" {
            continue;
        }
        let mut row: Vec<String> = Vec::new();
        for cell in child.children() {
            if !cell.is_element() || cell.tag_name().name() != "tc" {
                continue;
            }
            let cell_text: String = cell
                .descendants()
                .filter(|n| n.tag_name().name() == "t")
                .filter_map(|n| n.text())
                .collect();
            row.push(cell_text);
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }
    rows
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render_para(raw: RawPara, ctx: &mut MappingContext) -> Option<Value> {
    // Check if paragraph is only an image
    let image_rids: Vec<String> = raw
        .segments
        .iter()
        .filter_map(|s| if let Segment::Image { r_id } = s { Some(r_id.clone()) } else { None })
        .collect();
    let has_text = raw.segments.iter().any(|s| matches!(s, Segment::Text { text, .. } if !text.is_empty()));

    if !image_rids.is_empty() && !has_text {
        // Image-only paragraph
        return Some(render_image(&image_rids[0], ctx));
    }

    // Heading detection
    if let Some(sid) = &raw.style_id {
        if let Some(level) = heading_level(sid) {
            return Some(render_heading(&raw.segments, level, &raw.alignment, ctx));
        }
    }

    // Plain or rich paragraph
    Some(render_paragraph_value(raw, ctx))
}

fn render_heading(
    segments: &[Segment],
    level: u8,
    alignment: &Option<String>,
    ctx: &mut MappingContext,
) -> Value {
    let text = if ctx.mode == OutputMode::Template {
        ctx.placeholder(&format!("heading{level}"), &format!("Heading level {level} text"))
    } else {
        segments_to_plain_text(segments)
    };
    let mut v = json!({"type": "heading", "level": level, "text": text});
    if let Some(align) = alignment {
        if align != "left" {
            v["alignment"] = json!(align);
        }
    }
    v
}

fn render_paragraph_value(raw: RawPara, ctx: &mut MappingContext) -> Value {
    let style_ref = raw
        .style_id
        .as_deref()
        .and_then(|id| ctx.style_id_map.get(id))
        .cloned();

    if ctx.mode == OutputMode::Template {
        let key = ctx.placeholder("paragraph", "Paragraph text");
        let mut v = json!({"type": "paragraph", "text": key});
        if let Some(sr) = style_ref {
            v["style_ref"] = json!(sr);
        }
        if let Some(align) = &raw.alignment {
            v["alignment"] = json!(align);
        }
        return v;
    }

    let has_formatting = raw.segments.iter().any(|s| {
        matches!(s, Segment::Text { bold: true, .. }
            | Segment::Text { italic: true, .. }
            | Segment::Text { underline: true, .. }
            | Segment::Text { strike: true, .. })
    });

    if has_formatting {
        let ncrtf = segments_to_ncrtf(&raw.segments);
        let mut v = json!({"type": "rich_text", "content": ncrtf});
        if let Some(align) = &raw.alignment {
            v["alignment"] = json!(align);
        }
        return v;
    }

    let text = segments_to_plain_text(&raw.segments);
    if text.is_empty() {
        return json!({"type": "spacer", "height_mm": 4.0});
    }
    let mut v = json!({"type": "paragraph", "text": text});
    if let Some(sr) = style_ref {
        v["style_ref"] = json!(sr);
    }
    if let Some(align) = &raw.alignment {
        if align != "left" {
            v["alignment"] = json!(align);
        }
    }
    v
}

fn render_list(lt: ListType, paras: Vec<RawPara>, ctx: &mut MappingContext) -> Value {
    let list_type = if lt == ListType::Ordered { "ordered" } else { "bullet" };
    let items: Vec<Value> = paras
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            if ctx.mode == OutputMode::Template {
                let key = ctx.placeholder("list_item", &format!("List item {}", idx + 1));
                json!(key)
            } else {
                json!(segments_to_plain_text(&p.segments))
            }
        })
        .collect();
    json!({"type": "list", "list_type": list_type, "items": items})
}

fn render_table(rows: Vec<Vec<String>>, ctx: &mut MappingContext) -> Value {
    if rows.is_empty() {
        return json!({"type": "table", "headers": [], "rows": []});
    }

    if ctx.mode == OutputMode::Template {
        let headers: Vec<Value> = rows[0]
            .iter()
            .map(|_| json!(ctx.placeholder("header", "Table header text")))
            .collect();
        let data_rows: Vec<Value> = rows[1..]
            .iter()
            .map(|r| {
                let cells: Vec<Value> = r
                    .iter()
                    .map(|_| json!(ctx.placeholder("cell", "Table cell text")))
                    .collect();
                json!(cells)
            })
            .collect();
        return json!({"type": "table", "headers": headers, "rows": data_rows});
    }

    let headers: Vec<Value> = rows[0].iter().map(|c| json!(c)).collect();
    let data_rows: Vec<Value> = rows[1..]
        .iter()
        .map(|r| {
            let cells: Vec<Value> = r.iter().map(|c| json!(c)).collect();
            json!(cells)
        })
        .collect();
    json!({"type": "table", "headers": headers, "rows": data_rows})
}

fn render_image(r_id: &str, ctx: &MappingContext) -> Value {
    let target = match ctx.relationships.get(r_id) {
        Some(t) => t,
        None => return json!({"type": "image", "src": ""}),
    };
    let bytes = match ctx.media.get(target.as_str()) {
        Some(b) => b,
        None => return json!({"type": "image", "src": ""}),
    };
    let mime = mime_from_path(target);
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    let src = format!("data:{mime};base64,{b64}");
    json!({"type": "image", "src": src, "width_percent": 80.0})
}

// ── Text helpers ──────────────────────────────────────────────────────────────

fn segments_to_plain_text(segments: &[Segment]) -> String {
    segments
        .iter()
        .map(|s| match s {
            Segment::Text { text, .. } => text.as_str(),
            Segment::LineBreak => "\n",
            Segment::Image { .. } => "",
        })
        .collect()
}

fn segments_to_ncrtf(segments: &[Segment]) -> String {
    let children: Vec<Value> = segments
        .iter()
        .filter_map(|s| match s {
            Segment::Text { text, bold, italic, underline, strike } => {
                let mut marks: Vec<Value> = Vec::new();
                if *bold { marks.push(json!("bold")); }
                if *italic { marks.push(json!("italic")); }
                if *underline { marks.push(json!("underline")); }
                if *strike { marks.push(json!("strikethrough")); }
                Some(if marks.is_empty() {
                    json!({"type": "text", "text": text})
                } else {
                    json!({"type": "text", "text": text, "marks": marks})
                })
            }
            Segment::LineBreak => Some(json!({"type": "hard_break"})),
            Segment::Image { .. } => None,
        })
        .collect();

    let ncrtf = json!({
        "ncrtf": "1.3.0",
        "blocks": [{"type": "paragraph", "children": children}]
    });
    serde_json::to_string(&ncrtf).unwrap_or_default()
}

// ── Misc helpers ──────────────────────────────────────────────────────────────

fn heading_level(style_id: &str) -> Option<u8> {
    let normalized = style_id.to_lowercase().replace(' ', "").replace('-', "");
    match normalized.as_str() {
        "heading1" | "ttulo1" | "title1" => Some(1),
        "heading2" | "ttulo2" | "title2" => Some(2),
        "heading3" | "ttulo3" | "title3" => Some(3),
        "heading4" | "ttulo4" | "title4" => Some(4),
        "heading5" | "ttulo5" | "title5" => Some(5),
        "heading6" | "ttulo6" | "title6" => Some(6),
        _ => None,
    }
}

fn map_jc(word_val: &str) -> &str {
    match word_val {
        "center" => "center",
        "right" | "end" => "right",
        "both" | "distribute" => "justify",
        _ => "left",
    }
}

fn mime_from_path(path: &str) -> &str {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "image/png",
    }
}
