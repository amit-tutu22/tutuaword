//! Table of figures paragraph materialization.

use crate::format::{ParaFormat, TabAlignment, TabStop};
use crate::field::tof_field_data;
use crate::nodes::{Block, Paragraph, Run, RunContent};
use crate::Document;
use crate::ids::NodeId;

pub const TOF_TITLE: &str = "Table of Figures";

pub const TOF_TAB_POSITION: f32 = 468.0;

/// One captioned figure/table entry for a table of figures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptionEntry {
    pub paragraph_id: NodeId,
    pub text: String,
}

/// Walk the document and collect caption paragraphs (Caption style or Figure/Table prefix).
pub fn document_captions(doc: &Document) -> Vec<CaptionEntry> {
    let caption_style = doc.styles.find_style_by_name("Caption");
    let caption_style_id = caption_style.map(|s| s.id);

    let mut entries = Vec::new();
    for section in &doc.sections {
        for block in &section.blocks {
            let Some(para) = block.paragraph() else {
                continue;
            };
            let text = para.full_text().trim().to_string();
            if text.is_empty() {
                continue;
            }
            let is_caption_style = para
                .style_id
                .zip(caption_style_id)
                .is_some_and(|(a, b)| a == b);
            let is_figure_label = text.starts_with("Figure ")
                || text.starts_with("Fig. ")
                || text.starts_with("Table ");
            if is_caption_style || is_figure_label {
                entries.push(CaptionEntry {
                    paragraph_id: para.id,
                    text,
                });
            }
        }
    }
    entries
}

/// Build table-of-figures title + entry paragraphs.
pub fn build_tof_blocks(entries: &[(CaptionEntry, u32)]) -> Vec<Block> {
    let mut blocks = Vec::with_capacity(entries.len() + 1);

    let mut title = Paragraph::new();
    title.format.space_after = Some(12.0);
    title.runs = vec![Run {
        id: crate::NodeId::new(),
        format: Default::default(),
        content: RunContent::Field(tof_field_data(TOF_TITLE)),
        revision: None,
    }];
    blocks.push(Block::Paragraph(title));

    for (entry, page) in entries {
        blocks.push(Block::Paragraph(tof_entry_paragraph(entry, *page)));
    }

    blocks
}

fn tof_entry_paragraph(entry: &CaptionEntry, page: u32) -> Paragraph {
    let mut para = Paragraph::new();
    para.format = ParaFormat {
        tab_stops: Some(vec![TabStop {
            position: TOF_TAB_POSITION,
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
