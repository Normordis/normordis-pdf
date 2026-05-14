use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::error::Dotx2NdtError;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// All content extracted from a .docx / .dotx ZIP archive.
pub struct DotxExtractor {
    pub document_xml: String,
    pub styles_xml: Option<String>,
    pub numbering_xml: Option<String>,
    pub settings_xml: Option<String>,
    /// rId → target path relative to `word/` (e.g. `"media/image1.png"`).
    pub relationships: HashMap<String, String>,
    /// Media files keyed by path relative to `word/` (e.g. `"media/image1.png"`).
    pub media: HashMap<String, Vec<u8>>,
}

impl Default for DotxExtractor {
    fn default() -> Self {
        Self {
            document_xml: String::new(),
            styles_xml: None,
            numbering_xml: None,
            settings_xml: None,
            relationships: HashMap::new(),
            media: HashMap::new(),
        }
    }
}

impl DotxExtractor {
    /// Opens a .docx / .dotx file and extracts all known content from it.
    pub fn from_file(path: &Path) -> Result<Self, Dotx2NdtError> {
        let file = std::fs::File::open(path)
            .map_err(|e| Dotx2NdtError::Io(format!("cannot open {}: {}", path.display(), e)))?;

        let mut zip = zip::ZipArchive::new(file).map_err(|e| Dotx2NdtError::Zip(format!("{e}")))?;

        // Named XML entries
        let document_xml = Self::read_zip_text(&mut zip, "word/document.xml").unwrap_or_default();
        let styles_xml = Self::read_zip_text(&mut zip, "word/styles.xml").ok();
        let numbering_xml = Self::read_zip_text(&mut zip, "word/numbering.xml").ok();
        let settings_xml = Self::read_zip_text(&mut zip, "word/settings.xml").ok();
        let rels_xml = Self::read_zip_text(&mut zip, "word/_rels/document.xml.rels").ok();

        let relationships = rels_xml
            .as_deref()
            .map(parse_relationships)
            .unwrap_or_default();

        // Media files: iterate all entries
        let mut media: HashMap<String, Vec<u8>> = HashMap::new();
        let count = zip.len();
        for i in 0..count {
            let mut entry = match zip.by_index(i) {
                Ok(e) => e,
                Err(_) => continue,
            };
            let name = entry.name().to_string();
            if !name.starts_with("word/media/") {
                continue;
            }
            let rel_path = name.strip_prefix("word/").unwrap_or(&name).to_string();
            let mut bytes = Vec::new();
            if entry.read_to_end(&mut bytes).is_ok() {
                media.insert(rel_path, bytes);
            }
        }

        Ok(Self {
            document_xml,
            styles_xml,
            numbering_xml,
            settings_xml,
            relationships,
            media,
        })
    }

    /// Returns the Word compatibility mode value from `word/settings.xml`, if present.
    ///
    /// Reads `w:compatSetting[@w:name="compatibilityMode"]/@w:val`.
    /// Known values: 12=Word2007, 14=Word2010, 15=Word2013, 16=Word2016+
    pub fn extract_compat_mode(&self) -> Option<u32> {
        let xml = self.settings_xml.as_deref()?;
        let doc = roxmltree::Document::parse(xml).ok()?;
        for node in doc.descendants() {
            if node.tag_name().name() != "compatSetting" {
                continue;
            }
            let name = node.attribute((W_NS, "name")).unwrap_or("");
            if name == "compatibilityMode" {
                let val = node.attribute((W_NS, "val"))?;
                return val.parse().ok();
            }
        }
        None
    }

    fn read_zip_text(
        zip: &mut zip::ZipArchive<std::fs::File>,
        name: &str,
    ) -> Result<String, Dotx2NdtError> {
        let mut entry = zip
            .by_name(name)
            .map_err(|_| Dotx2NdtError::MissingEntry(name.into()))?;
        let mut contents = String::new();
        entry
            .read_to_string(&mut contents)
            .map_err(|e| Dotx2NdtError::Io(format!("read {name}: {e}")))?;
        Ok(contents)
    }
}

/// Parses `word/_rels/document.xml.rels` and returns rId → target-path map.
///
/// Targets are relative to `word/`, so `"media/image1.png"` can be looked up
/// directly in [`DotxExtractor::media`].
fn parse_relationships(xml: &str) -> HashMap<String, String> {
    let doc = match roxmltree::Document::parse(xml) {
        Ok(d) => d,
        Err(_) => return HashMap::new(),
    };
    let mut map = HashMap::new();
    for node in doc.descendants() {
        if node.tag_name().name() != "Relationship" {
            continue;
        }
        if let (Some(id), Some(target)) = (node.attribute("Id"), node.attribute("Target")) {
            // Targets may start with "media/..." or "../media/..."
            let clean = target.trim_start_matches("../");
            map.insert(id.to_string(), clean.to_string());
        }
    }
    map
}
