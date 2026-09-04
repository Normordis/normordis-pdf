//! Document built from NCRTF rich text JSON.
//! Run: cargo run --example 02_ncrtf_document -p normordis-pdf

use normordis_pdf::*;

const NCRTF_CONTENT: &str = r#"{
  "ncrtf_version": "2.0.0",
  "content": [
    {
      "type": "heading",
      "level": 1,
      "content": [{ "type": "text", "text": "Título do Relatório" }]
    },
    {
      "type": "paragraph",
      "alignment": "justify",
      "content": [
        { "type": "text", "text": "Texto com " },
        { "type": "text", "text": "negrito", "marks": ["bold"] },
        { "type": "text", "text": " e " },
        { "type": "text", "text": "itálico", "marks": ["italic"] },
        { "type": "text", "text": " no mesmo parágrafo." }
      ]
    },
    {
      "type": "list",
      "list_type": "bullet",
      "content": [
        { "type": "list_item", "content": [{ "type": "text", "text": "Primeiro item" }] },
        { "type": "list_item", "content": [{ "type": "text", "text": "Segundo item" }] }
      ]
    }
  ]
}"#;

fn main() -> Result<()> {
    let pdf = DocumentBuilder::new("NCRTF Document")
        .push_ncrtf(NCRTF_CONTENT)?
        .render_to_bytes()?;

    let out = std::env::temp_dir().join("normordis_ncrtf.pdf");
    std::fs::write(&out, &pdf)?;
    println!("PDF gerado: {} ({} bytes)", out.display(), pdf.len());
    Ok(())
}
