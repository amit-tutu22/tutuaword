mod access;
mod block_ops;
mod command;
mod normalize;
mod session;

pub use command::*;
pub use session::*;

use access::{with_paragraph_mut, with_run_mut};
use tw_model::{Document, NodeId, NumberingRef, Revision, Run, StyleId};
use tw_text::TextBuffer;

pub fn apply(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    command: Command,
) -> Result<EditResult, EditError> {
    let result = match &command {
        Command::InsertText { run_id, offset, text } => {
            insert_text(doc, buffer, *run_id, *offset, text)?
        }
        Command::DeleteRange {
            run_id,
            start,
            end,
        } => delete_range(doc, buffer, *run_id, *start, *end)?,
        Command::SetCharFormat {
            run_id,
            start,
            end,
            format,
            merge,
        } => set_char_format(doc, buffer, *run_id, *start, *end, format.clone(), *merge)?,
        Command::SetParaFormat {
            paragraph_id,
            format,
            merge,
        } => set_para_format(doc, *paragraph_id, format.clone(), *merge)?,
        Command::InsertParagraph { after_id } => insert_paragraph(doc, buffer, *after_id)?,
        Command::DeleteParagraph { id } => delete_paragraph(doc, buffer, *id)?,
        Command::InsertTable {
            after_block_id,
            rows,
            cols,
        } => block_ops::insert_table(doc, *after_block_id, *rows, *cols)?,
        Command::InsertImage {
            after_block_id,
            width,
            height,
        } => block_ops::insert_image(doc, *after_block_id, *width, *height)?,
        Command::ApplyParagraphStyle {
            paragraph_id,
            style_name,
        } => apply_paragraph_style(doc, *paragraph_id, style_name)?,
        Command::ApplyParagraphStyleById {
            paragraph_id,
            style_id,
        } => apply_paragraph_style_by_id(doc, *paragraph_id, *style_id)?,
        Command::SetNumbering {
            paragraph_id,
            numbering,
        } => set_numbering(doc, *paragraph_id, *numbering)?,
        Command::InsertPageBreak { after_block_id } => {
            block_ops::insert_page_break(doc, *after_block_id)?
        }
        Command::MergeTableCells {
            table_id,
            start_row,
            start_col,
            end_row,
            end_col,
        } => block_ops::merge_table_cells(
            doc,
            *table_id,
            *start_row,
            *start_col,
            *end_row,
            *end_col,
        )?,
        Command::ResizeTableColumn {
            table_id,
            column,
            width,
        } => block_ops::resize_table_column(doc, *table_id, *column, *width)?,
        Command::DeleteBlock { id } => block_ops::delete_block(doc, buffer, *id)?,
        Command::InsertBlock {
            after_block_id,
            block,
        } => block_ops::insert_block(doc, buffer, *after_block_id, block.clone())?,
    };

    normalize::normalize_runs(doc, buffer);
    Ok(result)
}

fn insert_text(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    run_id: NodeId,
    offset: usize,
    text: &str,
) -> Result<EditResult, EditError> {
    let track = doc.settings.track_changes_enabled;
    let author = doc.settings.author_name.clone();
    with_run_mut(doc, run_id, |run| {
        if let Some(t) = run.text_mut() {
            let char_count = t.chars().count();
            if offset > char_count {
                return Err(EditError::InvalidRange);
            }
            let byte_offset = t.char_indices().nth(offset).map(|(i, _)| i).unwrap_or(t.len());
            t.insert_str(byte_offset, text);
            buffer.insert(run_id, offset, text);
            if track {
                run.revision = Some(Revision::insert(author));
            }
        }
        Ok(EditResult {
            affected_nodes: vec![run_id],
            ..Default::default()
        })
    })
    .ok_or(EditError::RunNotFound(run_id))?
}

fn delete_range(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    run_id: NodeId,
    start: usize,
    end: usize,
) -> Result<EditResult, EditError> {
    if start >= end {
        return Err(EditError::InvalidRange);
    }
    if doc.settings.track_changes_enabled {
        return track_delete_range(doc, buffer, run_id, start, end);
    }

    let deleted = buffer.slice(run_id, start..end).into_owned();

    with_run_mut(doc, run_id, |run| {
        if let Some(t) = run.text_mut() {
            let chars: Vec<(usize, char)> = t.char_indices().collect();
            if end > chars.len() {
                return Err(EditError::InvalidRange);
            }
            let byte_start = chars[start].0;
            let byte_end = if end < chars.len() {
                chars[end].0
            } else {
                t.len()
            };
            t.replace_range(byte_start..byte_end, "");
            buffer.delete(run_id, start..end);
        }
        Ok(EditResult {
            affected_nodes: vec![run_id],
            deleted_text: Some(deleted),
            ..Default::default()
        })
    })
    .ok_or(EditError::RunNotFound(run_id))?
}

fn track_delete_range(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    run_id: NodeId,
    start: usize,
    end: usize,
) -> Result<EditResult, EditError> {
    let author = doc.settings.author_name.clone();
    let (si, bi, _) = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    let mut target = run_id;
    if start > 0 {
        target = split_run_at(doc, buffer, si, bi, target, start)?;
    }
    let delete_len = end - start;
    if delete_len < buffer.len(target) {
        split_run_at(doc, buffer, si, bi, target, delete_len)?;
    }

    with_run_mut(doc, target, |run| {
        run.revision = Some(Revision::delete(&author));
        Ok(EditResult {
            affected_nodes: vec![target],
            ..Default::default()
        })
    })
    .ok_or(EditError::RunNotFound(target))?
}

fn set_char_format(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    run_id: NodeId,
    start: usize,
    end: usize,
    format: tw_model::CharFormat,
    merge: bool,
) -> Result<EditResult, EditError> {
    if start == 0 && end >= buffer.len(run_id) {
        return with_run_mut(doc, run_id, |run| {
            let old = run.format.clone();
            if merge {
                run.format.merge(&format);
            } else {
                run.format = format;
            }
            Ok(EditResult {
                affected_nodes: vec![run_id],
                old_char_format: Some(old),
                ..Default::default()
            })
        })
        .ok_or(EditError::RunNotFound(run_id))?;
    }

    let (si, bi, _) = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;
    split_run_at(doc, buffer, si, bi, run_id, start)?;
    if start != end {
        split_run_at(doc, buffer, si, bi, run_id, end)?;
    }

    let para = doc
        .paragraph_at_mut(si, bi)
        .ok_or(EditError::RunNotFound(run_id))?;
    let run_idx = para.runs.iter().position(|r| r.id == run_id).unwrap();
    let old = para.runs.get(run_idx).map(|r| r.format.clone());
    for run in &mut para.runs[run_idx..=run_idx] {
        if merge {
            run.format.merge(&format);
        } else {
            run.format = format.clone();
        }
    }

    Ok(EditResult {
        affected_nodes: vec![run_id],
        old_char_format: old,
        ..Default::default()
    })
}

fn split_run_at(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    si: usize,
    bi: usize,
    run_id: NodeId,
    offset: usize,
) -> Result<NodeId, EditError> {
    let para = doc
        .paragraph_at_mut(si, bi)
        .ok_or(EditError::RunNotFound(run_id))?;
    let idx = para
        .runs
        .iter()
        .position(|r| r.id == run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    let text = buffer.to_string(run_id);
    if offset == 0 || offset >= text.chars().count() {
        return Ok(run_id);
    }

    let suffix: String = text.chars().skip(offset).collect();
    let prefix: String = text.chars().take(offset).collect();

    if let Some(t) = para.runs[idx].text_mut() {
        *t = prefix;
    }
    buffer.sync_from_run(run_id, &buffer.to_string(run_id));

    let new_run = Run {
        id: NodeId::new(),
        format: para.runs[idx].format.clone(),
        content: tw_model::RunContent::Text(suffix),
        revision: None,
    };
    buffer.register(new_run.id, new_run.text());
    let new_id = new_run.id;
    para.runs.insert(idx + 1, new_run);
    Ok(new_id)
}

fn set_para_format(
    doc: &mut Document,
    paragraph_id: NodeId,
    format: tw_model::ParaFormat,
    merge: bool,
) -> Result<EditResult, EditError> {
    with_paragraph_mut(doc, paragraph_id, |para| {
        let old = para.format.clone();
        if merge {
            para.format.merge(&format);
        } else {
            para.format = format;
        }
        Ok(EditResult {
            affected_nodes: vec![paragraph_id],
            old_para_format: Some(old),
            ..Default::default()
        })
    })
    .ok_or(EditError::ParagraphNotFound(paragraph_id))?
}

fn insert_paragraph(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    after_id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_paragraph_location(after_id)
        .ok_or(EditError::ParagraphNotFound(after_id))?;

    let new_para = tw_model::Paragraph::new();
    let new_id = new_para.id;
    for run in &new_para.runs {
        buffer.register(run.id, run.text());
    }

    doc.sections[si]
        .blocks
        .insert(bi + 1, tw_model::Block::Paragraph(new_para));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

fn delete_paragraph(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_paragraph_location(id)
        .ok_or(EditError::ParagraphNotFound(id))?;

    if bi == 0 || doc.sections[si].blocks.len() <= 1 {
        return Err(EditError::InvalidRange);
    }

    let after_id = doc.sections[si].blocks[bi - 1].paragraph().unwrap().id;

    let tw_model::Block::Paragraph(p) = doc.sections[si].blocks.remove(bi) else {
        return Err(EditError::ParagraphNotFound(id));
    };
    for run in &p.runs {
        buffer.unregister(run.id);
    }
    Ok(EditResult {
        affected_nodes: vec![id],
        previous_paragraph_id: Some(after_id),
        deleted_paragraph: Some(p),
        ..Default::default()
    })
}

fn apply_paragraph_style(
    doc: &mut Document,
    paragraph_id: NodeId,
    style_name: &str,
) -> Result<EditResult, EditError> {
    let style_id = doc
        .styles
        .find_style_by_name(style_name)
        .map(|s| s.id)
        .ok_or_else(|| EditError::StyleNotFound(style_name.to_string()))?;

    apply_paragraph_style_by_id(doc, paragraph_id, Some(style_id))
}

fn apply_paragraph_style_by_id(
    doc: &mut Document,
    paragraph_id: NodeId,
    style_id: Option<StyleId>,
) -> Result<EditResult, EditError> {
    let style = style_id.and_then(|id| doc.styles.paragraph_styles.get(&id).cloned());

    with_paragraph_mut(doc, paragraph_id, |para| {
        let old = para.style_id;
        para.style_id = style_id;
        if let Some(style) = style {
            para.format.merge(&style.para_format);
            if let Some(run) = para.runs.first_mut() {
                run.format.merge(&style.char_format);
            }
        }
        Ok(EditResult {
            affected_nodes: vec![paragraph_id],
            old_style_id: Some(old),
            ..Default::default()
        })
    })
    .ok_or(EditError::ParagraphNotFound(paragraph_id))?
}

fn set_numbering(
    doc: &mut Document,
    paragraph_id: NodeId,
    numbering: Option<NumberingRef>,
) -> Result<EditResult, EditError> {
    with_paragraph_mut(doc, paragraph_id, |para| {
        let old = para.format.numbering;
        para.format.numbering = numbering;
        Ok(EditResult {
            affected_nodes: vec![paragraph_id],
            old_numbering: Some(old),
            ..Default::default()
        })
    })
    .ok_or(EditError::ParagraphNotFound(paragraph_id))?
}

#[cfg(test)]
mod phase2_tests {
    use super::*;

    #[test]
    fn insert_table_adds_block() {
        let mut session = EditSession::new();
        let block_id = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .id;

        session
            .apply(Command::InsertTable {
                after_block_id: block_id,
                rows: 3,
                cols: 3,
            })
            .unwrap();

        assert_eq!(session.document.sections[0].blocks.len(), 2);
        assert!(session.document.sections[0].blocks[1].table().is_some());
    }

    #[test]
    fn apply_heading_style() {
        let mut doc = tw_model::Document::new();
        doc.styles = tw_model::StyleSheet::with_heading1();
        let mut session = EditSession::from_document(doc);
        let para_id = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .id;

        session
            .apply(Command::ApplyParagraphStyle {
                paragraph_id: para_id,
                style_name: "Heading 1".into(),
            })
            .unwrap();

        let para = session.document.paragraph_at(0, 0).unwrap();
        assert!(para.style_id.is_some());
        assert_eq!(para.runs[0].format.bold, Some(true));
    }

    #[test]
    fn set_numbered_list() {
        let mut session = EditSession::new();
        let para_id = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .id;

        session
            .apply(Command::SetNumbering {
                paragraph_id: para_id,
                numbering: Some(tw_model::NumberingRef {
                    numbering_id: 2,
                    level: 0,
                }),
            })
            .unwrap();

        let para = session.document.paragraph_at(0, 0).unwrap();
        assert_eq!(para.format.numbering.unwrap().numbering_id, 2);
    }
}
