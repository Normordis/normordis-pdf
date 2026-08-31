//! verify-pdf — external conformance validator for normordis-pdf output.
//!
//! Wraps veraPDF (https://verapdf.org) to validate a PDF file against:
//!   - PDF/A-4f  (ISO 19005-4, flavour F — allows embedded files)
//!   - PDF/UA-2  (ISO 14289-2:2024)
//!
//! veraPDF must be installed separately and available as `verapdf` in PATH.
//! Download: https://verapdf.org/software/
//!
//! Usage:
//!   verify-pdf <file.pdf> [options]
//!
//! Options:
//!   --pdfa-only     Skip PDF/UA-2 validation
//!   --ua-only       Skip PDF/A-4f validation
//!   --flavour <f>   PDF/A flavour: 1a,1b,2a,2b,2u,3a,3b,3u,4,4e,4f (default: 4f)
//!   --verapdf <p>   Path to verapdf executable (default: verapdf in PATH)
//!   --json          Output machine-readable JSON to stdout

use std::{
    path::{Path, PathBuf},
    process::{Command, exit},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        exit(0);
    }

    let file = PathBuf::from(&args[1]);
    if !file.exists() {
        eprintln!("error: file not found: {}", file.display());
        exit(1);
    }
    if file.extension().and_then(|e| e.to_str()) != Some("pdf") {
        eprintln!("warning: file does not have .pdf extension");
    }

    let validate_pdfa = !has_flag(&args, "--ua-only");
    let validate_ua = !has_flag(&args, "--pdfa-only");
    let json_output = has_flag(&args, "--json");
    let flavour = flag_value(&args, "--flavour").unwrap_or_else(|| "4f".to_string());
    let verapdf_bin = flag_value(&args, "--verapdf").unwrap_or_else(|| "verapdf".to_string());

    // Check veraPDF is reachable before doing anything else.
    match Command::new(&verapdf_bin).arg("--version").output() {
        Err(_) => {
            eprintln!("error: '{}' not found.", verapdf_bin);
            eprintln!("  Install veraPDF: https://verapdf.org/software/");
            eprintln!("  Or specify path: verify-pdf <file> --verapdf /path/to/verapdf");
            exit(2);
        }
        Ok(out) => {
            let version = String::from_utf8_lossy(&out.stdout);
            if !json_output {
                print!("veraPDF {}", version.lines().next().unwrap_or("(unknown version)"));
            }
        }
    }

    let mut results: Vec<ProfileResult> = Vec::new();

    if validate_pdfa {
        if !json_output {
            println!("\nValidating PDF/A-{} ...", flavour.to_uppercase());
        }
        let r = run_verapdf_pdfa(&verapdf_bin, &file, &flavour);
        results.push(r);
    }

    if validate_ua {
        if !json_output {
            println!("Validating PDF/UA-2 (ISO 14289-2) ...");
        }
        let r = run_verapdf_ua2(&verapdf_bin, &file);
        results.push(r);
    }

    let any_failed = results.iter().any(|r| !r.compliant || r.error.is_some());

    if json_output {
        print_json(&results, any_failed);
    } else {
        print_human(&results);
        if any_failed {
            println!("\n✗ FALHOU — o ficheiro não está conforme.");
        } else {
            println!("\n✓ PASSOU — o ficheiro está conforme com todos os perfis validados.");
        }
    }

    exit(if any_failed { 1 } else { 0 });
}

// ── veraPDF invocations ───────────────────────────────────────────────────────

fn run_verapdf_pdfa(bin: &str, file: &Path, flavour: &str) -> ProfileResult {
    let output = Command::new(bin)
        .args(["--flavour", flavour, "--format", "mrr"])
        .arg(file)
        .output();

    match output {
        Err(e) => ProfileResult::error(format!("PDF/A-{}", flavour.to_uppercase()), e.to_string()),
        Ok(out) => {
            let xml = String::from_utf8_lossy(&out.stdout).to_string();
            let mut r = parse_mrr(&xml);
            r.profile = format!("PDF/A-{}", flavour.to_uppercase());
            r
        }
    }
}

fn run_verapdf_ua2(bin: &str, file: &Path) -> ProfileResult {
    // veraPDF 1.30+: -f ua2 validates PDF/UA-2 (ISO 14289-2:2024).
    let output = Command::new(bin)
        .args(["-f", "ua2", "--format", "mrr"])
        .arg(file)
        .output();

    match output {
        Err(e) => ProfileResult::error("PDF/UA-2".to_string(), e.to_string()),
        Ok(out) => {
            let xml = String::from_utf8_lossy(&out.stdout).to_string();
            let mut r = parse_mrr(&xml);
            r.profile = "PDF/UA-2".to_string();
            r
        }
    }
}

// ── MRR parsing ───────────────────────────────────────────────────────────────

/// Result for a single validation profile.
struct ProfileResult {
    profile: String,
    compliant: bool,
    failed_rules: u32,
    passed_rules: u32,
    failed_checks: u32,
    passed_checks: u32,
    failures: Vec<RuleFailure>,
    error: Option<String>,
}

struct RuleFailure {
    specification: String,
    clause: String,
    test_number: String,
    description: Option<String>,
}

impl ProfileResult {
    fn error(profile: String, msg: String) -> Self {
        Self {
            profile,
            compliant: false,
            failed_rules: 0,
            passed_rules: 0,
            failed_checks: 0,
            passed_checks: 0,
            failures: vec![],
            error: Some(msg),
        }
    }
}

/// Parse veraPDF Machine Readable Report XML.
/// Uses simple string scanning — no XML parser dependency.
fn parse_mrr(xml: &str) -> ProfileResult {
    let compliant = xml.contains("isCompliant=\"true\"");
    let failed_rules = attr_u32(xml, "failedRules");
    let passed_rules = attr_u32(xml, "passedRules");
    let failed_checks = attr_u32(xml, "failedChecks");
    let passed_checks = attr_u32(xml, "passedChecks");

    let mut failures: Vec<RuleFailure> = Vec::new();
    // MRR embeds failed rules as: <rule specification="..." clause="..." testNumber="N" ...>
    if !compliant {
        for chunk in xml.split("<rule ") {
            if !chunk.contains("status=\"failed\"") {
                continue;
            }
            let specification = attr_str(chunk, "specification").unwrap_or_default().to_string();
            let clause = attr_str(chunk, "clause").unwrap_or_default().to_string();
            let test_number = attr_str(chunk, "testNumber").unwrap_or_default().to_string();
            // Description may appear in a child <description> element.
            let description = between(chunk, "<description>", "</description>")
                .map(|s| s.trim().to_string());
            failures.push(RuleFailure { specification, clause, test_number, description });
        }
    }

    ProfileResult {
        profile: String::new(),
        compliant,
        failed_rules,
        passed_rules,
        failed_checks,
        passed_checks,
        failures,
        error: None,
    }
}

// ── Output ────────────────────────────────────────────────────────────────────

fn print_human(results: &[ProfileResult]) {
    for r in results {
        if let Some(ref e) = r.error {
            println!("  {} → ERRO: {}", r.profile, e);
            continue;
        }
        if r.compliant {
            println!(
                "  {} → PASSOU  ({} regras, {} verificações)",
                r.profile, r.passed_rules, r.passed_checks
            );
        } else {
            println!(
                "  {} → FALHOU  ({} regras falhadas, {} verificações falhadas)",
                r.profile, r.failed_rules, r.failed_checks
            );
            for f in &r.failures {
                print!("      [{}] §{} teste {}", f.specification, f.clause, f.test_number);
                if let Some(ref d) = f.description {
                    print!(" — {}", d);
                }
                println!();
            }
        }
    }
}

fn print_json(results: &[ProfileResult], any_failed: bool) {
    println!("{{");
    println!("  \"compliant\": {},", !any_failed);
    println!("  \"profiles\": [");
    for (i, r) in results.iter().enumerate() {
        let comma = if i + 1 < results.len() { "," } else { "" };
        if let Some(ref e) = r.error {
            println!(
                "    {{\"profile\":\"{}\",\"compliant\":false,\"error\":\"{}\"}}{}",
                r.profile,
                e.replace('"', "\\\""),
                comma
            );
        } else {
            println!(
                "    {{\"profile\":\"{}\",\"compliant\":{},\"failedRules\":{},\
                 \"passedRules\":{},\"failedChecks\":{},\"passedChecks\":{},\"failures\":[",
                r.profile,
                r.compliant,
                r.failed_rules,
                r.passed_rules,
                r.failed_checks,
                r.passed_checks
            );
            for (j, f) in r.failures.iter().enumerate() {
                let fc = if j + 1 < r.failures.len() { "," } else { "" };
                let desc = f
                    .description
                    .as_deref()
                    .unwrap_or("")
                    .replace('"', "\\\"");
                println!(
                    "      {{\"specification\":\"{}\",\"clause\":\"{}\",\
                     \"testNumber\":\"{}\",\"description\":\"{}\"}}{}",
                    f.specification, f.clause, f.test_number, desc, fc
                );
            }
            println!("    ]}}{}",  comma);
        }
    }
    println!("  ]");
    println!("}}");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn print_usage() {
    println!("verify-pdf — validador de conformidade PDF/A-4f + PDF/UA-2");
    println!();
    println!("Uso:");
    println!("  verify-pdf <ficheiro.pdf> [opções]");
    println!();
    println!("Opções:");
    println!("  --pdfa-only        Valida apenas PDF/A (ignora PDF/UA-2)");
    println!("  --ua-only          Valida apenas PDF/UA-2 (ignora PDF/A)");
    println!("  --flavour <f>      Sabor PDF/A: 1a,1b,2a,2b,3a,3b,4,4e,4f (default: 4f)");
    println!("  --verapdf <path>   Caminho para o executável veraPDF (default: 'verapdf' em PATH)");
    println!("  --json             Output em JSON (para integração CI/CD)");
    println!();
    println!("Requisito:");
    println!("  veraPDF 2.x instalado — https://verapdf.org/software/");
    println!();
    println!("Exit codes:");
    println!("  0 — conforme com todos os perfis");
    println!("  1 — não conforme (falhou pelo menos um perfil)");
    println!("  2 — veraPDF não encontrado");
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn attr_u32(xml: &str, attr: &str) -> u32 {
    attr_str(xml, attr)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn attr_str<'a>(text: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{attr}=\"");
    text.find(needle.as_str()).and_then(|pos| {
        let rest = &text[pos + needle.len()..];
        rest.find('"').map(|end| &rest[..end])
    })
}

fn between<'a>(text: &'a str, open: &str, close: &str) -> Option<&'a str> {
    text.find(open).and_then(|s| {
        let rest = &text[s + open.len()..];
        rest.find(close).map(|e| &rest[..e])
    })
}
