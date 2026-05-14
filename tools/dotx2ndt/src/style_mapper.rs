// Maps Word paragraph style XML attributes to normordis-pdf NamedStyle fields.

use std::collections::HashMap;

use roxmltree::Document;
use serde::{Deserialize, Serialize};

use crate::error::Dotx2NdtError;

/// Intermediate representation of a Word paragraph style extracted from styles.xml.
#[derive(Debug, Serialize, Deserialize)]
pub struct WordStyle {
    pub id: String,
    pub name: String,
    pub based_on: Option<String>,
    pub font_size_pt: Option<f64>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub alignment: Option<String>,
    pub space_before_pt: Option<f64>,
    pub space_after_pt: Option<f64>,
    pub indent_left_twips: Option<i64>,
    pub indent_right_twips: Option<i64>,
    pub indent_first_line_twips: Option<i64>,
}

/// A minimal NDT-compatible NamedStyle output entry.
#[derive(Debug, Serialize, Deserialize)]
pub struct NdtNamedStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_before_mm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_after_mm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_left_mm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_right_mm: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_first_line_mm: Option<f64>,
}

const TWIPS_PER_MM: f64 = 56.692_913_385_826_77; // 1440 twips/inch ÷ 25.4 mm/inch
const PT_TO_MM: f64 = 25.4 / 72.0;

pub fn parse_styles_xml(xml: &str) -> Result<Vec<WordStyle>, Dotx2NdtError> {
    let doc = Document::parse(xml).map_err(|e| Dotx2NdtError::Xml(format!("{}", e)))?;

    let mut styles = Vec::new();

    for node in doc.descendants() {
        if node.tag_name().name() != "style" {
            continue;
        }
        if node
            .attribute((
                "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                "type",
            ))
            .unwrap_or("")
            != "paragraph"
        {
            continue;
        }

        let id = node
            .attribute((
                "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                "styleId",
            ))
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            continue;
        }

        let mut name = id.clone();
        let mut based_on: Option<String> = None;
        let mut font_size_pt: Option<f64> = None;
        let mut bold: Option<bool> = None;
        let mut italic: Option<bool> = None;
        let mut alignment: Option<String> = None;
        let mut space_before_pt: Option<f64> = None;
        let mut space_after_pt: Option<f64> = None;
        let mut indent_left_twips: Option<i64> = None;
        let mut indent_right_twips: Option<i64> = None;
        let mut indent_first_line_twips: Option<i64> = None;

        for child in node.descendants() {
            let local = child.tag_name().name();
            match local {
                "name" => {
                    if let Some(val) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "val",
                    )) {
                        name = val.to_string();
                    }
                }
                "basedOn" => {
                    if let Some(val) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "val",
                    )) {
                        based_on = Some(val.to_string());
                    }
                }
                "sz" | "szCs" if local == "sz" => {
                    if let Some(val) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "val",
                    )) {
                        if let Ok(half_pts) = val.parse::<f64>() {
                            font_size_pt = Some(half_pts / 2.0);
                        }
                    }
                }
                "b" => {
                    let val = child
                        .attribute((
                            "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                            "val",
                        ))
                        .unwrap_or("true");
                    bold = Some(val != "false" && val != "0");
                }
                "i" => {
                    let val = child
                        .attribute((
                            "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                            "val",
                        ))
                        .unwrap_or("true");
                    italic = Some(val != "false" && val != "0");
                }
                "jc" => {
                    if let Some(val) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "val",
                    )) {
                        alignment = Some(map_jc(val).to_string());
                    }
                }
                "spacing" => {
                    if let Some(before) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "before",
                    )) {
                        if let Ok(twips) = before.parse::<f64>() {
                            space_before_pt = Some(twips / 20.0);
                        }
                    }
                    if let Some(after) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "after",
                    )) {
                        if let Ok(twips) = after.parse::<f64>() {
                            space_after_pt = Some(twips / 20.0);
                        }
                    }
                }
                "ind" => {
                    if let Some(left) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "left",
                    )) {
                        indent_left_twips = left.parse().ok();
                    }
                    if let Some(right) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "right",
                    )) {
                        indent_right_twips = right.parse().ok();
                    }
                    if let Some(first) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "firstLine",
                    )) {
                        indent_first_line_twips = first.parse().ok();
                    }
                    if let Some(hang) = child.attribute((
                        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                        "hanging",
                    )) {
                        if let Ok(v) = hang.parse::<i64>() {
                            indent_first_line_twips = Some(-v);
                        }
                    }
                }
                _ => {}
            }
        }

        styles.push(WordStyle {
            id,
            name,
            based_on,
            font_size_pt,
            bold,
            italic,
            alignment,
            space_before_pt,
            space_after_pt,
            indent_left_twips,
            indent_right_twips,
            indent_first_line_twips,
        });
    }

    Ok(styles)
}

fn map_jc(word_val: &str) -> &str {
    match word_val {
        "center" => "center",
        "right" | "end" => "right",
        "both" | "distribute" => "justify",
        _ => "left",
    }
}

/// Converts a Vec of WordStyle to a HashMap<style_name, NdtNamedStyle> suitable for JSON output.
pub fn to_ndt_styles(word_styles: &[WordStyle]) -> HashMap<String, NdtNamedStyle> {
    let name_for_id: HashMap<&str, &str> = word_styles
        .iter()
        .map(|s| (s.id.as_str(), s.name.as_str()))
        .collect();

    let mut out = HashMap::new();

    for ws in word_styles {
        let ndt_name = slugify(&ws.name);
        let extends = ws
            .based_on
            .as_deref()
            .and_then(|id| name_for_id.get(id))
            .map(|n| slugify(n));

        let ndt = NdtNamedStyle {
            extends,
            font_size: ws.font_size_pt,
            bold: ws.bold,
            italic: ws.italic,
            alignment: ws.alignment.clone(),
            space_before_mm: ws.space_before_pt.map(|pt| pt * PT_TO_MM),
            space_after_mm: ws.space_after_pt.map(|pt| pt * PT_TO_MM),
            indent_left_mm: ws.indent_left_twips.map(|t| t as f64 / TWIPS_PER_MM),
            indent_right_mm: ws.indent_right_twips.map(|t| t as f64 / TWIPS_PER_MM),
            indent_first_line_mm: ws.indent_first_line_twips.map(|t| t as f64 / TWIPS_PER_MM),
        };

        out.insert(ndt_name, ndt);
    }

    out
}

/// Converts a style name to a lowercase_underscore slug suitable as a JSON key.
fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c.to_lowercase().next().unwrap()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
