//! Spike ADR-005 — pergunta 1 do plano em docs/architecture/spike-krilla-plano.md:
//! "Um documento PDF/A-4f gerado via krilla passa `veraPDF --flavour 4f`?"
//!
//! Gera um PDF/A-4f mínimo com krilla 0.8.2, usando o validador
//! `Archival::A4F` confirmado por leitura direta do código-fonte de
//! `crates/krilla/src/configure/validate.rs` (não documentação de segunda
//! mão). O ficheiro de saída é validado em CI por `tools/verify-pdf` +
//! veraPDF — este binário não valida nada por si, só gera.
//!
//! Não faz parte da API pública do normordis-pdf; existe só para este spike.

use std::path::PathBuf;

use krilla::configure::{Archival, ConfigurationBuilder};
use krilla::geom::Point;
use krilla::page::PageSettings;
use krilla::text::{Font, TextDirection};
use krilla::{Document, SerializeSettings};

const OUTPUT_PATH: &str = "/tmp/spike_krilla_a4f.pdf";

fn main() {
    // Fonte já embutida no crate principal — reutilizada para não duplicar
    // ficheiros de licença OFL no repositório só para este spike.
    let font_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fonts/LiberationSans-Regular.ttf");
    let font_data = std::fs::read(&font_path)
        .unwrap_or_else(|e| panic!("não consegui ler {}: {e}", font_path.display()));
    let font = Font::new(font_data.into(), 0).expect("LiberationSans-Regular.ttf inválido");

    let settings = SerializeSettings {
        configuration: ConfigurationBuilder::new()
            .with_archival_validator(Archival::A4F)
            .finish()
            .expect("configuração PDF/A-4f inválida"),
        ..Default::default()
    };

    let mut document = Document::new_with(settings);
    let mut page = document.start_page_with(PageSettings::from_wh(210.0, 297.0).unwrap());
    let mut surface = page.surface();
    surface.draw_text(
        Point::from_xy(20.0, 30.0),
        font,
        14.0,
        "normordis-pdf — spike krilla — PDF/A-4f (ADR-005)",
        false,
        TextDirection::Auto,
    );
    surface.finish();
    page.finish();

    match document.finish() {
        Ok(pdf) => {
            std::fs::write(OUTPUT_PATH, &pdf)
                .unwrap_or_else(|e| panic!("não consegui escrever {OUTPUT_PATH}: {e}"));
            eprintln!(
                "OK — {OUTPUT_PATH} escrito ({} bytes). Validar com: \
                 target/debug/verify-pdf {OUTPUT_PATH} --flavour 4f --pdfa-only",
                pdf.len()
            );
        }
        // Documenta a validação interna do krilla como o Validator::A(_)
        // menciona: erros só aqui significam que o próprio krilla recusou
        // o documento como não-conforme, antes de chegar ao veraPDF.
        Err(errors) => {
            eprintln!("krilla recusou o documento como PDF/A-4f: {errors:?}");
            std::process::exit(1);
        }
    }
}
