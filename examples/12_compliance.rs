//! Demonstrates v2.0.0: PDF/A-1b, font subsetting, opacity, and traceability.
//! Run: cargo run --example 12_compliance -p normordis-pdf

use normordis_pdf::{
    BulletList, CompressionLevel, DocumentBuilder, InstitutionalHeader, ListItemElement,
    NDT_VERSION, PDF_BACKEND, PageFooter, Paragraph, PdfStandard, Result, RgbColor, Section,
    SecurityClassification, Spacer, TraceabilityMetadata, VERSION, Watermark,
};

fn main() -> Result<()> {
    let out_dir = std::env::temp_dir();

    // ── PDF/A-1b com traceabilidade ───────────────────────────────────
    let pdf_a = DocumentBuilder::new("Acta n.º 1/2026")
        .standard(PdfStandard::PdfA1b)
        .compression(CompressionLevel::Best)
        .traceability(TraceabilityMetadata {
            engine_version: VERSION.into(),
            entity_id: "cm-lisboa".into(),
            document_ref: Some("ACT/2026/001".into()),
            // Public: sem marca de água automática — PDF/A-1 proíbe
            // transparência e a biblioteca recusa a combinação.
            classification: SecurityClassification::Public,
            generated_at: "2026-01-01T00:00:00Z".into(),
            ndt_version: NDT_VERSION.into(),
            framework_version: None,
        })
        .header(
            InstitutionalHeader::new("Câmara Municipal de Lisboa", "Acta n.º 1/2026")
                .with_reference("ACT/2026/001")
                .with_date("29 de Abril de 2026"),
        )
        .footer(
            PageFooter::new()
                .left("ACT/2026/001 — INTERNO")
                .right("Pág. {{page}} / {{total_pages}}"),
        )
        .push(Section::new("1. Abertura da Sessão", 1))
        .push(Paragraph::new(
            "Reuniu a Câmara Municipal de Lisboa, em sessão ordinária, \
             na sala de reuniões dos Paços do Concelho, pelas 10h00.",
        ))
        .push(Spacer::new(4.0))
        .push(Section::new("2. Ordem do Dia", 1))
        .push(BulletList::new(vec![
            ListItemElement::plain("Aprovação da acta anterior"),
            ListItemElement::plain("Ponto 1: Aprovação do Orçamento Municipal 2027"),
            ListItemElement::plain("Ponto 2: Deliberações diversas"),
        ]))
        .render_to_bytes()?;

    let path_a = out_dir.join("normordis_pdfa.pdf");
    std::fs::write(&path_a, &pdf_a)?;
    println!(
        "PDF/A-1b:       {} ({} KB)",
        path_a.display(),
        pdf_a.len() / 1024
    );

    // ── PDF/A-2b com classificação e marca de água translúcida ───────
    // O PDF/A-2 (ISO 19005-2) permite transparência; é o perfil correto
    // para documentos com marca de água de classificação translúcida.
    let pdf_a2 = DocumentBuilder::new("Despacho n.º 7/2026")
        .standard(PdfStandard::PdfA2b)
        .compression(CompressionLevel::Best)
        .traceability(TraceabilityMetadata {
            engine_version: VERSION.into(),
            entity_id: "cm-lisboa".into(),
            document_ref: Some("DSP/2026/007".into()),
            classification: SecurityClassification::Internal,
            generated_at: "2026-01-01T00:00:00Z".into(),
            ndt_version: NDT_VERSION.into(),
            framework_version: None,
        })
        .push(Section::new("Despacho", 1))
        .push(Paragraph::new(
            "Determino a abertura do procedimento referido em epígrafe, nos \
             termos da informação técnica anexa.",
        ))
        .render_to_bytes()?;

    let path_a2 = out_dir.join("normordis_pdfa2b.pdf");
    std::fs::write(&path_a2, &pdf_a2)?;
    println!(
        "PDF/A-2b:       {} ({} KB)",
        path_a2.display(),
        pdf_a2.len() / 1024
    );

    // ── Opacidade real na marca de água ───────────────────────────────
    let pdf_opacity = DocumentBuilder::new("Rascunho")
        .watermark(
            Watermark::new("RASCUNHO")
                .opacity(0.15)
                .color(RgbColor {
                    r: 0.8,
                    g: 0.0,
                    b: 0.0,
                })
                .font_size(80.0),
        )
        .push(Paragraph::new(
            "Este documento usa opacidade real via ExtGState.",
        ))
        .push(Paragraph::new(
            "A marca de água RASCUNHO é renderizada com alfa 0.15 sem \
             simulação de cor — funciona correctamente sobre qualquer fundo.",
        ))
        .render_to_bytes()?;

    let path_o = out_dir.join("normordis_opacity.pdf");
    std::fs::write(&path_o, &pdf_opacity)?;
    println!(
        "Opacidade real: {} ({} KB)",
        path_o.display(),
        pdf_opacity.len() / 1024
    );

    // ── NDT 2.0.0 ────────────────────────────────────────────────────
    // Este exemplo demonstrava classificação automática a partir de um bloco
    // NDT. O bloco foi retirado porque o renderizador de layout posicionado
    // do NDT 2.0.0 ainda não está implementado — ver
    // src/template/renderer.rs::render_template e o item correspondente em
    // TODO.md. O template NDT 2.0.0 válido continua em
    // examples/templates/relatorio-simples.ndt.json, pronto para quando o
    // renderizador existir; o exemplo 03_ndt_template exercita-o.

    println!("\nChecklist visual:");
    println!("  □ PDF/A: verificar com veraPDF — zero erros");
    println!("  □ PDF/A-2b: marca de água INTERNO translúcida (classif. auto)");
    println!("  □ Marca de água RASCUNHO com opacidade real");
    println!("  □ PDF backend: {}", PDF_BACKEND);

    Ok(())
}
