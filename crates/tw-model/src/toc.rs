//! Table of contents paragraph materialization (F16.S2).

use crate::format::{ParaFormat, TabAlignment, TabStop};
use crate::nodes::{Block, Paragraph, Run, RunContent};
use crate::outline::OutlineEntry;

pub const TOC_TITLE: &str = "Table of Contents";

/// Left indent per outline level (points).
pub const TOC_INDENT_PER_LEVEL: f32 = 18.0;

/// Right-aligned tab stop for page numbers (points from paragraph origin).
pub const TOC_TAB_POSITION: f32 = 468.0;

/// Build TOC title + entry paragraphs from outline entries and page numbers.
pub fn build_toc_blocks(entries: &[(OutlineEntry, u32)]) -> Vec<Block> {
    let mut blocks = Vec::with_capacity(entries.len() + 1);

    let mut title = Paragraph::with_text(TOC_TITLE);
    title.format.space_after = Some(12.0);
    blocks.push(Block::Paragraph(title));

    for (entry, page) in entries {
        blocks.push(Block::Paragraph(toc_entry_paragraph(entry, *page)));
    }

    blocks
}

fn toc_entry_paragraph(entry: &OutlineEntry, page: u32) -> Paragraph {
    let mut para = Paragraph::new();
    para.format = ParaFormat {
        indent_left: Some(entry.level as f32 * TOC_INDENT_PER_LEVEL),
        tab_stops: Some(vec![TabStop {
            position: TOC_TAB_POSITION,
            alignment: TabAlignment::Right,
        }]),
        ..Default::default()
    };
    para.runs = vec![
        Run::new_text(&entry.text),
        Run {
            id: crate::NodeId::new(),
            format: Default::default(),
            content: RunContent::Tab,
            revision: None,
        },
        Run::new_text(page.to_string()),
    ];
    para
}
