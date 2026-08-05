use tw_model::NodeId;

use crate::policy::DataClassification;

#[derive(Debug, Clone, Default)]
pub struct DocumentContext {
    pub selection: Option<ContextSelection>,
    pub surrounding_paragraphs: Vec<ParagraphSummary>,
    pub document_metadata: DocumentMetadata,
    pub total_token_estimate: u32,
    pub page_count: u32,
    pub classification: Option<DataClassification>,
}

#[derive(Debug, Clone)]
pub struct ContextSelection {
    pub text: String,
    pub char_format_summary: String,
    pub paragraph_style: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParagraphSummary {
    pub style: Option<String>,
    pub text_preview: String,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    pub title: String,
    pub doc_id: NodeId,
}
