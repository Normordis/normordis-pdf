use std::io::{self, Read};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use normordis_pdf::{parse_ndt, serialize_ndt_json, serialize_ndt_toml};

mod cmd {
    pub mod pdf_inspect;
}

#[derive(Parser)]
#[command(
    name = "ndt-tools",
    about = "NDT (NORMAXIS Document Template) utilities"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convert a TOML NDT file to JSON NDT format
    Toml2ndt {
        /// Input TOML file (omit to read stdin)
        #[arg(value_name = "FILE")]
        input: Option<PathBuf>,
        /// Output file (omit to write stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
    /// Convert a JSON NDT file to TOML NDT format
    Ndt2toml {
        /// Input JSON file (omit to read stdin)
        #[arg(value_name = "FILE")]
        input: Option<PathBuf>,
        /// Output file (omit to write stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
    /// Validate an NDT file (JSON or TOML)
    Validate {
        /// Input file (omit to read stdin)
        #[arg(value_name = "FILE")]
        input: Option<PathBuf>,
    },
    /// Pretty-print an NDT file (auto-detects format, outputs JSON)
    Pretty {
        /// Input file (omit to read stdin)
        #[arg(value_name = "FILE")]
        input: Option<PathBuf>,
        /// Output as TOML instead of JSON
        #[arg(long)]
        toml: bool,
    },
    /// Inspect PDF object sizes — diagnose file bloat
    PdfInspect {
        /// Path to the PDF file to inspect
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
}

fn read_input(path: &Option<PathBuf>) -> Result<String, String> {
    match path {
        Some(p) => {
            std::fs::read_to_string(p).map_err(|e| format!("cannot read {}: {e}", p.display()))
        }
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("cannot read stdin: {e}"))?;
            Ok(buf)
        }
    }
}

fn write_output(path: &Option<PathBuf>, content: &str) -> Result<(), String> {
    match path {
        Some(p) => {
            std::fs::write(p, content).map_err(|e| format!("cannot write {}: {e}", p.display()))
        }
        None => {
            print!("{content}");
            Ok(())
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Command::Toml2ndt { input, output } => read_input(input)
            .and_then(|src| parse_ndt(&src).map_err(|e| format!("parse error: {e}")))
            .and_then(|doc| serialize_ndt_json(&doc).map_err(|e| format!("serialize error: {e}")))
            .and_then(|json| write_output(output, &json)),

        Command::Ndt2toml { input, output } => read_input(input)
            .and_then(|src| parse_ndt(&src).map_err(|e| format!("parse error: {e}")))
            .and_then(|doc| serialize_ndt_toml(&doc).map_err(|e| format!("serialize error: {e}")))
            .and_then(|toml_str| write_output(output, &toml_str)),

        Command::Validate { input } => read_input(input)
            .and_then(|src| parse_ndt(&src).map_err(|e| format!("invalid NDT: {e}")))
            .map(|doc| {
                let compat = doc
                    .meta
                    .as_ref()
                    .and_then(|m| m.compat_mode)
                    .map(|v| format!(", compat_mode={v}"))
                    .unwrap_or_default();
                println!(
                    "OK — NDT v{} ({}{})",
                    doc.ndt,
                    detect_format_hint(&doc),
                    compat
                );
            }),

        Command::Pretty { input, toml } => read_input(input)
            .and_then(|src| parse_ndt(&src).map_err(|e| format!("parse error: {e}")))
            .and_then(|doc| {
                if *toml {
                    serialize_ndt_toml(&doc).map_err(|e| format!("serialize error: {e}"))
                } else {
                    serialize_ndt_json(&doc).map_err(|e| format!("serialize error: {e}"))
                }
            })
            .and_then(|out| write_output(&None, &out)),

        Command::PdfInspect { input } => cmd::pdf_inspect::inspect(input)
            .map(|summary| cmd::pdf_inspect::print_report(&summary))
            .map_err(|e| format!("error: {e}")),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn detect_format_hint(doc: &normordis_pdf::NdtDocument) -> &'static str {
    let _ = doc;
    "auto-detected"
}
