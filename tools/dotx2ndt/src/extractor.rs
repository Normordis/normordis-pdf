use std::io::Read;
use std::path::Path;

use crate::error::Dotx2NdtError;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// All XML streams extracted from a .dotx (or .docx) ZIP archive.
pub struct DotxExtractor {
    pub document_xml: String,
    pub styles_xml: Option<String>,
    pub numbering_xml: Option<String>,
    pub settings_xml: Option<String>,
}

impl Default for DotxExtractor {
    fn default() -> Self {
        Self {
            document_xml: String::new(),
            styles_xml: None,
            numbering_xml: None,
            settings_xml: None,
        }
    }
}

impl DotxExtractor {
    /// Opens a .dotx file and extracts all known XML streams from it.
    pub fn from_file(path: &Path) -> Result<Self, Dotx2NdtError> {
        let file = std::fs::File::open(path)
            .map_err(|e| Dotx2NdtError::Io(format!("cannot open {}: {}", path.display(), e)))?;

        let mut zip = zip::ZipArchive::new(file)
            .map_err(|e| Dotx2NdtError::Zip(format!("{e}")))?;

        let document_xml = Self::read_zip_entry(&mut zip, "word/document.xml")
            .unwrap_or_default();

        let styles_xml = Self::read_zip_entry(&mut zip, "word/styles.xml").ok();
        let numbering_xml = Self::read_zip_entry(&mut zip, "word/numbering.xml").ok();
        let settings_xml = Self::read_zip_entry(&mut zip, "word/settings.xml").ok();

        Ok(Self { document_xml, styles_xml, numbering_xml, settings_xml })
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

    fn read_zip_entry(
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
