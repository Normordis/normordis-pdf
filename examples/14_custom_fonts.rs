//! Demonstrates loading and using custom fonts via register_bytes / register_file.
//! Run: cargo run --example 14_custom_fonts

use normordis_pdf::*;

fn main() -> Result<()> {
    // ── Method 1: font_from_bytes (embedded via include_bytes!) ───────────────
    // Re-register Liberation Serif under a custom name to demonstrate the API.
    let pdf = DocumentBuilder::new("Fontes Personalizadas")
        .font_from_bytes(
            "SerifDemo",
            include_bytes!("../assets/fonts/LiberationSerif-Regular.ttf"),
            Some(include_bytes!("../assets/fonts/LiberationSerif-Bold.ttf")),
            Some(include_bytes!("../assets/fonts/LiberationSerif-Italic.ttf")),
            Some(include_bytes!(
                "../assets/fonts/LiberationSerif-BoldItalic.ttf"
            )),
        )?
        .font_from_bytes(
            "MonoDemo",
            include_bytes!("../assets/fonts/LiberationMono-Regular.ttf"),
            None,
            None,
            None,
        )?
        .push(Section::new("1. Fontes Embebidas (Liberation)", 1))
        .push(Paragraph::new("Este texto usa LiberationSans (default)."))
        .push(Paragraph::new("Este texto usa LiberationSerif.").font_family("LiberationSerif"))
        .push(Paragraph::new("Este texto usa LiberationMono.").font_family("LiberationMono"))
        .push(Spacer::new(4.0))
        .push(Section::new("2. Fontes Registadas Dinamicamente", 1))
        .push(
            Paragraph::new("Este texto usa SerifDemo (Liberation Serif re-registado).")
                .font_family("SerifDemo"),
        )
        .push(
            Paragraph::new("Este texto usa MonoDemo (Liberation Mono re-registado).")
                .font_family("MonoDemo"),
        )
        .push(Spacer::new(4.0))
        .push(Section::new("3. Fallback de Fonte", 1))
        .push(
            Paragraph::new("Fonte inexistente → fallback automático para a chain configurada.")
                .font_family("FonteQueNaoExiste"),
        )
        .render_to_bytes()?;

    let out = std::env::temp_dir().join("normordis_custom_fonts.pdf");
    std::fs::write(&out, &pdf)?;
    println!("✓ PDF: {} ({} KB)", out.display(), pdf.len() / 1024);
    println!("  Verificar:");
    println!("  □ Texto legível com todas as fontes");
    println!("  □ Serif visivelmente diferente do Sans");
    println!("  □ Mono com largura fixa por carácter");
    Ok(())
}
