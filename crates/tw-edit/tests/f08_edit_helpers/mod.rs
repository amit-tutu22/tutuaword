//! Shared helpers for F08 header/footer edit tests.

#![allow(dead_code)]

use tw_edit::{Command, EditSession};
use tw_model::{
    Block, Document, HeaderFooter, HeaderFooterType, NodeId, Paragraph,
};

pub fn header_with_text(text: &str) -> HeaderFooter {
    HeaderFooter {
        blocks: vec![Block::Paragraph(Paragraph::with_text(text))],
        plain_text: None,
    }
}

pub fn header_plain_text(
    doc: &Document,
    section_index: usize,
    hf_type: HeaderFooterType,
) -> String {
    doc.sections
        .get(section_index)
        .and_then(|section| section.headers.get(&hf_type))
        .map(|hf| {
            hf.blocks
                .iter()
                .filter_map(|b| b.paragraph())
                .flat_map(|p| p.runs.iter())
                .map(|r| r.text())
                .collect::<String>()
        })
        .unwrap_or_default()
}

pub fn first_block_id(doc: &Document) -> NodeId {
    doc.sections[0].blocks[0].paragraph().unwrap().id
}

pub fn insert_section_break_after_first(session: &mut EditSession) {
    let after = first_block_id(&session.document);
    session
        .apply(Command::InsertSectionBreak { after_block_id: after })
        .unwrap();
}
