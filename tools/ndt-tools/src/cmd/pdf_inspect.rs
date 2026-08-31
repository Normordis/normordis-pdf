use std::path::Path;

use lopdf::{Document, Object};

/// Summary of PDF object sizes by category.
#[derive(Debug, Default)]
pub struct PdfSizeSummary {
    pub total_bytes: usize,
    pub font_bytes: usize,
    pub stream_bytes: usize,
    pub image_bytes: usize,
    pub metadata_bytes: usize,
    pub other_bytes: usize,
    pub object_count: usize,
    pub compressed_streams: usize,
    pub uncompressed_streams: usize,
}

pub fn inspect(path: &Path) -> anyhow::Result<PdfSizeSummary> {
    let doc = Document::load(path)?;
    let mut summary = PdfSizeSummary {
        total_bytes: std::fs::metadata(path)?.len() as usize,
        object_count: doc.objects.len(),
        ..Default::default()
    };

    for object in doc.objects.values() {
        let obj_bytes = estimate_object_bytes(object);
        match classify_object(object) {
            ObjKind::Font => summary.font_bytes += obj_bytes,
            ObjKind::Image => summary.image_bytes += obj_bytes,
            ObjKind::Metadata => summary.metadata_bytes += obj_bytes,
            ObjKind::Stream => {
                summary.stream_bytes += obj_bytes;
                if is_compressed(object) {
                    summary.compressed_streams += 1;
                } else {
                    summary.uncompressed_streams += 1;
                }
            }
            ObjKind::Other => summary.other_bytes += obj_bytes,
        }
    }

    Ok(summary)
}

pub fn print_report(summary: &PdfSizeSummary) {
    println!("PDF Size Report");
    println!("{}", "═".repeat(39));
    println!("Total file size:      {:>8} KB", summary.total_bytes / 1024);
    println!("{}", "─".repeat(39));
    println!(
        "Fonts (embedded):     {:>8} KB  ({:.0}%)",
        summary.font_bytes / 1024,
        pct(summary.font_bytes, summary.total_bytes)
    );
    println!(
        "Content streams:      {:>8} KB  ({:.0}%)",
        summary.stream_bytes / 1024,
        pct(summary.stream_bytes, summary.total_bytes)
    );
    println!(
        "Images:               {:>8} KB  ({:.0}%)",
        summary.image_bytes / 1024,
        pct(summary.image_bytes, summary.total_bytes)
    );
    println!(
        "Metadata:             {:>8} KB  ({:.0}%)",
        summary.metadata_bytes / 1024,
        pct(summary.metadata_bytes, summary.total_bytes)
    );
    println!(
        "Other:                {:>8} KB  ({:.0}%)",
        summary.other_bytes / 1024,
        pct(summary.other_bytes, summary.total_bytes)
    );
    println!("{}", "─".repeat(39));
    println!("Object count:         {:>8}", summary.object_count);
    println!("Compressed streams:   {:>8}", summary.compressed_streams);
    println!(
        "Uncompressed streams: {:>8}  <- target: 0",
        summary.uncompressed_streams
    );

    if summary.uncompressed_streams > 0 {
        println!(
            "\nWARNING: {} uncompressed streams found — run with compression enabled",
            summary.uncompressed_streams
        );
    }
    if summary.font_bytes > 200_000 {
        println!(
            "WARNING: Large font data ({} KB) — font subsetting needed (v2.0.0)",
            summary.font_bytes / 1024
        );
    }
}

fn pct(part: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    part as f64 / total as f64 * 100.0
}

enum ObjKind {
    Font,
    Image,
    Stream,
    Metadata,
    Other,
}

fn classify_object(obj: &Object) -> ObjKind {
    let stream = match obj {
        Object::Stream(s) => s,
        Object::Dictionary(d) => {
            if d.get(b"Type").and_then(|t| t.as_name()).ok() == Some(b"FontDescriptor") {
                return ObjKind::Font;
            }
            return ObjKind::Other;
        }
        _ => return ObjKind::Other,
    };

    let dict = &stream.dict;

    if let Ok(subtype) = dict.get(b"Subtype").and_then(|t| t.as_name())
        && subtype == b"Image"
    {
        return ObjKind::Image;
    }

    if let Ok(ty) = dict.get(b"Type").and_then(|t| t.as_name()) {
        match ty {
            b"Font" | b"FontDescriptor" => return ObjKind::Font,
            b"Metadata" => return ObjKind::Metadata,
            _ => {}
        }
    }

    ObjKind::Stream
}

fn estimate_object_bytes(obj: &Object) -> usize {
    match obj {
        Object::Stream(s) => s.content.len() + 64,
        Object::Dictionary(d) => d.len() * 32,
        Object::String(bytes, _) => bytes.len(),
        _ => 16,
    }
}

fn is_compressed(obj: &Object) -> bool {
    if let Object::Stream(s) = obj {
        s.dict.get(b"Filter").is_ok()
    } else {
        false
    }
}
