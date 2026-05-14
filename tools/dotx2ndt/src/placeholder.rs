use std::collections::HashMap;

use serde_json::{json, Value};

use crate::style_mapper::NdtNamedStyle;

/// Builds the final NDT JSON output from extracted components.
///
/// - `named_styles` — Word styles converted to NDT format
/// - `title` — document title
/// - `compat_mode` — Word compat version (if extracted)
/// - `body` — rendered body elements
/// - `placeholders` — `(key, description)` pairs for template mode (empty in document mode)
pub fn build_ndt_output(
    named_styles: &HashMap<String, NdtNamedStyle>,
    title: &str,
    compat_mode: Option<u32>,
    body: Vec<Value>,
    placeholders: Vec<(String, String)>,
) -> Value {
    let styles_value: Value = serde_json::to_value(named_styles)
        .unwrap_or(Value::Object(Default::default()));

    let mut meta = json!({ "title": title });
    if let Some(mode) = compat_mode {
        meta["compat_mode"] = json!(mode);
    }

    let mut doc = json!({
        "ndt": "2.1.0",
        "meta": meta,
        "style": {
            "named_styles": styles_value
        },
        "body": body
    });

    if !placeholders.is_empty() {
        let mut ph_map = serde_json::Map::new();
        for (key, desc) in placeholders {
            ph_map.insert(
                key,
                json!({"type": "string", "required": true, "description": desc}),
            );
        }
        doc["placeholders"] = Value::Object(ph_map);
    }

    doc
}

/// Builds a minimal NDT skeleton (styles only, placeholder body).
///
/// Kept for backwards compatibility with existing callers.
pub fn build_ndt_skeleton(
    named_styles: &HashMap<String, NdtNamedStyle>,
    title: &str,
    compat_mode: Option<u32>,
) -> Value {
    let body = vec![json!({
        "type": "paragraph",
        "style_ref": "normal",
        "text": "<!-- Replace with document content -->"
    })];
    build_ndt_output(named_styles, title, compat_mode, body, vec![])
}
