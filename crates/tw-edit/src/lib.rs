mod undo;
mod access;
pub mod run_text;
mod block_ops;
mod command;
mod image_ops;
pub mod command_json;
pub mod command_builders;
mod normalize;
pub mod paste;
pub mod range;
mod session;

pub use command::*;
pub use command_json::*;
pub use command_builders::*;
pub use session::*;
pub use undo::{TransactionGuard, UndoEntry};
pub use range::paragraph_id_for_run;
pub use run_text::{run_char_len, run_char_len_by_id, run_slice, run_slice_by_id, run_with_id};

use access::{with_paragraph_mut, with_run_mut};
use tw_model::{CharFormat, Document, DocumentTheme, FieldData, FieldType, NodeId, NumberingRef, ParaFormat, Revision, RevisionType, Run, RunContent, StyleId, StyleSheetError, field_instruction, resolve_theme};

pub fn apply(
    doc: &mut Document,
    command: Command,
) -> Result<EditResult, EditError> {
    let result = match &command {
        Command::InsertText { run_id, offset, text } => {
            insert_text(doc, *run_id, *offset, text)?
        }
        Command::DeleteRange {
            run_id,
            start,
            end,
        } => delete_range(doc, *run_id, *start, *end)?,
        Command::DeleteDocRange { range } => delete_doc_range(doc, range)?,
        Command::SetCharFormat {
            run_id,
            start,
            end,
            format,
            merge,
        } => set_char_format(doc, *run_id, *start, *end, format.clone(), *merge)?,
        Command::SetCharFormatRange {
            range,
            format,
            merge,
        } => set_char_format_range(doc, range, format.clone(), *merge)?,
        Command::ClearCharFormatFields {
            range,
            clear_color,
            clear_highlight,
        } => clear_char_format_fields(doc, range, *clear_color, *clear_highlight)?,
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
        Command::InsertParagraph { after_id } => insert_paragraph(doc, *after_id)?,
        Command::SplitParagraphAt { run_id, offset } => {
            split_paragraph_at(doc, *run_id, *offset)?
        }
        Command::DeleteParagraph { id } => delete_paragraph(doc, *id)?,
        Command::MergeSplitParagraph { id } => merge_split_paragraph(doc, *id)?,
        Command::InsertTable {
            after_block_id,
            rows,
            cols,
        } => block_ops::insert_table(doc, *after_block_id, *rows, *cols)?,
        Command::InsertImage {
            after_block_id,
            width,
            height,
            data,
        } => block_ops::insert_image(doc, *after_block_id, *width, *height, data.clone())?,
        Command::SetImageSize {
            image_id,
            width,
            height,
        } => block_ops::set_image_size(doc, *image_id, *width, *height)?,
        Command::ReplaceImageBytes { image_id, data } => {
            block_ops::replace_image_bytes(doc, *image_id, data.clone())?
        }
        Command::SetImageWrap { image_id, wrap } => {
            block_ops::set_image_wrap(doc, *image_id, *wrap)?
        }
        Command::SetImageAnchor { image_id, anchor } => {
            block_ops::set_image_anchor(doc, *image_id, *anchor)?
        }
        Command::RestoreImageLayout {
            image_id,
            wrap,
            anchor,
        } => block_ops::restore_image_layout(doc, *image_id, *wrap, *anchor)?,
        Command::SetImageTransform {
            image_id,
            transform,
        } => block_ops::set_image_transform(doc, *image_id, *transform)?,
        Command::InsertImageCaption { image_id } => {
            block_ops::insert_image_caption(doc, *image_id)?
        }
        Command::RemoveImageCaption {
            image_id,
            caption_paragraph_id,
        } => block_ops::remove_image_caption(doc, *image_id, *caption_paragraph_id)?,
        Command::CompressImage { image_id, quality } => {
            block_ops::compress_image(doc, *image_id, *quality)?
        }
        Command::InsertShape {
            after_block_id,
            shape_type,
            width,
            height,
            style,
        } => block_ops::insert_shape(
            doc,
            *after_block_id,
            *shape_type,
            *width,
            *height,
            style.clone(),
        )?,
        Command::InsertTextBox {
            after_block_id,
            width,
            height,
            style,
        } => block_ops::insert_text_box(doc, *after_block_id, *width, *height, style.clone())?,
        Command::InsertWordArt {
            after_block_id,
            text,
            width,
            height,
        } => block_ops::insert_word_art(doc, *after_block_id, text.clone(), *width, *height)?,
        Command::InsertDiagram {
            after_block_id,
            width,
            height,
            kind,
        } => block_ops::insert_diagram(doc, *after_block_id, *width, *height, *kind)?,
        Command::InsertChart {
            after_block_id,
            width,
            height,
            kind,
        } => block_ops::insert_chart(doc, *after_block_id, *width, *height, *kind)?,
        Command::SetChartData {
            shape_id,
            chart_data,
        } => block_ops::set_chart_data(doc, *shape_id, chart_data.clone())?,
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
        Command::RestartNumbering { paragraph_id } => {
            restart_numbering(doc, *paragraph_id)?
        }
        Command::ContinueNumbering { paragraph_id } => {
            continue_numbering(doc, *paragraph_id)?
        }
        Command::CreateParagraphStyle {
            name,
            based_on_name,
            char_format,
            para_format,
        } => create_paragraph_style(
            doc,
            name,
            based_on_name.as_deref(),
            char_format,
            para_format,
        )?,
        Command::RenameParagraphStyle {
            style_name,
            new_name,
        } => rename_paragraph_style(doc, style_name, new_name)?,
        Command::DeleteParagraphStyle { style_id } => {
            delete_paragraph_style(doc, *style_id)?
        }
        Command::RestoreParagraphStyle {
            style,
            ooxml_style_id,
            paragraph_assignments,
        } => restore_paragraph_style(doc, style.clone(), ooxml_style_id, paragraph_assignments)?,
        Command::SetDocumentTheme { theme_name } => {
            set_document_theme(doc, theme_name)?
        }
        Command::SetEvenAndOddHeaders { enabled } => {
            set_even_and_odd_headers(doc, *enabled)?
        }
        Command::SetSectionFormat {
            section_index,
            format,
        } => set_section_format(doc, *section_index, format.clone())?,
        Command::InsertPageBreak { after_block_id } => {
            block_ops::insert_page_break(doc, *after_block_id)?
        }
        Command::InsertSectionBreak { after_block_id } => {
            block_ops::insert_section_break(doc, *after_block_id)?
        }
        Command::EnsureHeaderFooter {
            section_index,
            is_header,
            hf_type,
        } => block_ops::ensure_header_footer(doc, *section_index, *is_header, *hf_type)?,
        Command::SetHeaderFooterLink {
            section_index,
            is_header,
            hf_type,
            linked,
        } => block_ops::set_header_footer_link(
            doc,
            *section_index,
            *is_header,
            *hf_type,
            *linked,
        )?,
        Command::InsertField {
            run_id,
            offset,
            field_type,
        } => insert_field(doc, *run_id, *offset, field_type.clone())?,
        Command::MergeSection { section_index } => {
            block_ops::merge_section(doc, *section_index)?
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
        Command::SplitTableCell { table_id, row, col } => {
            block_ops::split_table_cell(doc, *table_id, *row, *col)?
        }
        Command::ResizeTableColumn {
            table_id,
            column,
            width,
        } => block_ops::resize_table_column(doc, *table_id, *column, *width)?,
        Command::SetTableBorder { table_id, border } => {
            block_ops::set_table_border(doc, *table_id, border.clone())?
        }
        Command::SetTableCellShading {
            table_id,
            row,
            col,
            background,
        } => block_ops::set_table_cell_shading(doc, *table_id, *row, *col, *background)?,
        Command::AutoFitTable {
            table_id,
            target_width,
        } => block_ops::autofit_table_to_width(doc, *table_id, *target_width)?,
        Command::RestoreTableColumnWidths {
            table_id,
            column_widths,
        } => block_ops::restore_table_column_widths(doc, *table_id, column_widths.clone())?,
        Command::SortTableRows {
            table_id,
            column,
            ascending,
            skip_header,
        } => block_ops::sort_table_rows(doc, *table_id, *column, *ascending, *skip_header)?,
        Command::RestoreTableRowOrder { table_id, rows } => {
            block_ops::restore_table_row_order(doc, *table_id, rows.clone())?
        }
        Command::InsertNestedTable {
            table_id,
            row,
            col,
            rows,
            cols,
        } => block_ops::insert_nested_table(doc, *table_id, *row, *col, *rows, *cols)?,
        Command::RemoveTableCellBlock {
            table_id,
            row,
            col,
            block_index,
        } => block_ops::remove_table_cell_block(
            doc,
            *table_id,
            *row,
            *col,
            *block_index,
        )?,
        Command::InsertTableCellBlock {
            table_id,
            row,
            col,
            block_index,
            block,
        } => block_ops::insert_table_cell_block(
            doc,
            *table_id,
            *row,
            *col,
            *block_index,
            block.clone(),
        )?,
        Command::DeleteTableRow { table_id, row } => {
            block_ops::delete_table_row(doc, *table_id, *row)?
        }
        Command::DeleteTableColumn { table_id, column } => {
            block_ops::delete_table_column(doc, *table_id, *column)?
        }
        Command::RestoreTableRow {
            table_id,
            row,
            row_data,
        } => block_ops::restore_table_row(doc, *table_id, *row, row_data.clone())?,
        Command::RestoreTableColumn {
            table_id,
            column,
            cells,
            column_width,
        } => block_ops::restore_table_column(
            doc,
            *table_id,
            *column,
            cells.clone(),
            *column_width,
        )?,
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
        } => find_replace(doc, range, find, replace, *match_case)?,
        Command::RestoreFindReplace { segments } => restore_find_replace(doc, segments)?,
        Command::DeleteBlock { id } => block_ops::delete_block(doc, *id)?,
        Command::InsertBlock {
            after_block_id,
            block,
        } => block_ops::insert_block(doc, *after_block_id, block.clone())?,
        Command::AcceptRevision { run_id } => {
            resolve_revision(doc, *run_id, RevisionResolution::Accept)?
        }
        Command::RejectRevision { run_id } => {
            resolve_revision(doc, *run_id, RevisionResolution::Reject)?
        }
        Command::AcceptAllRevisions => resolve_all_revisions(doc, RevisionResolution::Accept)?,
        Command::RejectAllRevisions => resolve_all_revisions(doc, RevisionResolution::Reject)?,
        Command::RestoreRevisionRuns { snapshots } => {
            restore_revision_runs(doc, snapshots)?
        }
    };

    normalize::normalize_runs(doc);
    Ok(result)
}

#[derive(Clone, Copy)]
enum RevisionResolution {
    Accept,
    Reject,
}

fn resolve_revision(
    doc: &mut Document,
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
        apply_resolution(run, run_id, &rev, resolution);
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
            apply_resolution(run, run_id, &rev, resolution);
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
    _run_id: NodeId,
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
            t.clear();
        }
    }
    run.revision = None;
}

fn restore_revision_runs(
    doc: &mut Document,
    snapshots: &[RevisionRunSnapshot],
) -> Result<EditResult, EditError> {
    let mut affected = Vec::new();
    for snap in snapshots {
        with_run_mut(doc, snap.run_id, |run| {
            if let Some(t) = run.text_mut() {
                *t = snap.text.clone();
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

fn insert_field(
    doc: &mut Document,
    run_id: NodeId,
    offset: usize,
    field_type: FieldType,
) -> Result<EditResult, EditError> {
    let loc = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    if offset == 0 {
        if let Some(para) = doc.paragraph_at_loc(loc) {
            if let Some(run) = para.runs.get(loc.run_index) {
                if matches!(&run.content, RunContent::Text(text) if text.is_empty()) {
                    with_run_mut(doc, run_id, |run| {
                        run.content = RunContent::Field(FieldData {
                            field_type: field_type.clone(),
                            instruction: Some(field_instruction(&field_type)),
                            display_text: None,
                        });
                    })
                    .ok_or(EditError::RunNotFound(run_id))?;
                    return Ok(EditResult {
                        affected_nodes: vec![run_id],
                        created_node_id: Some(run_id),
                        seed_run_id: Some(run_id),
                        ..Default::default()
                    });
                }
            }
        }
    }

    let format = doc
        .run_at(loc)
        .ok_or(EditError::RunNotFound(run_id))?
        .format
        .clone();
    let mut insert_at = loc.run_index;

    if doc
        .run_at(loc)
        .is_some_and(|run| matches!(run.content, RunContent::Text(_)))
    {
        let char_len = run_char_len_by_id(doc, run_id);
        if offset > 0 && offset < char_len {
            split_run_at(doc, loc, run_id, offset)?;
            insert_at = loc.run_index + 1;
        } else if offset >= char_len {
            insert_at = loc.run_index + 1;
        }
    }

    let instruction = field_instruction(&field_type);
    let field_run = Run {
        id: NodeId::new(),
        format,
        content: RunContent::Field(FieldData {
            field_type,
            instruction: Some(instruction),
            display_text: None,
        }),
        revision: None,
    };
    let field_id = field_run.id;

    doc.paragraph_at_loc_mut(loc)
        .ok_or(EditError::RunNotFound(run_id))?
        .runs
        .insert(insert_at, field_run);

    Ok(EditResult {
        affected_nodes: vec![field_id],
        created_node_id: Some(field_id),
        seed_run_id: Some(field_id),
        ..Default::default()
    })
}

fn insert_text(
    doc: &mut Document,
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
    run_id: NodeId,
    start: usize,
    end: usize,
) -> Result<EditResult, EditError> {
    if start >= end {
        return Err(EditError::InvalidRange);
    }
    if doc.settings.track_changes_enabled {
        return track_delete_range(doc, run_id, start, end);
    }

    let deleted = run_slice_by_id(doc, run_id, start..end);

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
    range: &DocRange,
) -> Result<EditResult, EditError> {
    let range = crate::range::normalize_range(doc, range)?;
    if crate::range::positions_equal(&range.start, &range.end) {
        return Err(EditError::InvalidRange);
    }
    if range.start.run_id == range.end.run_id {
        return delete_range(
            doc,
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
            if (si, tw_model::BlockZone::Body.sort_key(), bi) < start_loc.block_key()
                || (si, tw_model::BlockZone::Body.sort_key(), bi) > end_loc.block_key()
            {
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
                    run_char_len_by_id(doc, run.id)
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
        let deleted = run_slice_by_id(doc, run_id, start..end);
        undo_segments.push((run_id, start, deleted, String::new()));
        delete_range(doc, run_id, start, end)?;
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
    run_id: NodeId,
    start: usize,
    end: usize,
) -> Result<EditResult, EditError> {
    let author = doc.settings.author_name.clone();
    let loc = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    let mut target = run_id;
    if start > 0 {
        target = split_run_at(doc, loc, run_id, start)?;
    }
    let delete_len = end - start;
    if delete_len < run_char_len_by_id(doc, target) {
        let _ = split_run_at(doc, loc, target, delete_len)?;
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
    run_id: NodeId,
    start: usize,
    end: usize,
    format: tw_model::CharFormat,
    merge: bool,
) -> Result<EditResult, EditError> {
    let run_len = run_char_len_by_id(doc, run_id);
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
        return set_char_format(doc, run_id, 0, run_len, format, merge);
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

    let loc = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    // Split so `target` is exactly the [start, end) slice.
    // First split off the prefix; the returned id is the suffix starting at `start`.
    let mut target = run_id;
    if start > 0 {
        target = split_run_at(doc, loc, run_id, start)?;
    }
    // Then split off the tail beyond `end - start`.
    let slice_len = end - start;
    if slice_len < run_char_len_by_id(doc, target) {
        let _ = split_run_at(doc, loc, target, slice_len)?;
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
            if (si, tw_model::BlockZone::Body.sort_key(), bi) < start_loc.block_key()
                || (si, tw_model::BlockZone::Body.sort_key(), bi) > end_loc.block_key()
            {
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
                run.format.theme_color = None;
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
    range: &DocRange,
    format: tw_model::CharFormat,
    merge: bool,
) -> Result<EditResult, EditError> {
    let range = range::normalize_range(doc, range)?;
    if range::positions_equal(&range.start, &range.end) {
        return set_char_format(
            doc,
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
    if start_loc.block_key() == end_loc.block_key() {
        return format_runs_in_paragraph(
            doc,
            start_loc,
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
            let body_key = (si, tw_model::BlockZone::Body.sort_key(), bi);
            if body_key == start_loc.block_key() {
                let last = para.runs.last().ok_or(EditError::InvalidRange)?;
                first_end = DocPosition {
                    run_id: last.id,
                    char_offset: run_char_len_by_id(doc, last.id),
                };
            } else if body_key == end_loc.block_key() {
                let first = para.runs.first().ok_or(EditError::InvalidRange)?;
                last_start = DocPosition {
                    run_id: first.id,
                    char_offset: 0,
                };
            } else if body_key > start_loc.block_key() && body_key < end_loc.block_key() {
                middle.push((si, bi));
            }
        }
    }

    let mut affected = Vec::new();
    let mut old_run_formats = Vec::new();

    let partial = format_runs_in_paragraph(
        doc,
        start_loc,
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
                    char_offset: run_char_len_by_id(doc, last.id),
                },
            )
        };
        let partial = format_runs_in_paragraph(
            doc,
            tw_model::RunLocation {
                section_index: si,
                zone: tw_model::BlockZone::Body,
                block_index: bi,
                run_index: 0,
                table_cell: None,
                shape_paragraph: None,
            },
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
        end_loc,
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
    loc: tw_model::RunLocation,
    start: &DocPosition,
    end: &DocPosition,
    format: tw_model::CharFormat,
    merge: bool,
) -> Result<EditResult, EditError> {
    if start.run_id == end.run_id {
        return set_char_format(
            doc,
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
            .paragraph_at_loc(loc)
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
    } else if end.char_offset < run_char_len_by_id(doc, end.run_id) {
        let _ = split_run_at(doc, loc, end.run_id, end.char_offset)?;
        // Prefix keeps end.run_id and is exactly what we want.
        last_id = end.run_id;
    }

    let mut first_id = start.run_id;
    if start.char_offset > 0 {
        first_id = split_run_at(doc, loc, start.run_id, start.char_offset)?;
    } else if start.char_offset == 0 {
        first_id = start.run_id;
    }

    let para = doc
        .paragraph_at_loc_mut(loc)
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
    loc: tw_model::RunLocation,
    run_id: NodeId,
    offset: usize,
) -> Result<NodeId, EditError> {
    let para = doc
        .paragraph_at_loc_mut(loc)
        .ok_or(EditError::RunNotFound(run_id))?;
    let idx = para
        .runs
        .iter()
        .position(|r| r.id == run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    let text = para.runs[idx].text().to_string();
    let char_len = text.chars().count();
    if offset == 0 || offset >= char_len {
        return Ok(run_id);
    }

    let suffix: String = text.chars().skip(offset).collect();
    let prefix: String = text.chars().take(offset).collect();

    if let Some(t) = para.runs[idx].text_mut() {
        *t = prefix.clone();
    }

    let new_run = Run {
        id: NodeId::new(),
        format: para.runs[idx].format.clone(),
        content: tw_model::RunContent::Text(suffix),
        revision: None,
    };
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
    after_id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_paragraph_location(after_id)
        .ok_or(EditError::ParagraphNotFound(after_id))?;

    let new_para = tw_model::Paragraph::new();
    let new_id = new_para.id;

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
    run_id: NodeId,
    offset: usize,
) -> Result<EditResult, EditError> {
    let loc = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;

    let run_len = run_char_len_by_id(doc, run_id);
    if offset > run_len {
        return Err(EditError::InvalidRange);
    }

    // Mid-run: split so the caret boundary is a run start.
    let mut first_moved_run = run_id;
    if offset > 0 && offset < run_len {
        first_moved_run = split_run_at(doc, loc, run_id, offset)?;
    } else if offset == run_len {
        // Caret at end of this run — move subsequent runs (or insert empty para).
        let para = doc
            .paragraph_at_loc(loc)
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
            let result = insert_paragraph_in_zone(doc, loc, after_id)?;
            let new_para_id = result.created_node_id.unwrap();
            let new_run_id = doc
                .blocks_at(tw_model::RunLocation {
                    section_index: loc.section_index,
                    zone: loc.zone,
                    block_index: loc.block_index + 1,
                    run_index: 0,
                    table_cell: None,
                    shape_paragraph: None,
                })
                .and_then(|blocks| blocks.get(loc.block_index + 1))
                .and_then(|b| b.paragraph())
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
        .paragraph_at_loc_mut(loc)
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
        para.runs.push(empty);
    }

    let new_first_run = if moved.is_empty() {
        let empty = Run::new_text("");
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
    let previous_paragraph_id = doc
        .paragraph_at_loc(loc)
        .map(|p| p.id)
        .ok_or(EditError::RunNotFound(run_id))?;

    doc.blocks_at_mut(loc)
        .ok_or(EditError::InvalidRange)?
        .insert(loc.block_index + 1, tw_model::Block::Paragraph(new_para));

    Ok(EditResult {
        affected_nodes: vec![new_run_id, new_para_id],
        created_node_id: Some(new_para_id),
        previous_paragraph_id: Some(previous_paragraph_id),
        split_boundary: Some((run_id, offset)),
        ..Default::default()
    })
}

fn insert_paragraph_in_zone(
    doc: &mut Document,
    loc: tw_model::RunLocation,
    after_id: NodeId,
) -> Result<EditResult, EditError> {
    let blocks = doc.blocks_at_mut(loc).ok_or(EditError::InvalidRange)?;
    let insert_at = blocks
        .iter()
        .position(|b| b.paragraph().is_some_and(|p| p.id == after_id))
        .ok_or(EditError::ParagraphNotFound(after_id))?
        + 1;
    let para = tw_model::Paragraph::new();
    let new_id = para.id;
    blocks.insert(insert_at, tw_model::Block::Paragraph(para));
    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

/// Merge a paragraph created by [`split_paragraph_at`] back into its predecessor.
fn merge_split_paragraph(
    doc: &mut Document,
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
    let boundary_offset = prev
        .runs
        .last()
        .map(|r| r.text().chars().count())
        .unwrap_or(0);

    for run in new_para.runs {
        if let Some(last) = prev.runs.last_mut() {
            if last.format.equals(&run.format) && last.revision == run.revision {
                let suffix = run.text().to_string();
                if let Some(text) = last.text_mut() {
                    text.push_str(&suffix);
                }
                continue;
            }
        }
        prev.runs.push(run);
    }

    if prev.runs.is_empty() {
        let empty = Run::new_text("");
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

fn map_style_sheet_error(err: StyleSheetError) -> EditError {
    match err {
        StyleSheetError::BuiltinProtected(name) => EditError::BuiltinStyleProtected(name),
        StyleSheetError::DuplicateName(name) => EditError::DuplicateStyleName(name),
        StyleSheetError::NotFound(name) => EditError::StyleNotFound(name),
    }
}

fn create_paragraph_style(
    doc: &mut Document,
    name: &str,
    based_on_name: Option<&str>,
    char_format: &CharFormat,
    para_format: &ParaFormat,
) -> Result<EditResult, EditError> {
    let based_on = based_on_name
        .map(|n| {
            doc.styles
                .find_style_by_name(n)
                .map(|s| s.id)
                .ok_or_else(|| EditError::StyleNotFound(n.to_string()))
        })
        .transpose()?;
    let style_id = doc
        .styles
        .create_paragraph_style(
            name.to_string(),
            based_on,
            char_format.clone(),
            para_format.clone(),
        )
        .map_err(map_style_sheet_error)?;
    Ok(EditResult {
        created_style_id: Some(style_id),
        ..Default::default()
    })
}

fn rename_paragraph_style(
    doc: &mut Document,
    style_name: &str,
    new_name: &str,
) -> Result<EditResult, EditError> {
    let style_id = doc
        .styles
        .find_style_by_name(style_name)
        .map(|s| s.id)
        .ok_or_else(|| EditError::StyleNotFound(style_name.to_string()))?;
    let old_name = doc
        .styles
        .rename_paragraph_style(style_id, new_name.to_string())
        .map_err(map_style_sheet_error)?;
    Ok(EditResult {
        style_renamed: Some((style_id, old_name)),
        ..Default::default()
    })
}

fn delete_paragraph_style(doc: &mut Document, style_id: StyleId) -> Result<EditResult, EditError> {
    let style = doc
        .styles
        .paragraph_styles
        .get(&style_id)
        .cloned()
        .ok_or_else(|| EditError::StyleNotFound(format!("{style_id:?}")))?;
    let ooxml_id = doc
        .styles
        .ooxml_id_for(style_id)
        .unwrap_or_else(|| style.name.replace(' ', ""));
    let fallback = style.based_on.or_else(|| {
        doc.styles.find_style_by_name("Normal").map(|s| s.id)
    });
    let mut usage_updates = Vec::new();
    for para in doc.paragraphs_mut() {
        if para.style_id == Some(style_id) {
            usage_updates.push((para.id, para.style_id));
            para.style_id = fallback;
        }
    }
    let deleted = doc
        .styles
        .delete_paragraph_style(style_id)
        .map_err(map_style_sheet_error)?;
    Ok(EditResult {
        deleted_style: Some(deleted),
        deleted_style_ooxml_id: Some(ooxml_id),
        style_usage_updates: Some(usage_updates),
        ..Default::default()
    })
}

fn restore_paragraph_style(
    doc: &mut Document,
    style: tw_model::ParagraphStyle,
    ooxml_style_id: &str,
    paragraph_assignments: &[(NodeId, StyleId)],
) -> Result<EditResult, EditError> {
    let style_id = style.id;
    doc.styles
        .ooxml_style_ids
        .insert(ooxml_style_id.to_string(), style_id);
    doc.styles.paragraph_styles.insert(style_id, style);
    for (para_id, assigned_style) in paragraph_assignments {
        if *assigned_style != style_id {
            continue;
        }
        if let Some((si, bi)) = doc.find_paragraph_location(*para_id) {
            if let Some(para) = doc.paragraph_at_mut(si, bi) {
                para.style_id = Some(style_id);
            }
        }
    }
    Ok(EditResult::default())
}

fn set_document_theme(doc: &mut Document, theme_name: &str) -> Result<EditResult, EditError> {
    let theme = DocumentTheme::by_name(theme_name)
        .ok_or_else(|| EditError::ThemeNotFound(theme_name.to_string()))?;
    let old_theme = doc.settings.theme.clone();
    doc.settings.theme = theme;
    resolve_theme(doc);
    Ok(EditResult {
        old_document_theme: Some(old_theme),
        ..Default::default()
    })
}

fn set_even_and_odd_headers(doc: &mut Document, enabled: bool) -> Result<EditResult, EditError> {
    let old = doc.settings.even_and_odd_headers;
    doc.settings.even_and_odd_headers = enabled;
    Ok(EditResult {
        old_even_and_odd_headers: Some(old),
        ..Default::default()
    })
}

fn set_section_format(
    doc: &mut Document,
    section_index: usize,
    format: tw_model::SectionFormat,
) -> Result<EditResult, EditError> {
    let section = doc
        .sections
        .get_mut(section_index)
        .ok_or(EditError::InvalidRange)?;
    let old = section.format.clone();
    section.format.apply_geometry(&format);
    Ok(EditResult {
        old_section_format: Some((section_index, old)),
        ..Default::default()
    })
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
            if (si, tw_model::BlockZone::Body.sort_key(), bi) < start_loc.block_key()
                || (si, tw_model::BlockZone::Body.sort_key(), bi) > end_loc.block_key()
            {
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
                    run_char_len_by_id(doc, run.id)
                };
                if run_start >= run_end {
                    continue;
                }

                let slice = run_slice_by_id(doc, run.id, run_start..run_end);
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
        delete_range(doc, run_id, abs_start, abs_end)?;
        insert_text(doc, run_id, abs_start, replace)?;
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
    start_loc: tw_model::RunLocation,
    end_loc: tw_model::RunLocation,
    doc: &Document,
) -> bool {
    doc.find_run_location(run_id)
        .is_some_and(|loc| loc >= start_loc && loc <= end_loc)
}

/// Read plain text for a document-order range without mutating the model.
pub fn text_in_range(
    doc: &Document,
    range: &DocRange,
) -> Result<String, EditError> {
    let range = crate::range::normalize_range(doc, range)?;
    if crate::range::positions_equal(&range.start, &range.end) {
        return Ok(String::new());
    }
    if range.start.run_id == range.end.run_id {
        let len = run_char_len_by_id(doc, range.start.run_id);
        let start = range.start.char_offset.min(len);
        let end = range.end.char_offset.min(len);
        return Ok(run_slice_by_id(doc, range.start.run_id, start..end));
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
            if (si, tw_model::BlockZone::Body.sort_key(), bi) < start_loc.block_key()
                || (si, tw_model::BlockZone::Body.sort_key(), bi) > end_loc.block_key()
            {
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
                    run_char_len_by_id(doc, run.id)
                };
                let run_len = run_char_len_by_id(doc, run.id);
                let run_start = run_start.min(run_len);
                let run_end = run_end.min(run_len);
                if run_start < run_end {
                    out.push_str(&run_slice_by_id(doc, run.id, run_start..run_end));
                }
            }
        }
    }
    Ok(out)
}

fn restore_find_replace(
    doc: &mut Document,
    segments: &[(NodeId, usize, String, String)],
) -> Result<EditResult, EditError> {
    let mut affected = Vec::new();
    for (run_id, offset, find, replace) in segments.iter().rev() {
        if !replace.is_empty() {
            let replace_len = replace.chars().count();
            delete_range(doc, *run_id, *offset, *offset + replace_len)?;
        }
        if !find.is_empty() {
            insert_text(doc, *run_id, *offset, find)?;
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
    let outline_from_numbering = numbering.and_then(|nr| {
        tw_model::numbering_contributes_to_outline(doc, nr.numbering_id, nr.level)
            .then_some(nr.level.min(8) as u8)
    });
    with_paragraph_mut(doc, paragraph_id, |para| {
        let old = para.format.numbering;
        para.format.numbering = numbering;
        if let Some(level) = outline_from_numbering {
            para.format.outline_level = Some(level);
        } else if numbering.is_none() {
            if let Some(old_nr) = old {
                if para.format.outline_level == Some(old_nr.level.min(8) as u8) {
                    para.format.outline_level = None;
                }
            }
        }
        Ok(EditResult {
            affected_nodes: vec![paragraph_id],
            old_numbering: Some(old),
            ..Default::default()
        })
    })
    .ok_or(EditError::ParagraphNotFound(paragraph_id))?
}

fn restart_numbering(doc: &mut Document, paragraph_id: NodeId) -> Result<EditResult, EditError> {
    ensure_list_paragraph(doc, paragraph_id)?;
    set_para_format(
        doc,
        paragraph_id,
        tw_model::ParaFormat {
            num_restart: Some(true),
            ..Default::default()
        },
        true,
    )
}

fn continue_numbering(doc: &mut Document, paragraph_id: NodeId) -> Result<EditResult, EditError> {
    ensure_list_paragraph(doc, paragraph_id)?;
    with_paragraph_mut(doc, paragraph_id, |para| {
        let old = para.format.clone();
        para.format.num_restart = None;
        Ok(EditResult {
            affected_nodes: vec![paragraph_id],
            old_para_format: Some(old.clone()),
            old_para_formats: vec![(paragraph_id, old)],
            ..Default::default()
        })
    })
    .ok_or(EditError::ParagraphNotFound(paragraph_id))?
}

fn ensure_list_paragraph(doc: &Document, paragraph_id: NodeId) -> Result<(), EditError> {
    let (si, bi) = doc
        .find_paragraph_location(paragraph_id)
        .ok_or(EditError::ParagraphNotFound(paragraph_id))?;
    if doc.paragraph_at(si, bi).unwrap().format.numbering.is_none() {
        return Err(EditError::InvalidRange);
    }
    Ok(())
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
