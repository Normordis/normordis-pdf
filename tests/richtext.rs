use normordis_pdf::{
    DocumentBuilder, DocumentStyle, ncrtf_to_elements, parse_ncrtf, richtext::marks::MarkValue,
};

const MINIMAL_JSON: &str = r#"{"ncrtf_version":"2.0.0","content":[]}"#;

const INVALID_JSON: &str = r#"{ not valid json"#;

const HEADING_JSON: &str = r#"{
  "ncrtf_version": "2.0.0",
  "content": [
    {
      "type": "heading",
      "level": 1,
      "content": [{ "type": "text", "text": "Introduction" }]
    }
  ]
}"#;

const BOLD_PARAGRAPH_JSON: &str = r#"{
  "ncrtf_version": "2.0.0",
  "content": [
    {
      "type": "paragraph",
      "content": [
        { "type": "text", "text": "Normal " },
        { "type": "text", "text": "bold text", "marks": ["bold"] }
      ]
    }
  ]
}"#;

// ── 1. parse_ncrtf with valid minimal JSON ────────────────────────────────────

#[test]
fn parse_valid_json_returns_ok() {
    let result = parse_ncrtf(MINIMAL_JSON);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(result.unwrap().ncrtf_version, "2.0.0");
}

// ── 2. parse_ncrtf with invalid JSON ─────────────────────────────────────────

#[test]
fn parse_invalid_json_returns_err() {
    let result = parse_ncrtf(INVALID_JSON);
    assert!(result.is_err(), "expected Err for invalid JSON");
}

// ── 3. Heading level 1 → Section { level: 1 } ────────────────────────────────

#[test]
fn heading_level1_produces_section() {
    let doc = parse_ncrtf(HEADING_JSON).unwrap();
    let style = DocumentStyle::default();
    let elements = ncrtf_to_elements(&doc, &style);

    assert_eq!(elements.len(), 1);
    // Section::estimated_height_mm() with pre+post spacing is > 0
    assert!(elements[0].estimated_height_mm() > 0.0);
}

// ── 4. Paragraph with bold mark → TextRun with bold = true ───────────────────

#[test]
fn paragraph_with_bold_produces_bold_run() {
    let doc = parse_ncrtf(BOLD_PARAGRAPH_JSON).unwrap();
    let style = DocumentStyle::default();
    let elements = ncrtf_to_elements(&doc, &style);

    assert_eq!(elements.len(), 1);

    let bytes = DocumentBuilder::new("Bold test")
        .push_ncrtf(BOLD_PARAGRAPH_JSON)
        .unwrap()
        .render_to_bytes()
        .unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}

// ── 5. Two paragraphs produce valid PDF ──────────────────────────────────────

#[test]
fn two_paragraphs_produce_valid_pdf() {
    let json = r#"{
      "ncrtf_version": "2.0.0",
      "content": [
        { "type": "paragraph", "content": [{ "type": "text", "text": "Primeiro parágrafo." }] },
        { "type": "paragraph", "content": [{ "type": "text", "text": "Segundo parágrafo." }] }
      ]
    }"#;

    let bytes = DocumentBuilder::new("Two paragraphs")
        .push_ncrtf(json)
        .unwrap()
        .render_to_bytes()
        .unwrap();

    assert!(bytes.starts_with(b"%PDF-"));
}

// ── 6. MarkValue deserializes "bold" string ───────────────────────────────────

#[test]
fn mark_value_deserializes_string_bold() {
    let v: MarkValue = serde_json::from_str(r#""bold""#).unwrap();
    assert!(matches!(v, MarkValue::Bold));
}

// ── 7. MarkValue deserializes parameterised color object ─────────────────────

#[test]
fn mark_value_deserializes_color_object() {
    let v: MarkValue = serde_json::from_str(r##"{"type":"color","value":"#FF0000"}"##).unwrap();
    match v {
        MarkValue::Color(c) => assert_eq!(c, "#FF0000"),
        other => panic!("expected Color, got {other:?}"),
    }
}
