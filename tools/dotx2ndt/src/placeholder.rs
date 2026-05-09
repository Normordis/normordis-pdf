use std::collections::HashMap;

use serde_json::{json, Value};

use crate::style_mapper::NdtNamedStyle;

/// Builds a minimal NDT JSON skeleton embedding the extracted named styles.
pub fn build_ndt_skeleton(
    named_styles: &HashMap<String, NdtNamedStyle>,
    title: &str,
    compat_mode: Option<u32>,
) -> Value {
    let styles_value: Value = serde_json::to_value(named_styles)
        .unwrap_or(Value::Object(Default::default()));

    let mut meta = json!({ "title": title });
    if let Some(mode) = compat_mode {
        meta["compat_mode"] = json!(mode);
    }

    json!({
        "ndt": "1.4.0",
        "meta": meta,
        "style": {
            "named_styles": styles_value
        },
        "body": [
            {
                "type": "paragraph",
                "style_ref": "normal",
                "text": "<!-- Replace with document content -->"
            }
        ]
    })
}
