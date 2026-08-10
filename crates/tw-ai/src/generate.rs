//! Content generation into a new document (F28.S4).

use tw_model::{Block, Document, Paragraph};

use crate::context::DocumentContext;
use crate::provider::AiError;
use crate::service::{AiResponse, AiService};

/// Kinds of generated documents supported in F28.S4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    Outline,
    Minutes,
    Report,
}

impl ContentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ContentKind::Outline => "outline",
            ContentKind::Minutes => "minutes",
            ContentKind::Report => "report",
        }
    }

    pub fn prompt_instruction(self, topic: &str) -> String {
        match self {
            ContentKind::Outline => format!(
                "Generate a hierarchical markdown outline for: {topic}. \
                 Use # for title and ## for sections. Keep it concise."
            ),
            ContentKind::Minutes => format!(
                "Generate meeting minutes in markdown for: {topic}. \
                 Include # title, ## Attendees, ## Discussion, ## Action items."
            ),
            ContentKind::Report => format!(
                "Generate a short report in markdown for: {topic}. \
                 Include # title, ## Summary, ## Findings, ## Recommendations."
            ),
        }
    }
}

/// Result of generating content into a fresh document model.
#[derive(Debug, Clone)]
pub struct GeneratedDocument {
    pub kind: ContentKind,
    pub topic: String,
    pub markdown: String,
    pub document: Document,
}

impl GeneratedDocument {
    pub fn paragraph_count(&self) -> usize {
        self.document
            .sections
            .iter()
            .flat_map(|s| s.blocks.iter())
            .filter(|b| matches!(b, Block::Paragraph(_)))
            .count()
    }

    pub fn plain_text(&self) -> String {
        self.document
            .sections
            .iter()
            .flat_map(|s| s.blocks.iter())
            .filter_map(|b| b.paragraph())
            .map(|p| p.full_text())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Parse AI markdown into a [`Document`] with Heading 1 / Heading 2 styles.
pub fn document_from_markdown(markdown: &str) -> Document {
    let mut doc = Document::new();
    let mut blocks = Vec::new();
    for raw in markdown.lines() {
        let line = raw.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        let (style_name, text) = classify_markdown_line(line);
        let mut para = Paragraph::with_text(text);
        if let Some(name) = style_name {
            if let Some(style) = doc.styles.find_style_by_name(name) {
                para.style_id = Some(style.id);
            }
        }
        blocks.push(Block::Paragraph(para));
    }
    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }
    if let Some(section) = doc.sections.first_mut() {
        section.blocks = blocks;
    }
    doc
}

fn classify_markdown_line(line: &str) -> (Option<&'static str>, String) {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("### ") {
        return (Some("Heading 3"), rest.trim().to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("## ") {
        return (Some("Heading 2"), rest.trim().to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("# ") {
        return (Some("Heading 1"), rest.trim().to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("- ") {
        return (None, format!("• {rest}"));
    }
    if let Some(rest) = trimmed.strip_prefix("* ") {
        return (None, format!("• {rest}"));
    }
    (None, trimmed.to_string())
}

/// Call [`AiService::generate`] and materialize a new document.
pub fn generate_content(
    service: &impl AiService,
    kind: ContentKind,
    topic: &str,
    ctx: &DocumentContext,
) -> Result<GeneratedDocument, AiError> {
    let instruction = kind.prompt_instruction(topic);
    let response = service.generate(ctx, &instruction)?;
    let markdown = match response {
        AiResponse::TextSuggestion { text } => text,
        _ => {
            return Err(AiError::CompletionFailed(
                "generate did not return text".into(),
            ))
        }
    };
    let document = document_from_markdown(&markdown);
    Ok(GeneratedDocument {
        kind,
        topic: topic.to_string(),
        markdown,
        document,
    })
}
