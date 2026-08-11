//! Document Inspector findings (F22.S3).
//!
//! Categories: comments, document metadata, hidden (`w:vanish`) text.

use serde::{Deserialize, Serialize};

use crate::{Block, Document, Paragraph, RunContent, Table};

/// Inspectable privacy category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectCategory {
    Comments,
    Metadata,
    HiddenText,
}

/// One category finding with a count of items the inspector can remove.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentInspectFinding {
    pub category: InspectCategory,
    pub count: u32,
    pub message: String,
}

/// Scan the document for Inspect Document findings (F22.S3).
pub fn inspect_document(doc: &Document) -> Vec<DocumentInspectFinding> {
    let mut findings = Vec::new();

    let comment_count = count_comments(doc);
    if comment_count > 0 {
        findings.push(DocumentInspectFinding {
            category: InspectCategory::Comments,
            count: comment_count,
            message: format!(
                "{comment_count} comment{}",
                if comment_count == 1 { "" } else { "s" }
            ),
        });
    }

    let metadata_count = count_metadata(doc);
    if metadata_count > 0 {
        findings.push(DocumentInspectFinding {
            category: InspectCategory::Metadata,
            count: metadata_count,
            message: format!(
                "{metadata_count} document propert{}",
                if metadata_count == 1 { "y" } else { "ies" }
            ),
        });
    }

    let hidden_count = count_hidden_runs(doc);
    if hidden_count > 0 {
        findings.push(DocumentInspectFinding {
            category: InspectCategory::HiddenText,
            count: hidden_count,
            message: format!(
                "{hidden_count} hidden text run{}",
                if hidden_count == 1 { "" } else { "s" }
            ),
        });
    }

    findings
}

fn count_comments(doc: &Document) -> u32 {
    let threads = doc.comments.len() as u32;
    let mut refs = 0u32;
    for section in &doc.sections {
        count_comment_refs_in_blocks(&section.blocks, &mut refs);
        for hf in section.headers.values() {
            count_comment_refs_in_blocks(&hf.blocks, &mut refs);
        }
        for hf in section.footers.values() {
            count_comment_refs_in_blocks(&hf.blocks, &mut refs);
        }
    }
    threads.max(refs)
}

fn count_comment_refs_in_blocks(blocks: &[Block], count: &mut u32) {
    for block in blocks {
        match block {
            Block::Paragraph(para) => {
                for run in &para.runs {
                    if matches!(run.content, RunContent::CommentRef(_)) {
                        *count += 1;
                    }
                }
            }
            Block::Table(table) => count_comment_refs_in_table(table, count),
            _ => {}
        }
    }
}

fn count_comment_refs_in_table(table: &Table, count: &mut u32) {
    for row in &table.rows {
        for cell in &row.cells {
            count_comment_refs_in_blocks(&cell.blocks, count);
        }
    }
}

fn count_metadata(doc: &Document) -> u32 {
    let mut n = 0u32;
    if doc
        .properties
        .title
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some()
    {
        n += 1;
    }
    if doc
        .properties
        .author
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some()
    {
        n += 1;
    }
    n
}

fn count_hidden_runs(doc: &Document) -> u32 {
    let mut count = 0u32;
    for section in &doc.sections {
        count_hidden_in_blocks(&section.blocks, &mut count);
        for hf in section.headers.values() {
            count_hidden_in_blocks(&hf.blocks, &mut count);
        }
        for hf in section.footers.values() {
            count_hidden_in_blocks(&hf.blocks, &mut count);
        }
    }
    count
}

fn count_hidden_in_blocks(blocks: &[Block], count: &mut u32) {
    for block in blocks {
        match block {
            Block::Paragraph(para) => count_hidden_in_paragraph(para, count),
            Block::Table(table) => {
                for row in &table.rows {
                    for cell in &row.cells {
                        count_hidden_in_blocks(&cell.blocks, count);
                    }
                }
            }
            _ => {}
        }
    }
}

fn count_hidden_in_paragraph(para: &Paragraph, count: &mut u32) {
    for run in &para.runs {
        if run.format.hidden == Some(true) {
            *count += 1;
        }
    }
}
