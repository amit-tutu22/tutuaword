//! Bookmark targets and REF cross-reference helpers (F16.S4).

use crate::nodes::{Block, RunContent};
use crate::vocabulary::{BookmarkAnchor, FieldData, FieldType};
use crate::Document;

pub const INDEX_TITLE: &str = "Index";

/// Visible text anchored by a bookmark (runs after the bookmark in the same paragraph).
pub fn bookmark_anchor_text(doc: &Document, name: &str) -> Option<String> {
    for section in &doc.sections {
        for block in &section.blocks {
            let Some(para) = block.paragraph() else {
                continue;
            };
            for (index, run) in para.runs.iter().enumerate() {
                let RunContent::Bookmark(anchor) = &run.content else {
                    continue;
                };
                if !anchor.name.eq_ignore_ascii_case(name) {
                    continue;
                }
                let text: String = para.runs[index + 1..]
                    .iter()
                    .filter(|r| r.format.hidden != Some(true))
                    .map(|r| r.text().to_string())
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string();
                return Some(if text.is_empty() {
                    name.to_string()
                } else {
                    text
                });
            }
        }
    }
    None
}

pub fn bookmark_exists(doc: &Document, name: &str) -> bool {
    doc.sections.iter().any(|section| {
        section.blocks.iter().any(|block| {
            block.paragraph().is_some_and(|para| {
                para.runs.iter().any(|run| {
                    matches!(
                        &run.content,
                        RunContent::Bookmark(b) if b.name.eq_ignore_ascii_case(name)
                    )
                })
            })
        })
    })
}

pub fn cross_ref_instruction(bookmark_name: &str) -> String {
    format!(" REF {bookmark_name} \\h ")
}

pub fn cross_ref_field_data(doc: &Document, bookmark_name: &str) -> Option<FieldData> {
    let display = bookmark_anchor_text(doc, bookmark_name)?;
    Some(FieldData {
        field_type: FieldType::CrossRef,
        instruction: Some(cross_ref_instruction(bookmark_name)),
        display_text: Some(display),
    })
}

/// Bookmark names in document order (deduplicated).
pub fn bookmark_names(doc: &Document) -> Vec<String> {
    let mut names = Vec::new();
    for section in &doc.sections {
        for block in &section.blocks {
            let Some(para) = block.paragraph() else {
                continue;
            };
            for run in &para.runs {
                if let RunContent::Bookmark(anchor) = &run.content {
                    if !names.iter().any(|n: &String| n.eq_ignore_ascii_case(&anchor.name)) {
                        names.push(anchor.name.clone());
                    }
                }
            }
        }
    }
    names
}

/// Build index title + one entry per bookmark (sorted by display text).
pub fn build_index_blocks(doc: &Document) -> Vec<Block> {
    let mut entries: Vec<(String, String)> = bookmark_names(doc)
        .into_iter()
        .filter_map(|name| bookmark_anchor_text(doc, &name).map(|text| (name, text)))
        .collect();
    if entries.is_empty() {
        return Vec::new();
    }
    entries.sort_by(|a, b| a.1.to_ascii_lowercase().cmp(&b.1.to_ascii_lowercase()));

    let mut blocks = Vec::with_capacity(entries.len() + 1);
    blocks.push(Block::Paragraph(crate::Paragraph::with_text(INDEX_TITLE)));
    for (_, text) in entries {
        blocks.push(Block::Paragraph(crate::Paragraph::with_text(text)));
    }
    blocks
}

impl Document {
    pub fn next_bookmark_id(&self) -> i32 {
        let mut max_id = 0i32;
        for section in &self.sections {
            for block in &section.blocks {
                let Some(para) = block.paragraph() else {
                    continue;
                };
                for run in &para.runs {
                    if let RunContent::Bookmark(anchor) = &run.content {
                        if let Some(id) = anchor.bookmark_id {
                            max_id = max_id.max(id);
                        }
                    }
                }
            }
        }
        max_id + 1
    }
}

pub fn bookmark_run(name: impl Into<String>, bookmark_id: i32) -> crate::Run {
    crate::Run {
        id: crate::NodeId::new(),
        format: Default::default(),
        content: RunContent::Bookmark(BookmarkAnchor {
            name: name.into(),
            bookmark_id: Some(bookmark_id),
        }),
        revision: None,
    }
}
