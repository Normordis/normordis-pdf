use normordis_pdf::*;

// ── Font registration ─────────────────────────────────────────────────────────

#[test]
fn cf_01_register_bytes_liberation_sans() {
    let mut reg = FontRegistry::default();
    let bytes = include_bytes!("../assets/fonts/LiberationSans-Regular.ttf");
    assert!(reg.register_bytes("TestSans", bytes, None, None, None).is_ok());
    assert!(reg.contains("TestSans"));
}

#[test]
fn cf_02_register_bytes_invalid_data() {
    let mut reg = FontRegistry::default();
    let result = reg.register_bytes("BadFont", b"not a font", None, None, None);
    assert!(result.is_err());
}

#[test]
fn cf_03_register_single_bytes_only_regular() {
    let mut reg = FontRegistry::default();
    let bytes = include_bytes!("../assets/fonts/LiberationMono-Regular.ttf");
    reg.register_single_bytes("MonoOnly", bytes).unwrap();
    assert!(reg.contains("MonoOnly"));
    // Bold/italic fall back to regular — resolve() must succeed.
    let fam = reg.get_family("MonoOnly");
    assert_eq!(fam.name, "MonoOnly");
    assert!(fam.bold.is_none());
}

#[test]
fn cf_04_contains_liberation_sans() {
    let reg = FontRegistry::default();
    assert!(reg.contains("LiberationSans"));
}

#[test]
fn cf_05_contains_nonexistent_font() {
    let reg = FontRegistry::default();
    assert!(!reg.contains("FantasyFontThatDoesNotExist"));
}

#[test]
fn cf_06_registered_families_includes_liberation_sans() {
    let reg = FontRegistry::default();
    let families = reg.registered_families();
    assert!(families.contains(&"LiberationSans"), "missing LiberationSans in {families:?}");
}

// ── Aliases ───────────────────────────────────────────────────────────────────

#[test]
fn cf_07_resolve_arial_alias() {
    let reg = FontRegistry::default();
    assert_eq!(reg.resolve("Arial").name, "LiberationSans");
}

#[test]
fn cf_08_resolve_times_new_roman_alias() {
    let reg = FontRegistry::default();
    assert_eq!(reg.resolve("Times New Roman").name, "LiberationSerif");
}

#[test]
fn cf_09_add_custom_alias() {
    let mut reg = FontRegistry::default();
    reg.add_alias("MinhaNova", "LiberationSans");
    assert_eq!(reg.resolve("MinhaNova").name, "LiberationSans");
}

#[test]
fn cf_10_alias_cycle_does_not_panic() {
    let mut reg = FontRegistry::default();
    // A → B → A
    reg.add_alias("CycleA", "CycleB");
    reg.add_alias("CycleB", "CycleA");
    // Must not panic; falls back to default.
    let fam = reg.resolve("CycleA");
    assert_eq!(fam.name, "LiberationSans");
}

// ── detect_variant_suffix (via load_dir behaviour) ───────────────────────────

// We test the public surface through register_file/load_dir rather than
// the private helper directly.

#[test]
fn cf_16_load_dir_with_valid_fonts() {
    let mut reg = FontRegistry::default();
    let count = reg.load_dir("assets/fonts/").unwrap();
    assert!(count > 0, "expected at least one family from assets/fonts/");
}

#[test]
fn cf_17_load_dir_nonexistent() {
    let mut reg = FontRegistry::default();
    let result = reg.load_dir("does/not/exist/");
    assert!(result.is_err());
}

#[test]
fn cf_18_load_dir_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let mut reg = FontRegistry::default();
    let count = reg.load_dir(tmp.path()).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn cf_19_load_dir_groups_variants() {
    let mut reg = FontRegistry::default();
    reg.load_dir("assets/fonts/").unwrap();
    // Liberation Serif should have bold/italic variants.
    let fam = reg.get_family("LiberationSerif");
    assert_eq!(fam.name, "LiberationSerif");
    assert!(fam.bold.is_some(), "LiberationSerif should have Bold");
    assert!(fam.italic.is_some(), "LiberationSerif should have Italic");
}

// ── DocumentBuilder ───────────────────────────────────────────────────────────

#[test]
fn cf_20_font_from_bytes_available_in_render() {
    let pdf = DocumentBuilder::new("Test")
        .font_from_bytes(
            "CustomSans",
            include_bytes!("../assets/fonts/LiberationSans-Regular.ttf"),
            None, None, None,
        )
        .unwrap()
        .push(Paragraph::new("Texto em CustomSans.").font_family("CustomSans"))
        .render_to_bytes();
    assert!(pdf.is_ok(), "{pdf:?}");
    assert!(!pdf.unwrap().is_empty());
}

#[test]
fn cf_21_fonts_from_dir_available_in_render() {
    let pdf = DocumentBuilder::new("Test")
        .fonts_from_dir("assets/fonts/")
        .unwrap()
        .push(Paragraph::new("Texto normal."))
        .render_to_bytes();
    assert!(pdf.is_ok(), "{pdf:?}");
}

#[test]
fn cf_22_default_font_liberation_serif() {
    let pdf = DocumentBuilder::new("Test")
        .default_font("LiberationSerif")
        .unwrap()
        .push(Paragraph::new("Texto em Serif."))
        .render_to_bytes();
    assert!(pdf.is_ok(), "{pdf:?}");
}

#[test]
fn cf_23_unknown_font_family_fallback_no_panic() {
    // Should emit a warning to stderr but NOT panic or return an error.
    let pdf = DocumentBuilder::new("Test")
        .push(Paragraph::new("Texto com fonte inexistente.").font_family("NomeInexistente"))
        .render_to_bytes();
    assert!(pdf.is_ok(), "expected Ok, got {pdf:?}");
}

// ── FontFallbackChain ─────────────────────────────────────────────────────────

#[test]
fn cf_24_default_fallback_chain_has_liberation_families() {
    let style = DocumentStyle::default();
    let families = &style.font_fallback.fonts;
    assert!(families.iter().any(|f| f == "LiberationSans"));
    assert!(families.iter().any(|f| f == "LiberationSerif"));
    assert!(families.iter().any(|f| f == "LiberationMono"));
}

#[test]
fn cf_25_unknown_font_uses_fallback_chain() {
    // Load a style with a custom fallback chain that points to LiberationSerif.
    let mut style = DocumentStyle::default();
    style.font_fallback = FontFallbackChain::new(vec!["LiberationSerif"]);

    let pdf = DocumentBuilder::new("Test")
        .style(style)
        .push(Paragraph::new("Fallback test.").font_family("GhostFont"))
        .render_to_bytes();
    assert!(pdf.is_ok(), "{pdf:?}");
}

#[test]
fn cf_26_empty_fallback_chain_uses_default() {
    let mut style = DocumentStyle::default();
    style.font_fallback = FontFallbackChain::default(); // empty

    let pdf = DocumentBuilder::new("Test")
        .style(style)
        .push(Paragraph::new("Default fallback.").font_family("GhostFont"))
        .render_to_bytes();
    assert!(pdf.is_ok(), "{pdf:?}");
}

// ── Render with custom font ───────────────────────────────────────────────────

#[test]
fn cf_27_liberation_serif_paragraph_no_panic() {
    let pdf = DocumentBuilder::new("Test")
        .push(Paragraph::new("Texto em Serif.").font_family("LiberationSerif"))
        .render_to_bytes();
    assert!(pdf.is_ok(), "{pdf:?}");
}

#[test]
fn cf_28_pdf_opens_with_lopdf() {
    let bytes = DocumentBuilder::new("Test")
        .push(Paragraph::new("Texto de teste."))
        .render_to_bytes()
        .unwrap();
    let doc = lopdf::Document::load_mem(&bytes);
    assert!(doc.is_ok(), "lopdf failed to parse: {doc:?}");
}

#[test]
fn cf_29_font_family_override_renders_correctly() {
    let pdf = DocumentBuilder::new("Test")
        .push(Paragraph::new("Override.").font_family("LiberationMono"))
        .render_to_bytes();
    assert!(pdf.is_ok(), "{pdf:?}");
}

#[test]
fn cf_30_table_with_different_fonts_no_panic() {
    let table = Table::builder()
        .row(vec![
            TableCell::new("Sans"),
            TableCell::new("Serif"),
            TableCell::new("Mono"),
        ])
        .build();
    // Verify that a document with per-paragraph font families around a table
    // renders without panic.
    let pdf = DocumentBuilder::new("Test")
        .push(Paragraph::new("Sans font.").font_family("LiberationSans"))
        .push(Paragraph::new("Serif font.").font_family("LiberationSerif"))
        .push(Paragraph::new("Mono font.").font_family("LiberationMono"))
        .push(table)
        .render_to_bytes();
    assert!(pdf.is_ok(), "{pdf:?}");
}
