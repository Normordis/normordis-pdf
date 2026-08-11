use std::path::PathBuf;

use clap::Parser;
use dotx2ndt::element_mapper::{MappingContext, OutputMode, map_body_from_xml};
use dotx2ndt::extractor::DotxExtractor;
use dotx2ndt::numbering_mapper::parse_numbering_xml;
use dotx2ndt::placeholder::build_ndt_output;
use dotx2ndt::style_mapper::{parse_styles_xml, to_ndt_styles};

/// dotx2ndt — Convert Word .docx / .dotx to a NORMORDIS NDT JSON document or template.
///
/// Examples:
///
///   dotx2ndt report.docx
///
///   dotx2ndt contract.dotx --mode template --title "Contract Template"
///
///   dotx2ndt report.docx -o out/report.ndt.json --mode document
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input .docx or .dotx file.
    input: PathBuf,

    /// Output JSON file path. Defaults to <input-stem>.ndt.json.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Document title written to the NDT meta block.
    #[arg(short, long, default_value = "Untitled")]
    title: String,

    /// Output mode: `document` (preserve content) or `template` (replace text with placeholders).
    #[arg(long, default_value = "document", value_parser = ["document", "template"])]
    mode: String,

    /// Pretty-print the JSON output (default: true).
    #[arg(short, long, default_value_t = true)]
    pretty: bool,

    /// Print diagnostic information (style count, compat mode, element counts).
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let output_path = args.output.unwrap_or_else(|| {
        let stem = args.input.file_stem().unwrap_or_default().to_string_lossy();
        args.input.with_file_name(format!("{}.ndt.json", stem))
    });

    let output_mode = if args.mode == "template" {
        OutputMode::Template
    } else {
        OutputMode::Document
    };

    // ── Extract ───────────────────────────────────────────────────────────────

    let extractor = match DotxExtractor::from_file(&args.input) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", args.input.display(), e);
            std::process::exit(1);
        }
    };

    // ── Styles ────────────────────────────────────────────────────────────────

    let styles_xml = match extractor.styles_xml.as_deref() {
        Some(xml) => xml.to_string(),
        None => {
            eprintln!(
                "error: word/styles.xml not found in {}",
                args.input.display()
            );
            std::process::exit(1);
        }
    };

    let word_styles = match parse_styles_xml(&styles_xml) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot parse styles.xml: {}", e);
            std::process::exit(1);
        }
    };

    let compat_mode = extractor.extract_compat_mode();
    let ndt_styles = to_ndt_styles(&word_styles);

    // Build styleId → NDT slug map for body rendering
    let style_id_map: std::collections::HashMap<String, String> = word_styles
        .iter()
        .map(|ws| {
            let slug = ndt_styles
                .keys()
                .find(|k| {
                    k.replace('_', " ")
                        .eq_ignore_ascii_case(&ws.name.replace('_', " "))
                })
                .cloned()
                .unwrap_or_else(|| slugify_name(&ws.name));
            (ws.id.clone(), slug)
        })
        .collect();

    // ── Numbering ─────────────────────────────────────────────────────────────

    let numbering_types = extractor
        .numbering_xml
        .as_deref()
        .map(parse_numbering_xml)
        .unwrap_or_default();

    // ── Body ──────────────────────────────────────────────────────────────────

    let mut ctx = MappingContext::default();
    ctx.style_id_map = style_id_map;
    ctx.numbering_types = numbering_types;
    ctx.media = extractor.media;
    ctx.relationships = extractor.relationships;
    ctx.mode = output_mode;

    let body = match map_body_from_xml(&extractor.document_xml, &mut ctx) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot parse document.xml: {}", e);
            std::process::exit(1);
        }
    };

    // ── Assemble ──────────────────────────────────────────────────────────────

    let ndt = build_ndt_output(
        &ndt_styles,
        &args.title,
        compat_mode,
        body.clone(),
        ctx.placeholders,
    );

    // ── Verbose output ────────────────────────────────────────────────────────

    if args.verbose {
        eprintln!("styles:  {}", ndt_styles.len());
        eprintln!(
            "compat:  {}",
            compat_mode
                .map(|v| format!("{v}"))
                .unwrap_or_else(|| "not found".into())
        );
        eprintln!("elements: {}", body.len());
        eprintln!("mode:    {}", args.mode);
    } else {
        eprintln!(
            "{} style(s), {} element(s) → {}",
            ndt_styles.len(),
            body.len(),
            output_path.display()
        );
    }

    // ── Write ─────────────────────────────────────────────────────────────────

    let json_str = if args.pretty {
        serde_json::to_string_pretty(&ndt).unwrap()
    } else {
        serde_json::to_string(&ndt).unwrap()
    };

    if let Err(e) = std::fs::write(&output_path, &json_str) {
        eprintln!("error: cannot write {}: {}", output_path.display(), e);
        std::process::exit(1);
    }

    println!("{}", output_path.display());
}

/// Converts a style name to a lowercase_underscore slug.
fn slugify_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c.to_lowercase().next().unwrap()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
