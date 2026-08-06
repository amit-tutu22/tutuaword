//! Paste helpers for plain and formatted clipboard content.

use crate::command::{DocPosition, DocRange};
use crate::range;
use crate::{Command, EditError, EditSession};
use tw_model::{Block, CharFormat, Document, NodeId, Paragraph};
use tw_text::TextBuffer;

#[derive(Debug, Clone, PartialEq)]
pub struct PasteSegment {
    pub text: String,
    pub format: CharFormat,
}

/// Insert plain text at a caret position (same as [`Command::InsertText`]).
pub fn paste_plain_at(
    session: &mut EditSession,
    run_id: NodeId,
    offset: usize,
    text: &str,
) -> Result<(), EditError> {
    if text.is_empty() {
        return Ok(());
    }
    session.apply(Command::InsertText {
        run_id,
        offset,
        text: text.into(),
    })?;
    Ok(())
}

/// Insert formatted inline segments at a caret position.
pub fn paste_inline_segments_at(
    session: &mut EditSession,
    run_id: NodeId,
    offset: usize,
    segments: &[PasteSegment],
) -> Result<(), EditError> {
    let full: String = segments.iter().map(|s| s.text.as_str()).collect();
    if full.is_empty() {
        return Ok(());
    }

    let para_start = run_char_offset_in_paragraph(&session.document, &session.buffer, run_id, offset)?;
    session.apply(Command::InsertText {
        run_id,
        offset,
        text: full,
    })?;

    let para_id = range::paragraph_id_for_run(&session.document, run_id)?;
    let (si, bi) = session
        .document
        .find_block_location(para_id)
        .ok_or(EditError::ParagraphNotFound(para_id))?;
    let anchor_run = session
        .document
        .paragraph_at(si, bi)
        .and_then(|p| p.runs.first().map(|r| r.id))
        .ok_or(EditError::ParagraphNotFound(para_id))?;

    let mut rel = 0usize;
    for seg in segments {
        let len = seg.text.chars().count();
        if len == 0 || seg.format == CharFormat::default() {
            rel += len;
            continue;
        }

        let (start_run, start_off) = char_position_in_paragraph(
            &session.document,
            &session.buffer,
            anchor_run,
            para_start + rel,
        )?;
        let (end_run, end_off) = char_position_in_paragraph(
            &session.document,
            &session.buffer,
            anchor_run,
            para_start + rel + len,
        )?;

        if start_run == end_run {
            session.apply(Command::SetCharFormat {
                run_id: start_run,
                start: start_off,
                end: end_off,
                format: seg.format.clone(),
                merge: false,
            })?;
        } else {
            session.apply(Command::SetCharFormatRange {
                range: DocRange {
                    start: DocPosition {
                        run_id: start_run,
                        char_offset: start_off,
                    },
                    end: DocPosition {
                        run_id: end_run,
                        char_offset: end_off,
                    },
                },
                format: seg.format.clone(),
                merge: false,
            })?;
        }
        rel += len;
    }
    Ok(())
}

/// Paste a parsed document fragment at the caret.
pub fn paste_fragment_at(
    session: &mut EditSession,
    run_id: NodeId,
    offset: usize,
    fragment: &Document,
) -> Result<(), EditError> {
    let Some(blocks) = fragment.sections.first().map(|s| s.blocks.as_slice()) else {
        return Ok(());
    };
    if blocks.is_empty() {
        return Ok(());
    }

    if let Block::Paragraph(para) = &blocks[0] {
        let segments = segments_from_paragraph(para);
        paste_inline_segments_at(session, run_id, offset, &segments)?;
    }

    let mut after_id = range::paragraph_id_for_run(&session.document, run_id)?;
    for block in blocks.iter().skip(1) {
        let block_id = match block {
            Block::Paragraph(p) => p.id,
            Block::Table(t) => t.id,
            Block::ImageBlock(i) => i.id,
        };
        session.apply(Command::InsertBlock {
            after_block_id: after_id,
            block: block.clone(),
        })?;
        after_id = block_id;
    }
    Ok(())
}

pub fn segments_from_paragraph(para: &Paragraph) -> Vec<PasteSegment> {
    para.runs
        .iter()
        .filter_map(|run| {
            let text = run.text().to_string();
            if text.is_empty() {
                None
            } else {
                Some(PasteSegment {
                    text,
                    format: run.format.clone(),
                })
            }
        })
        .collect()
}

fn run_char_offset_in_paragraph(
    doc: &Document,
    buffer: &TextBuffer,
    run_id: NodeId,
    offset_in_run: usize,
) -> Result<usize, EditError> {
    let (si, bi, ri) = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;
    let para = doc
        .paragraph_at(si, bi)
        .ok_or(EditError::RunNotFound(run_id))?;
    let mut total = 0usize;
    for (i, run) in para.runs.iter().enumerate() {
        if i < ri {
            total += buffer.len(run.id);
        } else if i == ri {
            total += offset_in_run;
            break;
        }
    }
    Ok(total)
}

fn char_position_in_paragraph(
    doc: &Document,
    buffer: &TextBuffer,
    anchor_run: NodeId,
    char_offset: usize,
) -> Result<(NodeId, usize), EditError> {
    let (si, bi, _) = doc
        .find_run_location(anchor_run)
        .ok_or(EditError::RunNotFound(anchor_run))?;
    let para = doc
        .paragraph_at(si, bi)
        .ok_or(EditError::RunNotFound(anchor_run))?;
    let mut remaining = char_offset;
    for run in &para.runs {
        let len = buffer.len(run.id);
        if remaining <= len {
            return Ok((run.id, remaining));
        }
        remaining -= len;
    }
    let last = para
        .runs
        .last()
        .ok_or(EditError::RunNotFound(anchor_run))?;
    Ok((last.id, buffer.len(last.id)))
}
