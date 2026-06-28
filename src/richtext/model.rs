use serde::Deserialize;

use super::marks::MarkValue;
use crate::layout::TextAlign;

// ── Root document ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct NcrtfDocument {
    pub ncrtf_version: String,
    pub content: Vec<Block>,
}

// ── Block nodes ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Paragraph(ParagraphBlock),
    Heading(HeadingBlock),
    List(ListBlock),
    Blockquote(BlockquoteBlock),
    Table(TableBlock),
    Image(ImageBlock),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParagraphBlock {
    pub alignment: Option<TextAlign>,
    pub indent: Option<u8>,
    pub font_family: Option<String>,
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HeadingBlock {
    /// Heading depth 1–3.
    pub level: u8,
    pub alignment: Option<TextAlign>,
    pub font_family: Option<String>,
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListBlock {
    pub list_type: ListType,
    pub alignment: Option<TextAlign>,
    pub content: Vec<ListItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListItem {
    /// Only present when `list_type = "checklist"`.
    pub checked: Option<bool>,
    /// Inline nodes or a nested list.
    pub content: Vec<ListItemContent>,
}

/// Content node inside a list item — either an inline node or a nested list.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListItemContent {
    Text(TextNode),
    Link(LinkNode),
    HardBreak,
    List(ListBlock),
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableBlock {
    #[serde(default)]
    pub head: Vec<TableRow>,
    pub body: Vec<TableRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableRow {
    pub cells: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockquoteBlock {
    pub alignment: Option<TextAlign>,
    pub font_family: Option<String>,
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageBlock {
    /// Asset key, e.g. `"assets/logo.png"`. Not a data URI or HTTP URL.
    #[serde(rename = "ref")]
    pub image_ref: String,
    pub alt: String,
    pub caption: Option<String>,
    pub width_percent: Option<u32>,
}

// ── Inline nodes ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Inline {
    Text(TextNode),
    Link(LinkNode),
    HardBreak,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextNode {
    pub text: String,
    pub marks: Option<Vec<MarkValue>>,
    pub font_family: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LinkNode {
    pub href: String,
    pub title: Option<String>,
    /// `"_blank"` | `"_self"`
    pub target: Option<String>,
    /// Link label — only `text` nodes, no nested inlines.
    pub content: Vec<TextNode>,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListType {
    Bullet,
    Ordered,
    Checklist,
}
