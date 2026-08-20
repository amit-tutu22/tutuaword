//! Bibliography sources and citation markers (F16.S3).

use crate::nodes::{Block, Paragraph, Run, RunContent};
use crate::{CitationRef, Document, NodeId};

pub const BIBLIOGRAPHY_TITLE: &str = "Bibliography";

/// One bibliographic source keyed for citation fields.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BibliographySource {
    pub key: String,
    pub author: String,
    pub title: String,
    pub year: String,
}

impl BibliographySource {
    pub fn new(
        key: impl Into<String>,
        author: impl Into<String>,
        title: impl Into<String>,
        year: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            author: author.into(),
            title: title.into(),
            year: year.into(),
        }
    }
}

/// Parenthetical author-year display for an inline citation.
pub fn citation_display(source: &BibliographySource) -> String {
    let surname = source
        .author
        .split(',')
        .next()
        .or_else(|| source.author.split_whitespace().last())
        .unwrap_or(source.author.as_str())
        .trim();
    format!("({surname}, {})", source.year)
}

/// Bibliography entry line: `Author. Title. Year.`
pub fn bibliography_entry_line(source: &BibliographySource) -> String {
    format!("{}. {}. {}.", source.author, source.title, source.year)
}

/// Collect cited source keys in document order (deduplicated).
pub fn cited_source_keys(doc: &Document) -> Vec<String> {
    let mut keys = Vec::new();
    for section in &doc.sections {
        for block in &section.blocks {
            let Some(para) = block.paragraph() else {
                continue;
            };
            for run in &para.runs {
                if let RunContent::CitationRef(cite) = &run.content {
                    if !keys.iter().any(|k| k == &cite.source_key) {
                        keys.push(cite.source_key.clone());
                    }
                }
            }
        }
    }
    keys
}

/// Build bibliography title + entry paragraphs for cited sources.
///
/// When nothing is cited yet, fall back to registered sources so
/// References → Bibliography still produces a usable section.
pub fn build_bibliography_blocks(doc: &Document) -> Vec<Block> {
    let mut keys = cited_source_keys(doc);
    if keys.is_empty() {
        keys = doc
            .bibliography_sources
            .iter()
            .map(|s| s.key.clone())
            .collect();
    }
    if keys.is_empty() {
        return Vec::new();
    }

    let mut blocks = Vec::with_capacity(keys.len() + 1);
    let mut title = Paragraph::with_text(BIBLIOGRAPHY_TITLE);
    title.format.space_after = Some(12.0);
    blocks.push(Block::Paragraph(title));

    for key in keys {
        let Some(source) = doc.bibliography_source_by_key(&key) else {
            continue;
        };
        blocks.push(Block::Paragraph(Paragraph::with_text(bibliography_entry_line(
            source,
        ))));
    }

    blocks
}

impl Document {
    pub fn bibliography_source_by_key(&self, key: &str) -> Option<&BibliographySource> {
        self.bibliography_sources
            .iter()
            .find(|s| s.key.eq_ignore_ascii_case(key))
    }

    pub fn ensure_bibliography_source(&mut self, source: BibliographySource) {
        if self.bibliography_source_by_key(&source.key).is_some() {
            return;
        }
        self.bibliography_sources.push(source);
    }
}

/// Create a citation ref run for a known source.
pub fn citation_ref_run(source: &BibliographySource) -> Run {
    Run {
        id: NodeId::new(),
        format: Default::default(),
        content: RunContent::CitationRef(CitationRef {
            source_key: source.key.clone(),
            display_text: Some(citation_display(source)),
        }),
        revision: None,
    }
}
