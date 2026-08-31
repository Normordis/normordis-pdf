use std::collections::HashMap;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// Whether a numbered list is ordered (1, 2, 3…) or unordered (bullets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    Bullet,
    Ordered,
}

/// Parses `word/numbering.xml` and returns a map of `numId → ListType`.
///
/// Falls back to [`ListType::Bullet`] for ambiguous or missing entries.
pub fn parse_numbering_xml(xml: &str) -> HashMap<u32, ListType> {
    let doc = match roxmltree::Document::parse(xml) {
        Ok(d) => d,
        Err(_) => return HashMap::new(),
    };

    // Step 1: abstractNumId → ListType (from the first ilvl's numFmt)
    let mut abstract_types: HashMap<u32, ListType> = HashMap::new();
    for node in doc.descendants() {
        if node.tag_name().name() != "abstractNum" {
            continue;
        }
        let abstract_id: u32 = match node
            .attribute((W_NS, "abstractNumId"))
            .and_then(|v| v.parse().ok())
        {
            Some(id) => id,
            None => continue,
        };
        // Find the first numFmt element (ilvl=0)
        for child in node.descendants() {
            if child.tag_name().name() != "numFmt" {
                continue;
            }
            let val = child.attribute((W_NS, "val")).unwrap_or("bullet");
            let lt = if matches!(
                val,
                "decimal"
                    | "lowerLetter"
                    | "upperLetter"
                    | "lowerRoman"
                    | "upperRoman"
                    | "ordinal"
                    | "cardinalText"
            ) {
                ListType::Ordered
            } else {
                ListType::Bullet
            };
            abstract_types.insert(abstract_id, lt);
            break;
        }
    }

    // Step 2: numId → abstractNumId → ListType
    let mut result: HashMap<u32, ListType> = HashMap::new();
    for node in doc.descendants() {
        if node.tag_name().name() != "num" {
            continue;
        }
        let num_id: u32 = match node.attribute((W_NS, "numId")).and_then(|v| v.parse().ok()) {
            Some(id) => id,
            None => continue,
        };
        for child in node.children() {
            if child.tag_name().name() == "abstractNumId"
                && let Some(abs_id) = child
                    .attribute((W_NS, "val"))
                    .and_then(|v| v.parse::<u32>().ok())
            {
                let lt = abstract_types
                    .get(&abs_id)
                    .copied()
                    .unwrap_or(ListType::Bullet);
                result.insert(num_id, lt);
            }
        }
    }

    result
}
