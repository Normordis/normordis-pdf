//! Demonstrates loading all fonts from a directory via fonts_from_dir().
//! Run: cargo run --example 15_fonts_from_dir

use normordis_pdf::*;

fn main() -> Result<()> {
    // Loads every TTF/OTF file from assets/fonts/, grouping them by family.
    let pdf = DocumentBuilder::new("Fontes de Directório")
        .fonts_from_dir("assets/fonts/")?
        .push(Section::new("Fontes Carregadas do Directório", 1))
        .push(Paragraph::new("Texto com LiberationSans (default após load_dir)."))
        .push(Paragraph::new("Texto com LiberationSerif.")
            .font_family("LiberationSerif"))
        .push(Paragraph::new("Texto com LiberationMono.")
            .font_family("LiberationMono"))
        .push(Spacer::new(4.0))
        .push(Paragraph::new("Alias Word: 'Arial' → LiberationSans.")
            .font_family("Arial"))
        .push(Paragraph::new("Alias Word: 'Times New Roman' → LiberationSerif.")
            .font_family("Times New Roman"))
        .render_to_bytes()?;

    let out = std::env::temp_dir().join("normordis_fonts_dir.pdf");
    std::fs::write(&out, &pdf)?;
    println!("✓ {} ({} KB)", out.display(), pdf.len() / 1024);
    Ok(())
}
