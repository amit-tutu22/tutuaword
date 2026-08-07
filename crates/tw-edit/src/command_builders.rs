//! Build [`Command`] values from document context (caret, first paragraph, etc.).

use tw_model::{Block, Document, NodeId, NumberingRef};

use crate::{paragraph_id_for_run, Command};

pub fn first_paragraph_id(doc: &Document) -> Option<NodeId> {
    doc.sections.first()?.blocks.iter().find_map(|b| match b {
        Block::Paragraph(p) => Some(p.id),
        _ => None,
    })
}

pub fn last_block_id(doc: &Document) -> Option<NodeId> {
    doc.sections.first()?.blocks.last().and_then(|b| match b {
        Block::Paragraph(p) => Some(p.id),
        Block::Table(t) => Some(t.id),
        Block::ImageBlock(i) => Some(i.id),
        Block::ShapeBlock(s) => Some(s.id),
        _ => None,
    })
}

pub fn paragraph_id_from_caret(doc: &Document, caret_run_id: Option<NodeId>) -> Option<NodeId> {
    caret_run_id
        .and_then(|run_id| paragraph_id_for_run(doc, run_id).ok())
        .or_else(|| first_paragraph_id(doc))
}

pub fn heading1_command_for(paragraph_id: NodeId) -> Command {
    Command::ApplyParagraphStyle {
        paragraph_id,
        style_name: "Heading 1".into(),
    }
}

pub fn normal_style_command_for(paragraph_id: NodeId) -> Command {
    Command::ApplyParagraphStyle {
        paragraph_id,
        style_name: "Normal".into(),
    }
}

pub fn bullet_list_command_for(paragraph_id: NodeId) -> Command {
    Command::SetNumbering {
        paragraph_id,
        numbering: Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        }),
    }
}

pub fn numbered_list_command_for(paragraph_id: NodeId) -> Command {
    Command::SetNumbering {
        paragraph_id,
        numbering: Some(NumberingRef {
            numbering_id: 2,
            level: 0,
        }),
    }
}

pub fn heading1_command(doc: &Document) -> Option<Command> {
    Some(heading1_command_for(first_paragraph_id(doc)?))
}

pub fn bullet_list_command(doc: &Document) -> Option<Command> {
    Some(bullet_list_command_for(first_paragraph_id(doc)?))
}

pub fn numbered_list_command(doc: &Document) -> Option<Command> {
    Some(numbered_list_command_for(first_paragraph_id(doc)?))
}

pub fn heading1_command_for_caret(doc: &Document, caret_run_id: Option<NodeId>) -> Option<Command> {
    Some(heading1_command_for(paragraph_id_from_caret(doc, caret_run_id)?))
}

pub fn normal_style_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    Some(normal_style_command_for(paragraph_id_from_caret(
        doc, caret_run_id,
    )?))
}

pub fn bullet_list_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    Some(bullet_list_command_for(paragraph_id_from_caret(
        doc, caret_run_id,
    )?))
}

pub fn numbered_list_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    Some(numbered_list_command_for(paragraph_id_from_caret(
        doc, caret_run_id,
    )?))
}

pub fn insert_table_command(doc: &Document, rows: u32, cols: u32) -> Option<Command> {
    Some(Command::InsertTable {
        after_block_id: last_block_id(doc)?,
        rows,
        cols,
    })
}

pub fn insert_image_command(doc: &Document, width: f32, height: f32) -> Option<Command> {
    Some(Command::InsertImage {
        after_block_id: last_block_id(doc)?,
        width,
        height,
    })
}

pub fn insert_page_break_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    Some(Command::InsertPageBreak {
        after_block_id: paragraph_id_from_caret(doc, caret_run_id)?,
    })
}
