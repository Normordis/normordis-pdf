/// Filter criteria for listing NDT documents.
#[derive(Debug, Clone, Default)]
pub struct TemplateFilter {
    pub namespace: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub locale: Option<String>,
}
