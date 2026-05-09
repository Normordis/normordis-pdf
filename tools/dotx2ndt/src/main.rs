use dotx2ndt::extractor::DotxExtractor;
use dotx2ndt::placeholder::build_ndt_skeleton;
use dotx2ndt::style_mapper::{parse_styles_xml, to_ndt_styles};

use std::path::PathBuf;

use clap::Parser;

/// dotx2ndt — Extract Word .dotx paragraph styles to a NORMAXIS NDT named-styles JSON skeleton.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input .dotx file.
    input: PathBuf,

    /// Output JSON file path. Defaults to <input>.ndt.json.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Document title for the NDT skeleton.
    #[arg(short, long, default_value = "Untitled")]
    title: String,

    /// Pretty-print the JSON output.
    #[arg(short, long, default_value_t = true)]
    pretty: bool,

    /// Print extra information (compat mode, style count, etc.).
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let output_path = args.output.unwrap_or_else(|| {
        let stem = args.input.file_stem().unwrap_or_default().to_string_lossy();
        args.input.with_file_name(format!("{}.ndt.json", stem))
    });

    let extractor = match DotxExtractor::from_file(&args.input) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading {}: {}", args.input.display(), e);
            std::process::exit(1);
        }
    };

    let styles_xml = match extractor.styles_xml.as_deref() {
        Some(xml) => xml.to_string(),
        None => {
            eprintln!("Error: word/styles.xml not found in {}", args.input.display());
            std::process::exit(1);
        }
    };

    let word_styles = match parse_styles_xml(&styles_xml) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error parsing styles.xml: {}", e);
            std::process::exit(1);
        }
    };

    let compat_mode = extractor.extract_compat_mode();

    if args.verbose {
        eprintln!("Found {} paragraph style(s).", word_styles.len());
        match compat_mode {
            Some(16) => eprintln!("Compat mode: {} (Word 2016+)", 16),
            Some(15) => eprintln!("Compat mode: {} (Word 2013)", 15),
            Some(14) => eprintln!("Compat mode: {} (Word 2010)", 14),
            Some(12) => eprintln!("Compat mode: {} (Word 2007)", 12),
            Some(v)  => eprintln!("Compat mode: {}", v),
            None     => eprintln!("Compat mode: not found"),
        }
    } else {
        eprintln!("Found {} paragraph style(s).", word_styles.len());
    }

    let ndt_styles = to_ndt_styles(&word_styles);
    let skeleton = build_ndt_skeleton(&ndt_styles, &args.title, compat_mode);

    let json_str = if args.pretty {
        serde_json::to_string_pretty(&skeleton).unwrap()
    } else {
        serde_json::to_string(&skeleton).unwrap()
    };

    if let Err(e) = std::fs::write(&output_path, &json_str) {
        eprintln!("Error writing {}: {}", output_path.display(), e);
        std::process::exit(1);
    }

    println!("Written {} styles to {}", ndt_styles.len(), output_path.display());
}
