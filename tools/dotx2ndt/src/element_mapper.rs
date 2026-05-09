use serde_json::Value;

use crate::error::Dotx2NdtError;

/// Maps the body of a Word document XML to NDT body element JSON values,
/// suppressing dummy anchor paragraphs between consecutive tables.
pub fn map_body_from_xml(document_xml: &str) -> Result<Vec<Value>, Dotx2NdtError> {
    let doc = roxmltree::Document::parse(document_xml)
        .map_err(|e| Dotx2NdtError::Xml(format!("{e}")))?;

    let body = doc
        .descendants()
        .find(|n| n.tag_name().name() == "body")
        .ok_or_else(|| Dotx2NdtError::Xml("missing w:body".into()))?;

    let children: Vec<roxmltree::Node> =
        body.children().filter(|n| n.is_element()).collect();

    let mut result = Vec::new();
    for (i, node) in children.iter().enumerate() {
        let prev_is_table = i > 0 && children[i - 1].tag_name().name() == "tbl";
        let next_is_table =
            i + 1 < children.len() && children[i + 1].tag_name().name() == "tbl";

        if is_dummy_paragraph(node, prev_is_table, next_is_table) {
            continue;
        }

        if let Some(elem) = map_node(node) {
            result.push(elem);
        }
    }

    Ok(result)
}

/// Returns `true` if `node` is a dummy anchor paragraph that should be suppressed.
///
/// A dummy paragraph is a `w:p` element that sits between two consecutive tables
/// (`prev_is_table && next_is_table`) and contains no visible text. Word inserts
/// these as structural anchors; they carry no semantic content in the NDT output.
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
    let has_visible_text = node
        .descendants()
        .filter(|n| n.tag_name().name() == "t")
        .any(|n| !n.text().unwrap_or("").trim().is_empty());
    !has_visible_text
}

fn map_node(node: &roxmltree::Node) -> Option<Value> {
    match node.tag_name().name() {
        "p" => {
            let text = collect_text(node);
            Some(serde_json::json!({ "type": "paragraph", "text": text }))
        }
        "tbl" => Some(serde_json::json!({ "type": "table", "rows": [] })),
        _ => None,
    }
}

fn collect_text(node: &roxmltree::Node) -> String {
    node.descendants()
        .filter(|n| n.tag_name().name() == "t")
        .filter_map(|n| n.text())
        .collect()
}
