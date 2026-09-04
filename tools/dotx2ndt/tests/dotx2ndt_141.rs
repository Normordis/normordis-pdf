// Integration tests for dotx2ndt.
// C1 — compat_mode extraction (C1-01 … C1-08)
// C2 — dummy paragraph suppression (C2-01 … C2-07)

use dotx2ndt::element_mapper::{MappingContext, map_body_from_xml};
use dotx2ndt::extractor::DotxExtractor;

const W_NS: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main""#;

fn settings_xml(val: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:settings {W_NS}>
  <w:compat>
    <w:compatSetting w:name="compatibilityMode"
      w:uri="http://schemas.microsoft.com/office/word"
      w:val="{val}"/>
  </w:compat>
</w:settings>"#,
        W_NS = W_NS,
        val = val
    )
}

fn body_xml(inner: &str) -> String {
    format!(
        r#"<w:document {W_NS}><w:body>{inner}</w:body></w:document>"#,
        W_NS = W_NS,
        inner = inner
    )
}

// ── C1: compat_mode extraction ────────────────────────────────────────────────

#[test]
fn c1_01_word2016_compat_mode_16() {
    let e = DotxExtractor {
        settings_xml: Some(settings_xml("16")),
        ..Default::default()
    };
    assert_eq!(e.extract_compat_mode(), Some(16));
}

#[test]
fn c1_02_word2013_compat_mode_15() {
    let e = DotxExtractor {
        settings_xml: Some(settings_xml("15")),
        ..Default::default()
    };
    assert_eq!(e.extract_compat_mode(), Some(15));
}

#[test]
fn c1_03_word2010_compat_mode_14() {
    let e = DotxExtractor {
        settings_xml: Some(settings_xml("14")),
        ..Default::default()
    };
    assert_eq!(e.extract_compat_mode(), Some(14));
}

#[test]
fn c1_04_word2007_compat_mode_12() {
    let e = DotxExtractor {
        settings_xml: Some(settings_xml("12")),
        ..Default::default()
    };
    assert_eq!(e.extract_compat_mode(), Some(12));
}

#[test]
fn c1_05_no_compat_setting_returns_none() {
    let e = DotxExtractor {
        settings_xml: Some(format!(
            r#"<w:settings {W_NS}><w:compat/></w:settings>"#,
            W_NS = W_NS
        )),
        ..Default::default()
    };
    assert_eq!(e.extract_compat_mode(), None);
}

#[test]
fn c1_06_settings_xml_none_returns_none() {
    let e = DotxExtractor::default();
    assert_eq!(e.extract_compat_mode(), None);
}

#[test]
fn c1_07_compat_mode_serialises_to_json() {
    let json = serde_json::json!({
        "ndt": "2.1.0",
        "meta": { "title": "Test", "compat_mode": 15 },
        "body": []
    });
    let s = serde_json::to_string(&json).unwrap();
    assert!(
        s.contains("\"compat_mode\":15"),
        "compat_mode must appear in JSON: {s}"
    );
}

#[test]
fn c1_08_compat_mode_none_omitted_from_json() {
    let json = serde_json::json!({
        "ndt": "2.1.0",
        "meta": { "title": "Test" },
        "body": []
    });
    let s = serde_json::to_string(&json).unwrap();
    assert!(
        !s.contains("compat_mode"),
        "compat_mode must be absent: {s}"
    );
}

// ── C2: dummy paragraph suppression ──────────────────────────────────────────

#[test]
fn c2_01_empty_para_between_tables_is_suppressed() {
    let xml = body_xml("<w:tbl/><w:p/><w:tbl/>");
    let elems = map_body_from_xml(&xml, &mut MappingContext::default()).unwrap();
    assert_eq!(
        elems.len(),
        2,
        "dummy paragraph between two tables must be suppressed"
    );
    assert!(elems.iter().all(|e| e["type"] == "table"));
}

#[test]
fn c2_02_para_after_single_table_is_kept() {
    let xml = body_xml("<w:tbl/><w:p/>");
    let elems = map_body_from_xml(&xml, &mut MappingContext::default()).unwrap();
    assert_eq!(
        elems.len(),
        2,
        "paragraph after table without a following table must be kept"
    );
}

#[test]
fn c2_03_para_before_single_table_is_kept() {
    let xml = body_xml("<w:p/><w:tbl/>");
    let elems = map_body_from_xml(&xml, &mut MappingContext::default()).unwrap();
    assert_eq!(
        elems.len(),
        2,
        "paragraph before table without a preceding table must be kept"
    );
}

#[test]
fn c2_04_para_with_text_between_tables_is_kept() {
    let xml = body_xml(r#"<w:tbl/><w:p><w:r><w:t>Hello</w:t></w:r></w:p><w:tbl/>"#);
    let elems = map_body_from_xml(&xml, &mut MappingContext::default()).unwrap();
    assert_eq!(
        elems.len(),
        3,
        "paragraph with visible text between tables must NOT be suppressed"
    );
    assert_eq!(elems[1]["type"], "paragraph");
    assert_eq!(elems[1]["text"], "Hello");
}

#[test]
fn c2_05_para_with_whitespace_only_between_tables_is_suppressed() {
    let xml = body_xml(r#"<w:tbl/><w:p><w:r><w:t>   </w:t></w:r></w:p><w:tbl/>"#);
    let elems = map_body_from_xml(&xml, &mut MappingContext::default()).unwrap();
    assert_eq!(
        elems.len(),
        2,
        "whitespace-only paragraph between tables must be suppressed"
    );
}

#[test]
fn c2_06_multiple_dummy_paras_all_suppressed() {
    let xml = body_xml("<w:tbl/><w:p/><w:tbl/><w:p/><w:tbl/>");
    let elems = map_body_from_xml(&xml, &mut MappingContext::default()).unwrap();
    assert_eq!(elems.len(), 3);
    assert!(elems.iter().all(|e| e["type"] == "table"));
}

#[test]
fn c2_07_non_dummy_para_between_tables_preserved() {
    let xml = body_xml(r#"<w:tbl/><w:p><w:r><w:t>Section title</w:t></w:r></w:p><w:tbl/>"#);
    let elems = map_body_from_xml(&xml, &mut MappingContext::default()).unwrap();
    assert_eq!(elems.len(), 3);
    assert_eq!(elems[1]["type"], "paragraph");
    assert_eq!(elems[1]["text"], "Section title");
}
