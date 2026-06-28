use normordis_pdf::{
    DocumentBuilder, DocumentStyle, parse_ndt, parse_ndt_data, serialize_ndt_json,
    serialize_ndt_toml,
    template::{TemplateError, check_version_compatibility},
};

const MINIMAL_NDT: &str = r#"{
    "ndt_version": "2.0.0",
    "schema_id": "urn:normordis:ndt:test",
    "versao_ndt": "1.0.0",
    "paginas_def": [{"id": "p1"}],
    "sequencia": [{"pagina_def": "p1", "repeticao": "unica"}]
}"#;

const MINIMAL_DATA: &str = r#"{"ndt_data":"1.0.0","data":{}}"#;

// ── 1. parse_ndt valid ────────────────────────────────────────────────────────

#[test]
fn parse_ndt_valid_json_returns_ok() {
    assert!(parse_ndt(MINIMAL_NDT).is_ok());
}

// ── 2. parse_ndt invalid ──────────────────────────────────────────────────────

#[test]
fn parse_ndt_invalid_json_returns_err() {
    let result = parse_ndt("{not valid json");
    assert!(result.is_err(), "expected error for invalid JSON");
    assert!(matches!(result.unwrap_err(), TemplateError::JsonError(_)));
}

// ── 3. check_version_compatibility same version ───────────────────────────────

#[test]
fn version_compatibility_exact_match_ok() {
    assert!(check_version_compatibility("2.0.0").is_ok());
}

// ── 4. check_version_compatibility different minor ────────────────────────────

#[test]
fn version_compatibility_minor_ok() {
    assert!(check_version_compatibility("2.1.0").is_ok());
}

// ── 5. check_version_compatibility future major → error ───────────────────────

#[test]
fn version_compatibility_major_mismatch_err() {
    let result = check_version_compatibility("3.0.0");
    assert!(result.is_err(), "future major version must be rejected");
    assert!(matches!(
        result.unwrap_err(),
        TemplateError::IncompatibleVersion { .. }
    ));
}

#[test]
fn version_compatibility_v200_is_accepted() {
    assert!(check_version_compatibility("2.0.0").is_ok());
}


// ── 8. resolve_string replaces placeholder ────────────────────────────────────

#[test]
fn resolve_string_replaces_placeholder() {
    use normordis_pdf::template::resolver;

    let data = parse_ndt_data(r#"{"ndt_data":"1.0.0","data":{"name":"Maria"}}"#).unwrap();
    let result = resolver::resolve_string("Olá {{name}}!", &data);
    assert_eq!(result, "Olá Maria!");
}

// ── 9. resolve_string unknown key left as-is ──────────────────────────────────

#[test]
fn resolve_string_unknown_key_preserved() {
    use normordis_pdf::template::resolver;

    let data = parse_ndt_data(MINIMAL_DATA).unwrap();
    let result = resolver::resolve_string("Hello {{name}}", &data);
    assert_eq!(result, "Hello {{name}}");
}

// ── 10. resolve_string nested key ────────────────────────────────────────────

#[test]
fn resolve_string_nested_key() {
    use normordis_pdf::template::resolver;

    let json = r#"{"ndt_data":"1.0.0","data":{"obj":{"field":"world"}}}"#;
    let data = parse_ndt_data(json).unwrap();
    let result = resolver::resolve_string("Hello {{obj.field}}", &data);
    assert_eq!(result, "Hello world");
}

// ── 15. push_ndt returns RenderError (positioned renderer not yet impl) ───────

#[test]
fn push_ndt_returns_render_error() {
    let pdf_result = DocumentBuilder::new("NDT Test")
        .push_ndt(MINIMAL_NDT, MINIMAL_DATA);

    assert!(pdf_result.is_err(), "push_ndt with NDT 2.0.0 must return Err until renderer is implemented");
}

// ── 16. parse_ndt TOML auto-detect ───────────────────────────────────────────

#[test]
fn parse_ndt_toml_returns_ok() {
    let toml = r#"
ndt_version = "2.0.0"
schema_id = "urn:normordis:ndt:test"
versao_ndt = "1.0.0"

[[paginas_def]]
id = "pagina1"

[[sequencia]]
pagina_def = "pagina1"
repeticao = "unica"
"#;
    let doc = parse_ndt(toml).expect("TOML NDT should parse");
    assert_eq!(doc.ndt_version, "2.0.0");
    assert_eq!(doc.paginas_def.len(), 1);
}

// ── 17. parse_ndt invalid TOML returns TomlError ──────────────────────────────

#[test]
fn parse_ndt_invalid_toml_returns_err() {
    let result = parse_ndt("ndt_version = this is not valid toml !!!!");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TemplateError::TomlError(_)));
}

// ── 18. serialize_ndt_json round-trip ─────────────────────────────────────────

#[test]
fn serialize_ndt_json_round_trip() {
    let doc = parse_ndt(MINIMAL_NDT).unwrap();
    let json = serialize_ndt_json(&doc).expect("JSON serialization should succeed");
    let doc2 = parse_ndt(&json).expect("re-parsed document should be valid");
    assert_eq!(doc.ndt_version, doc2.ndt_version);
    assert_eq!(doc.schema_id, doc2.schema_id);
    assert_eq!(doc.paginas_def.len(), doc2.paginas_def.len());
}

// ── 19. serialize_ndt_toml round-trip ────────────────────────────────────────

#[test]
fn serialize_ndt_toml_round_trip() {
    let doc = parse_ndt(MINIMAL_NDT).unwrap();
    let toml_str = serialize_ndt_toml(&doc).expect("TOML serialization should succeed");
    let doc2 = parse_ndt(&toml_str).expect("TOML round-trip should parse back");
    assert_eq!(doc.ndt_version, doc2.ndt_version);
    assert_eq!(doc.schema_id, doc2.schema_id);
    assert_eq!(doc.paginas_def.len(), doc2.paginas_def.len());
}
