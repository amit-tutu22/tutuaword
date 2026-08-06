mod access;
mod block_ops;
mod command;
mod normalize;
pub mod paste;
pub mod range;
mod session;

pub use command::*;
pub use session::*;
pub use range::paragraph_id_for_run;

use access::{with_paragraph_mut, with_run_mut};
use tw_model::{Document, NodeId, NumberingRef, Revision, RevisionType, Run, StyleId};
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
        Command::DeleteDocRange { range } => delete_doc_range(doc, buffer, range)?,
        Command::SetCharFormat {
            run_id,
            start,
            end,
            format,
            merge,
        } => set_char_format(doc, buffer, *run_id, *start, *end, format.clone(), *merge)?,
        Command::SetCharFormatRange {
            range,
            format,
            merge,
        } => set_char_format_range(doc, buffer, range, format.clone(), *merge)?,
        Command::ClearCharFormatFields {
            range,
            clear_color,
            clear_highlight,
        } => clear_char_format_fields(doc, buffer, range, *clear_color, *clear_highlight)?,
        Command::SetParaFormat {
            paragraph_id,
            format,
            merge,
        } => set_para_format(doc, *paragraph_id, format.clone(), *merge)?,
        Command::SetParaFormatRange {
            range,
            format,
            merge,
        } => set_para_format_range(doc, range, format.clone(), *merge)?,
        Command::RestoreRunFormats { formats } => restore_run_formats(doc, formats)?,
        Command::RestoreParaFormats { formats } => restore_para_formats(doc, formats)?,
        Command::InsertParagraph { after_id } => insert_paragraph(doc, buffer, *after_id)?,
        Command::SplitParagraphAt { run_id, offset } => {
            split_paragraph_at(doc, buffer, *run_id, *offset)?
        }
        Command::DeleteParagraph { id } => delete_paragraph(doc, buffer, *id)?,
        Command::MergeSplitParagraph { id } => merge_split_paragraph(doc, buffer, *id)?,
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
        Command::SetTableCellSpan {
            table_id,
            row,
            col,
            colspan,
            rowspan,
        } => block_ops::set_table_cell_span(doc, *table_id, *row, *col, *colspan, *rowspan)?,
        Command::FindReplace {
            range,
            find,
            replace,
            match_case,
        } => find_replace(doc, buffer, range, find, replace, *match_case)?,
        Command::RestoreFindReplace { segments } => restore_find_replace(doc, buffer, segments)?,
        Command::DeleteBlock { id } => block_ops::delete_block(doc, buffer, *id)?,
        Command::InsertBlock {
            after_block_id,
            block,
        } => block_ops::insert_block(doc, buffer, *after_block_id, block.clone())?,
        Command::AcceptRevision { run_id } => {
            resolve_revision(doc, buffer, *run_id, RevisionResolution::Accept)?
        }
        Command::RejectRevision { run_id } => {
            resolve_revision(doc, buffer, *run_id, RevisionResolution::Reject)?
        }
        Command::AcceptAllRevisions => resolve_all_revisions(doc, buffer, RevisionResolution::Accept)?,
        Command::RejectAllRevisions => resolve_all_revisions(doc, buffer, RevisionResolution::Reject)?,
        Command::RestoreRevisionRuns { snapshots } => {
            restore_revision_runs(doc, buffer, snapshots)?
        }
    };

    normalize::normalize_runs(doc, buffer);
    Ok(result)
}

#[derive(Clone, Copy)]
enum RevisionResolution {
    Accept,
    Reject,
}

fn resolve_revision(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    run_id: NodeId,
    resolution: RevisionResolution,
) -> Result<EditResult, EditError> {
    let snapshot = with_run_mut(doc, run_id, |run| {
        let Some(rev) = run.revision.clone() else {
            return None;
        };
        let text = run.text().to_string();
        let snap = RevisionRunSnapshot {
            run_id,
            text: text.clone(),
            revision: Some(rev.clone()),
        };
        apply_resolution(run, buffer, run_id, &rev, resolution);
        Some(snap)
    })
    .ok_or(EditError::RunNotFound(run_id))?
    .ok_or(EditError::InvalidRange)?;

    Ok(EditResult {
        affected_nodes: vec![run_id],
        revision_snapshots: Some(vec![snapshot]),
        ..Default::default()
    })
}

fn resolve_all_revisions(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    resolution: RevisionResolution,
) -> Result<EditResult, EditError> {
    let mut targets = Vec::new();
    for para in doc.paragraphs_mut() {
        for run in &para.runs {
            if run.revision.is_some() {
                targets.push(run.id);
            }
        }
    }
    if targets.is_empty() {
        return Ok(EditResult::default());
    }

    let mut snapshots = Vec::new();
    let mut affected = Vec::new();
    for run_id in targets {
        let snap = with_run_mut(doc, run_id, |run| {
            let Some(rev) = run.revision.clone() else {
                return None;
            };
            let text = run.text().to_string();
            let snap = RevisionRunSnapshot {
                run_id,
                text: text.clone(),
                revision: Some(rev.clone()),
            };
            apply_resolution(run, buffer, run_id, &rev, resolution);
            Some(snap)
        })
        .flatten();
        if let Some(snap) = snap {
            affected.push(run_id);
            snapshots.push(snap);
        }
    }

    Ok(EditResult {
        affected_nodes: affected,
        revision_snapshots: Some(snapshots),
        ..Default::default()
    })
}

fn apply_resolution(
    run: &mut Run,
    buffer: &mut TextBuffer,
    run_id: NodeId,
    rev: &Revision,
    resolution: RevisionResolution,
) {
    let drop_text = match (resolution, rev.revision_type) {
        (RevisionResolution::Accept, RevisionType::Delete)
        | (RevisionResolution::Reject, RevisionType::Insert) => true,
        (RevisionResolution::Accept, RevisionType::Insert)
        | (RevisionResolution::Reject, RevisionType::Delete) => false,
    };
    if drop_text {
        if let Some(t) = run.text_mut() {
            let len = t.chars().count();
            t.clear();
            if len > 0 {
                buffer.delete(run_id, 0..len);
            }
        }
    }
    run.revision = None;
}

fn restore_revision_runs(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    snapshots: &[RevisionRunSnapshot],
) -> Result<EditResult, EditError> {
    let mut affected = Vec::new();
    for snap in snapshots {
        with_run_mut(doc, snap.run_id, |run| {
            if let Some(t) = run.text_mut() {
                let old_len = t.chars().count();
                if old_len > 0 {
                    buffer.delete(snap.run_id, 0..old_len);
                }
                *t = snap.text.clone();
                if !snap.text.is_empty() {
                    buffer.insert(snap.run_id, 0, &snap.text);
                }
            }
            run.revision = snap.revision.clone();
        })
        .ok_or(EditError::RunNotFound(snap.run_id))?;
        affected.push(snap.run_id);
    }
    Ok(EditResult {
        affected_nodes: affected,
        ..Default::default()
    })
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

fn delete_doc_range(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    range: &DocRange,
) -> Result<EditResult, EditError> {
    let range = crate::range::normalize_range(doc, range)?;
    if crate::range::positions_equal(&range.start, &range.end) {
        return Err(EditError::InvalidRange);
    }
    if range.start.run_id == range.end.run_id {
        return delete_range(
            doc,
            buffer,
            range.start.run_id,
            range.start.char_offset,
            range.end.char_offset,
        );
    }

    let start_loc = doc
        .find_run_location(range.start.run_id)
        .ok_or(EditError::RunNotFound(range.start.run_id))?;
    let end_loc = doc
        .find_run_location(range.end.run_id)
        .ok_or(EditError::RunNotFound(range.end.run_id))?;

    let mut segments = Vec::new();
    for (si, section) in doc.sections.iter().enumerate() {
        for (bi, block) in section.blocks.iter().enumerate() {
            if (si, bi) < (start_loc.0, start_loc.1) || (si, bi) > (end_loc.0, end_loc.1) {
                continue;
            }
            let Some(para) = block.paragraph() else {
                continue;
            };
            for run in &para.runs {
                if !run_in_doc_range(run.id, start_loc, end_loc, doc) {
                    continue;
                }
                let run_start = if run.id == range.start.run_id {
                    range.start.char_offset
                } else {
                    0
                };
                let run_end = if run.id == range.end.run_id {
                    range.end.char_offset
                } else {
                    buffer.len(run.id)
                };
                if run_start < run_end {
                    segments.push((run.id, run_start, run_end));
                }
            }
        }
    }

    let mut undo_segments = Vec::new();
    let mut affected = Vec::new();
    for (run_id, start, end) in segments.into_iter().rev() {
        let deleted = buffer.slice(run_id, start..end).into_owned();
        undo_segments.push((run_id, start, deleted, String::new()));
        delete_range(doc, buffer, run_id, start, end)?;
        affected.push(run_id);
    }

    Ok(EditResult {
        affected_nodes: affected,
        find_replace_undo: Some(undo_segments),
        ..Default::default()
    })
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
    let run_len = buffer.len(run_id);
    let mut start = start;
    let mut end = end.min(run_len);
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    if start > end {
        return Err(EditError::InvalidRange);
    }
    if start == end {
        if run_len == 0 {
            return with_run_mut(doc, run_id, |run| {
                let old = run.format.clone();
                if merge {
                    run.format.merge(&format);
                } else {
                    run.format = format;
                }
                Ok(EditResult {
                    affected_nodes: vec![run_id],
                    old_char_format: Some(old.clone()),
                    old_run_formats: vec![(run_id, old)],
                    ..Default::default()
                })
            })
            .ok_or(EditError::RunNotFound(run_id))?;
        }
        // Collapsed caret (often at run end via FFI `usize::MAX`) — apply to the
        // whole run so ribbon font/size changes affect visible text.
        return set_char_format(doc, buffer, run_id, 0, run_len, format, merge);
    }

    // Whole-run fast path.
    if start == 0 && end == run_len {
        return with_run_mut(doc, run_id, |run| {
            let old = run.format.clone();
            if merge {
                run.format.merge(&format);
            } else {
                run.format = format;
            }
            Ok(EditResult {
                affected_nodes: vec![run_id],
                old_char_format: Some(old.clone()),
                old_run_formats: vec![(run_id, old)],
                ..Default::default()
            })
        })
        .ok_or(EditError::RunNotFound(run_id))?;
    }

    let (si, bi, _) = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    // Split so `target` is exactly the [start, end) slice.
    // First split off the prefix; the returned id is the suffix starting at `start`.
    let mut target = run_id;
    if start > 0 {
        target = split_run_at(doc, buffer, si, bi, run_id, start)?;
    }
    // Then split off the tail beyond `end - start`.
    let slice_len = end - start;
    if slice_len < buffer.len(target) {
        let _ = split_run_at(doc, buffer, si, bi, target, slice_len)?;
    }

    with_run_mut(doc, target, |run| {
        let old = run.format.clone();
        if merge {
            run.format.merge(&format);
        } else {
            run.format = format;
        }
        Ok(EditResult {
            affected_nodes: vec![target],
            old_char_format: Some(old.clone()),
            old_run_formats: vec![(target, old)],
            ..Default::default()
        })
    })
    .ok_or(EditError::RunNotFound(target))?
}

fn clear_char_format_fields(
    doc: &mut Document,
    _buffer: &mut TextBuffer,
    range: &DocRange,
    clear_color: bool,
    clear_highlight: bool,
) -> Result<EditResult, EditError> {
    if !clear_color && !clear_highlight {
        return Ok(EditResult::default());
    }
    let range = range::normalize_range(doc, range)?;
    let start_loc = doc
        .find_run_location(range.start.run_id)
        .ok_or(EditError::RunNotFound(range.start.run_id))?;
    let end_loc = doc
        .find_run_location(range.end.run_id)
        .ok_or(EditError::RunNotFound(range.end.run_id))?;

    let mut old_run_formats = Vec::new();
    let mut affected = Vec::new();
    let mut run_ids = Vec::new();

    for (si, section) in doc.sections.iter().enumerate() {
        for (bi, block) in section.blocks.iter().enumerate() {
            if (si, bi) < (start_loc.0, start_loc.1) || (si, bi) > (end_loc.0, end_loc.1) {
                continue;
            }
            let Some(para) = block.paragraph() else {
                continue;
            };
            for run in &para.runs {
                if run_in_doc_range(run.id, start_loc, end_loc, doc) {
                    run_ids.push(run.id);
                }
            }
        }
    }

    for run_id in run_ids {
        if let Some(old) = with_run_mut(doc, run_id, |run| {
            let old = run.format.clone();
            if clear_color {
                run.format.color = None;
            }
            if clear_highlight {
                run.format.highlight = None;
            }
            Some(old)
        }) {
            if let Some(old) = old {
                old_run_formats.push((run_id, old));
                affected.push(run_id);
            }
        }
    }

    Ok(EditResult {
        affected_nodes: affected,
        old_run_formats,
        ..Default::default()
    })
}

fn set_char_format_range(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    range: &DocRange,
    format: tw_model::CharFormat,
    merge: bool,
) -> Result<EditResult, EditError> {
    let range = range::normalize_range(doc, range)?;
    if range::positions_equal(&range.start, &range.end) {
        return set_char_format(
            doc,
            buffer,
            range.start.run_id,
            range.start.char_offset,
            usize::MAX,
            format,
            merge,
        );
    }

    let start_loc = doc
        .find_run_location(range.start.run_id)
        .ok_or(EditError::RunNotFound(range.start.run_id))?;
    let end_loc = doc
        .find_run_location(range.end.run_id)
        .ok_or(EditError::RunNotFound(range.end.run_id))?;

    // Same paragraph — handle as one contiguous run span.
    if start_loc.0 == end_loc.0 && start_loc.1 == end_loc.1 {
        return format_runs_in_paragraph(
            doc,
            buffer,
            start_loc.0,
            start_loc.1,
            &range.start,
            &range.end,
            format,
            merge,
        );
    }

    // Snapshot middle paragraph coordinates and boundary end-of-para /
    // start-of-para positions before mutating.
    let mut middle: Vec<(usize, usize)> = Vec::new();
    let mut first_end = DocPosition {
        run_id: range.start.run_id,
        char_offset: 0,
    };
    let mut last_start = DocPosition {
        run_id: range.end.run_id,
        char_offset: 0,
    };

    for (si, section) in doc.sections.iter().enumerate() {
        for (bi, block) in section.blocks.iter().enumerate() {
            let Some(para) = block.paragraph() else {
                continue;
            };
            if (si, bi) == (start_loc.0, start_loc.1) {
                let last = para.runs.last().ok_or(EditError::InvalidRange)?;
                first_end = DocPosition {
                    run_id: last.id,
                    char_offset: buffer.len(last.id),
                };
            } else if (si, bi) == (end_loc.0, end_loc.1) {
                let first = para.runs.first().ok_or(EditError::InvalidRange)?;
                last_start = DocPosition {
                    run_id: first.id,
                    char_offset: 0,
                };
            } else if (si, bi) > (start_loc.0, start_loc.1) && (si, bi) < (end_loc.0, end_loc.1)
            {
                middle.push((si, bi));
            }
        }
    }

    let mut affected = Vec::new();
    let mut old_run_formats = Vec::new();

    let partial = format_runs_in_paragraph(
        doc,
        buffer,
        start_loc.0,
        start_loc.1,
        &range.start,
        &first_end,
        format.clone(),
        merge,
    )?;
    affected.extend(partial.affected_nodes);
    old_run_formats.extend(partial.old_run_formats);

    for (si, bi) in middle {
        let para_empty = doc
            .paragraph_at(si, bi)
            .map(|p| p.runs.is_empty())
            .unwrap_or(true);
        if para_empty {
            continue;
        }
        let (mid_start, mid_end) = {
            let para = doc.paragraph_at(si, bi).ok_or(EditError::InvalidRange)?;
            let first = para.runs[0].id;
            let last = para.runs.last().unwrap();
            (
                DocPosition {
                    run_id: first,
                    char_offset: 0,
                },
                DocPosition {
                    run_id: last.id,
                    char_offset: buffer.len(last.id),
                },
            )
        };
        let partial = format_runs_in_paragraph(
            doc,
            buffer,
            si,
            bi,
            &mid_start,
            &mid_end,
            format.clone(),
            merge,
        )?;
        affected.extend(partial.affected_nodes);
        old_run_formats.extend(partial.old_run_formats);
    }

    let partial = format_runs_in_paragraph(
        doc,
        buffer,
        end_loc.0,
        end_loc.1,
        &last_start,
        &range.end,
        format,
        merge,
    )?;
    affected.extend(partial.affected_nodes);
    old_run_formats.extend(partial.old_run_formats);

    Ok(EditResult {
        affected_nodes: affected,
        old_run_formats,
        ..Default::default()
    })
}

fn format_runs_in_paragraph(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    si: usize,
    bi: usize,
    start: &DocPosition,
    end: &DocPosition,
    format: tw_model::CharFormat,
    merge: bool,
) -> Result<EditResult, EditError> {
    if start.run_id == end.run_id {
        return set_char_format(
            doc,
            buffer,
            start.run_id,
            start.char_offset,
            end.char_offset,
            format,
            merge,
        );
    }

    // Split the end run first so earlier indices stay stable, then the start run.
    let mut last_id = end.run_id;
    if end.char_offset == 0 {
        // Empty selection into the end run — exclude it by walking to previous run.
        let para = doc
            .paragraph_at(si, bi)
            .ok_or(EditError::RunNotFound(end.run_id))?;
        let idx = para
            .runs
            .iter()
            .position(|r| r.id == end.run_id)
            .ok_or(EditError::RunNotFound(end.run_id))?;
        if idx == 0 {
            return Ok(EditResult::default());
        }
        last_id = para.runs[idx - 1].id;
    } else if end.char_offset < buffer.len(end.run_id) {
        let _ = split_run_at(doc, buffer, si, bi, end.run_id, end.char_offset)?;
        // Prefix keeps end.run_id and is exactly what we want.
        last_id = end.run_id;
    }

    let mut first_id = start.run_id;
    if start.char_offset > 0 {
        first_id = split_run_at(doc, buffer, si, bi, start.run_id, start.char_offset)?;
    } else if start.char_offset == 0 {
        first_id = start.run_id;
    }

    let para = doc
        .paragraph_at_mut(si, bi)
        .ok_or(EditError::RunNotFound(start.run_id))?;
    let i0 = para
        .runs
        .iter()
        .position(|r| r.id == first_id)
        .ok_or(EditError::RunNotFound(first_id))?;
    let i1 = para
        .runs
        .iter()
        .position(|r| r.id == last_id)
        .ok_or(EditError::RunNotFound(last_id))?;
    if i0 > i1 {
        return Ok(EditResult::default());
    }

    let mut affected = Vec::new();
    let mut old_run_formats = Vec::new();
    for run in &mut para.runs[i0..=i1] {
        old_run_formats.push((run.id, run.format.clone()));
        if merge {
            run.format.merge(&format);
        } else {
            run.format = format.clone();
        }
        affected.push(run.id);
    }

    Ok(EditResult {
        affected_nodes: affected,
        old_char_format: old_run_formats.first().map(|(_, f)| f.clone()),
        old_run_formats,
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
    let char_len = text.chars().count();
    if offset == 0 || offset >= char_len {
        return Ok(run_id);
    }

    let suffix: String = text.chars().skip(offset).collect();
    let prefix: String = text.chars().take(offset).collect();

    if let Some(t) = para.runs[idx].text_mut() {
        *t = prefix.clone();
    }
    buffer.sync_from_run(run_id, &prefix);

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
            old_para_format: Some(old.clone()),
            old_para_formats: vec![(paragraph_id, old)],
            ..Default::default()
        })
    })
    .ok_or(EditError::ParagraphNotFound(paragraph_id))?
}

fn set_para_format_range(
    doc: &mut Document,
    range: &DocRange,
    format: tw_model::ParaFormat,
    merge: bool,
) -> Result<EditResult, EditError> {
    let range = range::normalize_range(doc, range)?;
    let ids = range::paragraph_ids_in_range(doc, &range)?;
    if ids.is_empty() {
        // Caret / empty range: still format the paragraph holding the caret.
        let id = range::paragraph_id_for_run(doc, range.start.run_id)?;
        return set_para_format(doc, id, format, merge);
    }

    let mut affected = Vec::new();
    let mut old_para_formats = Vec::new();
    for id in ids {
        let partial = set_para_format(doc, id, format.clone(), merge)?;
        affected.extend(partial.affected_nodes);
        old_para_formats.extend(partial.old_para_formats);
    }
    Ok(EditResult {
        affected_nodes: affected,
        old_para_format: old_para_formats.first().map(|(_, f)| f.clone()),
        old_para_formats,
        ..Default::default()
    })
}

fn restore_run_formats(
    doc: &mut Document,
    formats: &[(NodeId, tw_model::CharFormat)],
) -> Result<EditResult, EditError> {
    let mut affected = Vec::new();
    for (run_id, format) in formats {
        with_run_mut(doc, *run_id, |run| {
            run.format = format.clone();
            affected.push(*run_id);
        })
        .ok_or(EditError::RunNotFound(*run_id))?;
    }
    Ok(EditResult {
        affected_nodes: affected,
        ..Default::default()
    })
}

fn restore_para_formats(
    doc: &mut Document,
    formats: &[(NodeId, tw_model::ParaFormat)],
) -> Result<EditResult, EditError> {
    let mut affected = Vec::new();
    for (para_id, format) in formats {
        with_paragraph_mut(doc, *para_id, |para| {
            para.format = format.clone();
            affected.push(*para_id);
        })
        .ok_or(EditError::ParagraphNotFound(*para_id))?;
    }
    Ok(EditResult {
        affected_nodes: affected,
        ..Default::default()
    })
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

/// Word Enter: split the paragraph at `(run_id, offset)`.
///
/// Returns `created_node_id` = new paragraph id and puts the new paragraph's
/// first run id in `affected_nodes[0]` so the caret can land there.
fn split_paragraph_at(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    run_id: NodeId,
    offset: usize,
) -> Result<EditResult, EditError> {
    let (si, bi, _) = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    let run_len = buffer.len(run_id);
    if offset > run_len {
        return Err(EditError::InvalidRange);
    }

    // Mid-run: split so the caret boundary is a run start.
    let mut first_moved_run = run_id;
    if offset > 0 && offset < run_len {
        first_moved_run = split_run_at(doc, buffer, si, bi, run_id, offset)?;
    } else if offset == run_len {
        // Caret at end of this run — move subsequent runs (or insert empty para).
        let para = doc
            .paragraph_at(si, bi)
            .ok_or(EditError::RunNotFound(run_id))?;
        let idx = para
            .runs
            .iter()
            .position(|r| r.id == run_id)
            .ok_or(EditError::RunNotFound(run_id))?;
        if idx + 1 < para.runs.len() {
            first_moved_run = para.runs[idx + 1].id;
        } else {
            // End of paragraph → empty new paragraph after this one.
            let after_id = para.id;
            let result = insert_paragraph(doc, buffer, after_id)?;
            let new_para_id = result.created_node_id.unwrap();
            let new_run_id = doc
                .paragraph_at(si, bi + 1)
                .and_then(|p| p.runs.first().map(|r| r.id))
                .ok_or(EditError::ParagraphNotFound(new_para_id))?;
            return Ok(EditResult {
                affected_nodes: vec![new_run_id, new_para_id],
                created_node_id: Some(new_para_id),
                previous_paragraph_id: Some(after_id),
                split_boundary: Some((run_id, offset)),
                ..Default::default()
            });
        }
    }
    // offset == 0: move this run and everything after.

    let para = doc
        .paragraph_at_mut(si, bi)
        .ok_or(EditError::RunNotFound(run_id))?;
    let move_from = para
        .runs
        .iter()
        .position(|r| r.id == first_moved_run)
        .ok_or(EditError::RunNotFound(first_moved_run))?;

    let old_format = para.format.clone();
    let old_style = para.style_id;
    let moved: Vec<Run> = para.runs.drain(move_from..).collect();

    if para.runs.is_empty() {
        let empty = Run::new_text("");
        buffer.register(empty.id, "");
        para.runs.push(empty);
    }

    let new_first_run = if moved.is_empty() {
        let empty = Run::new_text("");
        buffer.register(empty.id, "");
        let id = empty.id;
        (id, vec![empty])
    } else {
        (moved[0].id, moved)
    };

    let new_para = tw_model::Paragraph {
        id: NodeId::new(),
        format: old_format,
        style_id: old_style,
        runs: new_first_run.1,
    };
    let new_para_id = new_para.id;
    let new_run_id = new_first_run.0;

    doc.sections[si]
        .blocks
        .insert(bi + 1, tw_model::Block::Paragraph(new_para));

    Ok(EditResult {
        affected_nodes: vec![new_run_id, new_para_id],
        created_node_id: Some(new_para_id),
        previous_paragraph_id: Some(doc.sections[si].blocks[bi].paragraph().unwrap().id),
        split_boundary: Some((run_id, offset)),
        ..Default::default()
    })
}

/// Merge a paragraph created by [`split_paragraph_at`] back into its predecessor.
fn merge_split_paragraph(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_paragraph_location(id)
        .ok_or(EditError::ParagraphNotFound(id))?;

    if bi == 0 {
        return Err(EditError::InvalidRange);
    }

    let tw_model::Block::Paragraph(new_para) = doc.sections[si].blocks.remove(bi) else {
        return Err(EditError::ParagraphNotFound(id));
    };

    let prev = doc.sections[si].blocks[bi - 1]
        .paragraph_mut()
        .ok_or(EditError::ParagraphNotFound(id))?;

    let boundary_run = prev
        .runs
        .last()
        .map(|r| r.id)
        .ok_or(EditError::InvalidRange)?;
    let boundary_offset = buffer.len(boundary_run);

    for run in new_para.runs {
        if let Some(last) = prev.runs.last_mut() {
            if last.format.equals(&run.format) && last.revision == run.revision {
                let suffix = buffer.to_string(run.id);
                let last_id = last.id;
                if let Some(text) = last.text_mut() {
                    text.push_str(&suffix);
                    let merged = text.clone();
                    buffer.sync_from_run(last_id, &merged);
                }
                buffer.unregister(run.id);
                continue;
            }
        }
        prev.runs.push(run);
    }

    if prev.runs.is_empty() {
        let empty = Run::new_text("");
        buffer.register(empty.id, "");
        prev.runs.push(empty);
    }

    Ok(EditResult {
        affected_nodes: vec![prev.id],
        split_boundary: Some((boundary_run, boundary_offset)),
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

fn find_in(haystack: &str, needle: &str, match_case: bool) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    let needle_chars: Vec<char> = needle.chars().collect();
    let hay_chars: Vec<char> = haystack.chars().collect();
    for start in 0..=hay_chars.len().saturating_sub(needle_chars.len()) {
        let matched = needle_chars.iter().enumerate().all(|(i, &nc)| {
            let hc = hay_chars[start + i];
            if match_case {
                hc == nc
            } else {
                hc.to_lowercase().eq(nc.to_lowercase())
            }
        });
        if matched {
            return Some(start);
        }
    }
    None
}

fn find_replace(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    range: &DocRange,
    find: &str,
    replace: &str,
    match_case: bool,
) -> Result<EditResult, EditError> {
    if find.is_empty() {
        return Err(EditError::InvalidRange);
    }
    let range = crate::range::normalize_range(doc, range)?;
    let start_loc = doc
        .find_run_location(range.start.run_id)
        .ok_or(EditError::RunNotFound(range.start.run_id))?;
    let end_loc = doc
        .find_run_location(range.end.run_id)
        .ok_or(EditError::RunNotFound(range.end.run_id))?;

    let find_len = find.chars().count();
    let mut undo_segments = Vec::new();
    let mut affected = Vec::new();
    let mut pending = Vec::new();

    for (si, section) in doc.sections.iter().enumerate() {
        for (bi, block) in section.blocks.iter().enumerate() {
            if (si, bi) < (start_loc.0, start_loc.1) || (si, bi) > (end_loc.0, end_loc.1) {
                continue;
            }
            let tw_model::Block::Paragraph(para) = block else {
                continue;
            };
            for run in &para.runs {
                if !run_in_doc_range(run.id, start_loc, end_loc, doc) {
                    continue;
                }
                let run_start = if run.id == range.start.run_id {
                    range.start.char_offset
                } else {
                    0
                };
                let run_end = if run.id == range.end.run_id {
                    range.end.char_offset
                } else {
                    buffer.len(run.id)
                };
                if run_start >= run_end {
                    continue;
                }

                let slice = buffer.slice(run.id, run_start..run_end);
                let mut matches = Vec::new();
                let mut search_from = 0usize;
                let slice_len = slice.chars().count();
                while search_from < slice_len {
                    let tail: String = slice.chars().skip(search_from).collect();
                    if let Some(rel) = find_in(&tail, find, match_case) {
                        let abs = run_start + search_from + rel;
                        matches.push(abs);
                        search_from += rel + find_len;
                    } else {
                        break;
                    }
                }

                for abs_start in matches.into_iter().rev() {
                    pending.push((run.id, abs_start));
                }
            }
        }
    }

    for (run_id, abs_start) in pending {
        let abs_end = abs_start + find_len;
        undo_segments.push((run_id, abs_start, find.to_string(), replace.to_string()));
        delete_range(doc, buffer, run_id, abs_start, abs_end)?;
        insert_text(doc, buffer, run_id, abs_start, replace)?;
        affected.push(run_id);
    }

    Ok(EditResult {
        affected_nodes: affected,
        find_replace_undo: Some(undo_segments),
        ..Default::default()
    })
}

fn run_in_doc_range(
    run_id: NodeId,
    start_loc: (usize, usize, usize),
    end_loc: (usize, usize, usize),
    doc: &Document,
) -> bool {
    doc.find_run_location(run_id)
        .is_some_and(|loc| loc >= start_loc && loc <= end_loc)
}

/// Read plain text for a document-order range without mutating the model.
pub fn text_in_range(
    doc: &Document,
    buffer: &TextBuffer,
    range: &DocRange,
) -> Result<String, EditError> {
    let range = crate::range::normalize_range(doc, range)?;
    if crate::range::positions_equal(&range.start, &range.end) {
        return Ok(String::new());
    }
    if range.start.run_id == range.end.run_id {
        let len = buffer.len(range.start.run_id);
        let start = range.start.char_offset.min(len);
        let end = range.end.char_offset.min(len);
        return Ok(buffer.slice(range.start.run_id, start..end).into_owned());
    }

    let start_loc = doc
        .find_run_location(range.start.run_id)
        .ok_or(EditError::RunNotFound(range.start.run_id))?;
    let end_loc = doc
        .find_run_location(range.end.run_id)
        .ok_or(EditError::RunNotFound(range.end.run_id))?;

    let mut out = String::new();
    for (si, section) in doc.sections.iter().enumerate() {
        for (bi, block) in section.blocks.iter().enumerate() {
            if (si, bi) < (start_loc.0, start_loc.1) || (si, bi) > (end_loc.0, end_loc.1) {
                continue;
            }
            let Some(para) = block.paragraph() else {
                continue;
            };
            for run in &para.runs {
                if !run_in_doc_range(run.id, start_loc, end_loc, doc) {
                    continue;
                }
                let run_start = if run.id == range.start.run_id {
                    range.start.char_offset
                } else {
                    0
                };
                let run_end = if run.id == range.end.run_id {
                    range.end.char_offset
                } else {
                    buffer.len(run.id)
                };
                let run_len = buffer.len(run.id);
                let run_start = run_start.min(run_len);
                let run_end = run_end.min(run_len);
                if run_start < run_end {
                    out.push_str(&buffer.slice(run.id, run_start..run_end));
                }
            }
        }
    }
    Ok(out)
}

fn restore_find_replace(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    segments: &[(NodeId, usize, String, String)],
) -> Result<EditResult, EditError> {
    let mut affected = Vec::new();
    for (run_id, offset, find, replace) in segments.iter().rev() {
        if !replace.is_empty() {
            let replace_len = replace.chars().count();
            delete_range(doc, buffer, *run_id, *offset, *offset + replace_len)?;
        }
        if !find.is_empty() {
            insert_text(doc, buffer, *run_id, *offset, find)?;
        }
        affected.push(*run_id);
    }
    Ok(EditResult {
        affected_nodes: affected,
        ..Default::default()
    })
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
    fn text_in_range_reads_selection() {
        let mut session = EditSession::new();
        let run_id = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .id;
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "Hello world".into(),
            })
            .unwrap();

        let text = text_in_range(
            &session.document,
            &session.buffer,
            &DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: 5,
                },
            },
        )
        .unwrap();
        assert_eq!(text, "Hello");
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

    #[test]
    fn mid_run_char_format_splits_correctly() {
        let mut session = EditSession::new();
        let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "HelloWorld".into(),
            })
            .unwrap();

        session
            .apply(Command::SetCharFormat {
                run_id,
                start: 5,
                end: 10,
                format: tw_model::CharFormat {
                    bold: Some(true),
                    ..Default::default()
                },
                merge: true,
            })
            .unwrap();

        let runs = &session.document.paragraph_at(0, 0).unwrap().runs;
        assert!(runs.len() >= 2);
        let texts: Vec<String> = runs.iter().map(|r| r.text().to_string()).collect();
        assert_eq!(texts.join(""), "HelloWorld");
        let bold_run = runs.iter().find(|r| r.format.bold == Some(true)).unwrap();
        assert_eq!(bold_run.text(), "World");
        let plain = runs.iter().find(|r| r.format.bold != Some(true)).unwrap();
        assert_eq!(plain.text(), "Hello");
    }

    #[test]
    fn char_format_range_spans_two_runs() {
        let mut session = EditSession::new();
        let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "abcdef".into(),
            })
            .unwrap();
        // Split into two runs by formatting the second half first.
        session
            .apply(Command::SetCharFormat {
                run_id,
                start: 3,
                end: 6,
                format: tw_model::CharFormat {
                    italic: Some(true),
                    ..Default::default()
                },
                merge: true,
            })
            .unwrap();

        let runs = &session.document.paragraph_at(0, 0).unwrap().runs;
        assert_eq!(runs.len(), 2);
        let start = DocPosition {
            run_id: runs[0].id,
            char_offset: 1,
        };
        let end = DocPosition {
            run_id: runs[1].id,
            char_offset: 2,
        };

        session
            .apply(Command::SetCharFormatRange {
                range: DocRange { start, end },
                format: tw_model::CharFormat {
                    bold: Some(true),
                    ..Default::default()
                },
                merge: true,
            })
            .unwrap();

        let runs = &session.document.paragraph_at(0, 0).unwrap().runs;
        let bold_text: String = runs
            .iter()
            .filter(|r| r.format.bold == Some(true))
            .map(|r| r.text().to_string())
            .collect();
        assert_eq!(bold_text, "bcde");
    }

    #[test]
    fn para_format_range_sets_alignment() {
        let mut session = EditSession::new();
        let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
        session
            .apply(Command::SetParaFormatRange {
                range: DocRange {
                    start: DocPosition {
                        run_id,
                        char_offset: 0,
                    },
                    end: DocPosition {
                        run_id,
                        char_offset: 0,
                    },
                },
                format: tw_model::ParaFormat {
                    alignment: Some(tw_model::Alignment::Center),
                    ..Default::default()
                },
                merge: true,
            })
            .unwrap();

        let para = session.document.paragraph_at(0, 0).unwrap();
        assert_eq!(para.format.alignment, Some(tw_model::Alignment::Center));
    }

    #[test]
    fn char_format_range_undo_restores() {
        let mut session = EditSession::new();
        let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "abc".into(),
            })
            .unwrap();
        session
            .apply(Command::SetCharFormatRange {
                range: DocRange {
                    start: DocPosition {
                        run_id,
                        char_offset: 0,
                    },
                    end: DocPosition {
                        run_id,
                        char_offset: 3,
                    },
                },
                format: tw_model::CharFormat {
                    bold: Some(true),
                    ..Default::default()
                },
                merge: true,
            })
            .unwrap();
        assert_eq!(
            session.document.paragraph_at(0, 0).unwrap().runs[0]
                .format
                .bold,
            Some(true)
        );
        session.undo().unwrap();
        assert_ne!(
            session.document.paragraph_at(0, 0).unwrap().runs[0]
                .format
                .bold,
            Some(true)
        );
    }

    #[test]
    fn find_replace_within_range() {
        let mut session = EditSession::new();
        let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "foo bar foo".into(),
            })
            .unwrap();
        session
            .apply(Command::FindReplace {
                range: DocRange {
                    start: DocPosition {
                        run_id,
                        char_offset: 0,
                    },
                    end: DocPosition {
                        run_id,
                        char_offset: 11,
                    },
                },
                find: "foo".into(),
                replace: "baz".into(),
                match_case: true,
            })
            .unwrap();
        assert_eq!(
            session.document.paragraph_at(0, 0).unwrap().runs[0].text(),
            "baz bar baz"
        );
        session.undo().unwrap();
        assert_eq!(
            session.document.paragraph_at(0, 0).unwrap().runs[0].text(),
            "foo bar foo"
        );
    }

    #[test]
    fn merge_table_cells_undo_restores_span() {
        let mut session = EditSession::new();
        let after = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .id;
        session
            .apply(Command::InsertTable {
                after_block_id: after,
                rows: 2,
                cols: 2,
            })
            .unwrap();
        let table_id = session.document.sections[0].blocks[1].table().unwrap().id;
        session
            .apply(Command::MergeTableCells {
                table_id,
                start_row: 0,
                start_col: 0,
                end_row: 0,
                end_col: 1,
            })
            .unwrap();
        assert_eq!(
            session.document.sections[0].blocks[1]
                .table()
                .unwrap()
                .rows[0]
                .cells[0]
                .format
                .colspan,
            2
        );
        session.undo().unwrap();
        assert_eq!(
            session.document.sections[0].blocks[1]
                .table()
                .unwrap()
                .rows[0]
                .cells[0]
                .format
                .colspan,
            1
        );
    }
}
