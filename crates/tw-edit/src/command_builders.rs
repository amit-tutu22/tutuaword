//! Build [`Command`] values from document context (caret, first paragraph, etc.).

use tw_model::{BibliographySource, Block, Document, NodeId, NumberingRef};

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
    paragraph_style_command_for(paragraph_id, "Normal")
}

pub fn paragraph_style_command_for(paragraph_id: NodeId, style_name: &str) -> Command {
    Command::ApplyParagraphStyle {
        paragraph_id,
        style_name: style_name.into(),
    }
}

pub fn paragraph_style_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    style_name: &str,
) -> Option<Command> {
    Some(paragraph_style_command_for(
        paragraph_id_from_caret(doc, caret_run_id)?,
        style_name,
    ))
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

/// Promote (+1) or demote (−1) a list paragraph's `ilvl`, clamped to the
/// numbering definition. Returns `None` when the paragraph is not in a list or
/// the level would not change.
pub fn adjust_list_level_command_for(
    doc: &Document,
    paragraph_id: NodeId,
    delta: i32,
) -> Option<Command> {
    let (si, bi) = doc.find_paragraph_location(paragraph_id)?;
    let para = doc.paragraph_at(si, bi)?;
    let current = para.format.numbering.as_ref()?;
    let def = doc.settings.numbering.get(current.numbering_id)?;
    let max_level = def.levels.iter().map(|l| l.level).max().unwrap_or(0);
    let new_level = (current.level as i32 + delta).clamp(0, max_level as i32) as u32;
    if new_level == current.level {
        return None;
    }
    Some(Command::SetNumbering {
        paragraph_id,
        numbering: Some(NumberingRef {
            numbering_id: current.numbering_id,
            level: new_level,
        }),
    })
}

pub fn adjust_list_level_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    delta: i32,
) -> Option<Command> {
    adjust_list_level_command_for(doc, paragraph_id_from_caret(doc, caret_run_id)?, delta)
}

pub fn restart_numbering_command_for(paragraph_id: NodeId) -> Command {
    Command::RestartNumbering { paragraph_id }
}

pub fn continue_numbering_command_for(paragraph_id: NodeId) -> Command {
    Command::ContinueNumbering { paragraph_id }
}

pub fn restart_numbering_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let paragraph_id = paragraph_id_from_caret(doc, caret_run_id)?;
    let (si, bi) = doc.find_paragraph_location(paragraph_id)?;
    if doc.paragraph_at(si, bi)?.format.numbering.is_none() {
        return None;
    }
    Some(restart_numbering_command_for(paragraph_id))
}

pub fn continue_numbering_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let paragraph_id = paragraph_id_from_caret(doc, caret_run_id)?;
    let (si, bi) = doc.find_paragraph_location(paragraph_id)?;
    if doc.paragraph_at(si, bi)?.format.numbering.is_none() {
        return None;
    }
    Some(continue_numbering_command_for(paragraph_id))
}

pub fn insert_table_command(doc: &Document, rows: u32, cols: u32) -> Option<Command> {
    insert_table_command_for_caret(doc, None, rows, cols)
}

pub fn insert_table_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    rows: u32,
    cols: u32,
) -> Option<Command> {
    Some(Command::InsertTable {
        after_block_id: block_id_from_caret(doc, caret_run_id).or_else(|| last_block_id(doc))?,
        rows,
        cols,
    })
}

pub fn delete_table_row_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, row, _) = doc.find_table_cell_for_run(run_id)?;
    Some(Command::DeleteTableRow {
        table_id,
        row: row as u32,
    })
}

pub fn delete_table_column_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, row, cell) = doc.find_table_cell_for_run(run_id)?;
    let table = doc
        .find_block_location(table_id)
        .and_then(|(si, bi)| doc.sections.get(si)?.blocks.get(bi))?
        .table()?;
    let grid_col = table.rows[row].grid_column_for_cell_index(cell);
    Some(Command::DeleteTableColumn {
        table_id,
        column: grid_col as u32,
    })
}

pub fn merge_table_cells_right_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, row, cell) = doc.find_table_cell_for_run(run_id)?;
    let table = doc
        .find_block_location(table_id)
        .and_then(|(si, bi)| doc.sections.get(si)?.blocks.get(bi))?
        .table()?;
    if cell + 1 >= table.rows[row].cells.len() {
        return None;
    }
    Some(Command::MergeTableCells {
        table_id,
        start_row: row as u32,
        start_col: cell as u32,
        end_row: row as u32,
        end_col: (cell + 1) as u32,
    })
}

pub fn split_table_cell_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, row, cell) = doc.find_table_cell_for_run(run_id)?;
    Some(Command::SplitTableCell {
        table_id,
        row: row as u32,
        col: cell as u32,
    })
}

pub fn set_table_border_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    border: Option<tw_model::BorderSpec>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, _, _) = doc.find_table_cell_for_run(run_id)?;
    Some(Command::SetTableBorder { table_id, border })
}

pub fn set_table_cell_shading_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    background: Option<tw_model::Color>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, row, cell) = doc.find_table_cell_for_run(run_id)?;
    Some(Command::SetTableCellShading {
        table_id,
        row: row as u32,
        col: cell as u32,
        background,
    })
}

pub fn resize_table_column_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    width: f32,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, row, cell) = doc.find_table_cell_for_run(run_id)?;
    let table = doc
        .find_block_location(table_id)
        .and_then(|(si, bi)| doc.sections.get(si)?.blocks.get(bi))?
        .table()?;
    let grid_col = table.rows[row].grid_column_for_cell_index(cell);
    Some(Command::ResizeTableColumn {
        table_id,
        column: grid_col as u32,
        width,
    })
}

pub fn autofit_table_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, _, _) = doc.find_table_cell_for_run(run_id)?;
    let section_index = section_index_for_caret(doc, Some(run_id));
    let format = &doc.sections.get(section_index)?.format;
    let target_width = format.page_width - format.margin_left - format.margin_right;
    Some(Command::AutoFitTable {
        table_id,
        target_width,
    })
}

pub fn sort_table_rows_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    ascending: bool,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, row, cell) = doc.find_table_cell_for_run(run_id)?;
    let table = doc
        .find_block_location(table_id)
        .and_then(|(si, bi)| doc.sections.get(si)?.blocks.get(bi))?
        .table()?;
    let grid_col = table.rows[row].grid_column_for_cell_index(cell);
    Some(Command::SortTableRows {
        table_id,
        column: grid_col as u32,
        ascending,
        skip_header: true,
    })
}

pub fn insert_nested_table_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    rows: u32,
    cols: u32,
) -> Option<Command> {
    let run_id = caret_run_id?;
    let (table_id, row, cell) = doc.find_table_cell_for_run(run_id)?;
    Some(Command::InsertNestedTable {
        table_id,
        row: row as u32,
        col: cell as u32,
        rows,
        cols,
    })
}

pub fn insert_table_sum_field_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    doc.find_table_cell_for_run(run_id)?;
    Some(insert_field_command_for(
        run_id,
        0,
        tw_model::FieldType::TableSumAbove,
    ))
}

pub fn insert_image_command(doc: &Document, width: f32, height: f32) -> Option<Command> {
    Some(Command::InsertImage {
        after_block_id: last_block_id(doc)?,
        width,
        height,
        data: None,
    })
}

pub fn insert_shape_command(doc: &Document, shape_type: tw_model::ShapeKind) -> Option<Command> {
    let (width, height) = match shape_type {
        tw_model::ShapeKind::Line => (120.0, 60.0),
        _ => (120.0, 80.0),
    };
    Some(Command::InsertShape {
        after_block_id: block_id_from_caret(doc, None).or_else(|| last_block_id(doc))?,
        shape_type,
        width,
        height,
        style: tw_model::ShapeStyle::inserted_default(),
    })
}

pub fn insert_text_box_command(doc: &Document) -> Option<Command> {
    Some(Command::InsertTextBox {
        after_block_id: block_id_from_caret(doc, None).or_else(|| last_block_id(doc))?,
        width: 180.0,
        height: 90.0,
        style: tw_model::ShapeStyle::inserted_default(),
    })
}

pub fn insert_word_art_command(doc: &Document, text: impl Into<String>) -> Option<Command> {
    Some(Command::InsertWordArt {
        after_block_id: block_id_from_caret(doc, None).or_else(|| last_block_id(doc))?,
        text: text.into(),
        width: 220.0,
        height: 72.0,
    })
}

pub fn insert_diagram_command(doc: &Document) -> Option<Command> {
    insert_diagram_command_with_kind(doc, tw_model::DiagramKind::Process)
}

pub fn insert_diagram_command_with_kind(
    doc: &Document,
    kind: tw_model::DiagramKind,
) -> Option<Command> {
    Some(Command::InsertDiagram {
        after_block_id: block_id_from_caret(doc, None).or_else(|| last_block_id(doc))?,
        width: 432.0,
        height: 216.0,
        kind,
    })
}

pub fn insert_chart_command(doc: &Document) -> Option<Command> {
    insert_chart_command_with_kind(doc, tw_model::ChartKind::Column)
}

pub fn insert_chart_command_with_kind(
    doc: &Document,
    kind: tw_model::ChartKind,
) -> Option<Command> {
    Some(Command::InsertChart {
        after_block_id: block_id_from_caret(doc, None).or_else(|| last_block_id(doc))?,
        width: 432.0,
        height: 252.0,
        kind,
    })
}

pub fn replace_image_bytes_command(
    image_id: NodeId,
    bytes: Vec<u8>,
    mime_type: String,
) -> Command {
    Command::ReplaceImageBytes {
        image_id,
        data: tw_model::ImageData::from_bytes(bytes, Some(mime_type)),
    }
}

pub fn insert_image_bytes_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    bytes: Vec<u8>,
    mime_type: String,
) -> Option<Command> {
    let data = tw_model::ImageData::from_bytes(bytes, Some(mime_type));
    let (width, height) = data.display_size(468.0);
    Some(Command::InsertImage {
        after_block_id: block_id_from_caret(doc, caret_run_id).or_else(|| last_block_id(doc))?,
        width,
        height,
        data: Some(data),
    })
}

pub fn insert_page_break_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    Some(Command::InsertPageBreak {
        after_block_id: block_id_from_caret(doc, caret_run_id)?,
    })
}

pub fn insert_section_break_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    Some(Command::InsertSectionBreak {
        after_block_id: block_id_from_caret(doc, caret_run_id)?,
    })
}

pub fn ensure_header_footer_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    is_header: bool,
    page_index: Option<u32>,
) -> Option<Command> {
    let section_index = section_index_for_caret(doc, caret_run_id);
    let page_index = page_index.unwrap_or(0);
    let section = doc.sections.get(section_index)?;
    let is_first = section_index == 0 && page_index == 0;
    let hf_type = tw_model::HeaderFooterType::for_page_layout(
        page_index + 1,
        is_first,
        section.format.different_first_page,
        doc.settings.even_and_odd_headers,
    );
    Some(Command::EnsureHeaderFooter {
        section_index,
        is_header,
        hf_type,
    })
}

pub fn set_header_footer_link_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    is_header: bool,
    page_index: Option<u32>,
    linked: bool,
) -> Option<Command> {
    let section_index = section_index_for_caret(doc, caret_run_id);
    if section_index == 0 {
        return None;
    }
    let page_index = page_index.unwrap_or(0);
    let section = doc.sections.get(section_index)?;
    let is_first = section_index == 0 && page_index == 0;
    let hf_type = tw_model::HeaderFooterType::for_page_layout(
        page_index + 1,
        is_first,
        section.format.different_first_page,
        doc.settings.even_and_odd_headers,
    );
    Some(Command::SetHeaderFooterLink {
        section_index,
        is_header,
        hf_type,
        linked,
    })
}

pub fn insert_field_command_for(
    run_id: NodeId,
    offset: usize,
    field_type: tw_model::FieldType,
) -> Command {
    insert_field_command_with_merge(run_id, offset, field_type, None)
}

pub fn insert_field_command_with_merge(
    run_id: NodeId,
    offset: usize,
    field_type: tw_model::FieldType,
    merge_name: Option<String>,
) -> Command {
    Command::InsertField {
        run_id,
        offset,
        field_type,
        merge_name,
    }
}

pub fn reply_to_comment_command_for(comment_id: i32, body_text: impl Into<String>) -> Command {
    Command::ReplyToComment {
        comment_id,
        body_text: body_text.into(),
    }
}

pub fn resolve_comment_command_for(comment_id: i32, resolved: bool) -> Command {
    Command::ResolveComment {
        comment_id,
        resolved,
    }
}

pub fn insert_footnote_command_for(run_id: NodeId, offset: usize) -> Command {
    Command::InsertFootnote { run_id, offset }
}

pub fn insert_endnote_command_for(run_id: NodeId, offset: usize) -> Command {
    Command::InsertEndnote { run_id, offset }
}

pub fn insert_comment_command_for(
    run_id: NodeId,
    offset: usize,
    body_text: impl Into<String>,
) -> Command {
    Command::InsertComment {
        run_id,
        offset,
        body_text: body_text.into(),
    }
}

/// Document Inspector remove selected categories (F22.S3).
pub fn remove_inspect_findings_command(
    comments: bool,
    metadata: bool,
    hidden_text: bool,
) -> Command {
    Command::RemoveInspectFindings {
        comments,
        metadata,
        hidden_text,
    }
}

/// Attach a digital signature (F22.S4).
pub fn add_digital_signature_command(signature: tw_model::DigitalSignature) -> Command {
    Command::AddDigitalSignature { signature }
}

/// Remove all digital signatures (F22.S4).
pub fn clear_digital_signatures_command() -> Command {
    Command::ClearDigitalSignatures
}

pub fn insert_table_of_contents_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    page_numbers: Vec<u32>,
) -> Option<Command> {
    Some(Command::InsertTableOfContents {
        after_block_id: block_id_from_caret(doc, caret_run_id)?,
        page_numbers,
    })
}

pub fn insert_table_of_figures_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    page_numbers: Vec<u32>,
) -> Option<Command> {
    Some(Command::InsertTableOfFigures {
        after_block_id: block_id_from_caret(doc, caret_run_id)?,
        page_numbers,
    })
}

pub fn add_bibliography_source_command(source: BibliographySource) -> Command {
    Command::AddBibliographySource { source }
}

pub fn insert_citation_command_for(
    run_id: NodeId,
    offset: usize,
    source_key: impl Into<String>,
) -> Command {
    Command::InsertCitation {
        run_id,
        offset,
        source_key: source_key.into(),
    }
}

pub fn insert_bibliography_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    Some(Command::InsertBibliography {
        after_block_id: block_id_from_caret(doc, caret_run_id)?,
    })
}

pub fn insert_bookmark_command_for(
    run_id: NodeId,
    offset: usize,
    name: impl Into<String>,
) -> Command {
    Command::InsertBookmark {
        run_id,
        offset,
        name: name.into(),
    }
}

pub fn insert_hyperlink_command_for(
    run_id: NodeId,
    offset: usize,
    url: impl Into<String>,
    text: impl Into<String>,
    tooltip: Option<String>,
) -> Command {
    Command::InsertHyperlink {
        run_id,
        offset,
        url: url.into(),
        text: text.into(),
        tooltip,
    }
}

pub fn insert_cross_reference_command_for(
    run_id: NodeId,
    offset: usize,
    bookmark_name: impl Into<String>,
) -> Command {
    Command::InsertCrossReference {
        run_id,
        offset,
        bookmark_name: bookmark_name.into(),
    }
}

pub fn insert_index_command_for(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    Some(Command::InsertIndex {
        after_block_id: block_id_from_caret(doc, caret_run_id)?,
    })
}

pub fn insert_office_math_command_for(
    run_id: NodeId,
    offset: usize,
    xml: impl Into<String>,
) -> Command {
    Command::InsertOfficeMath {
        run_id,
        offset,
        xml: xml.into(),
    }
}

pub fn insert_office_math_display_command_for_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
    xml: impl Into<String>,
) -> Option<Command> {
    Some(Command::InsertOfficeMathDisplay {
        after_block_id: block_id_from_caret(doc, caret_run_id)?,
        xml: xml.into(),
    })
}

pub fn set_office_math_command_for(run_id: NodeId, xml: impl Into<String>) -> Command {
    Command::SetOfficeMath {
        run_id,
        xml: xml.into(),
    }
}

pub fn section_index_for_caret(doc: &Document, caret_run_id: Option<NodeId>) -> usize {
    caret_run_id
        .and_then(|run_id| doc.find_run_location(run_id).map(|loc| loc.section_index))
        .unwrap_or(0)
}

fn block_id_from_caret(doc: &Document, caret_run_id: Option<NodeId>) -> Option<NodeId> {
    if let Some(run_id) = caret_run_id {
        let loc = doc.find_run_location(run_id)?;
        let block = doc.blocks_at(loc)?.get(loc.block_index)?;
        return Some(match block {
            Block::Paragraph(p) => p.id,
            Block::Table(t) => t.id,
            Block::ImageBlock(i) => i.id,
            Block::ShapeBlock(s) => s.id,
            _ => return None,
        });
    }
    doc.sections.first()?.blocks.last().and_then(|b| match b {
        Block::Paragraph(p) => Some(p.id),
        Block::Table(t) => Some(t.id),
        Block::ImageBlock(i) => Some(i.id),
        Block::ShapeBlock(s) => Some(s.id),
        _ => None,
    })
}

pub fn accept_revision_at_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    tw_model::revision_at_run(doc, run_id).map(|run_id| Command::AcceptRevision { run_id })
}

pub fn reject_revision_at_caret(
    doc: &Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let run_id = caret_run_id?;
    tw_model::revision_at_run(doc, run_id).map(|run_id| Command::RejectRevision { run_id })
}
