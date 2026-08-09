use serde::{Deserialize, Serialize};
use web_time::{Duration, Instant};
use tw_model::{CharFormat, NodeId, NumberingRef, ParaFormat, Revision, StyleId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocPosition {
    pub run_id: NodeId,
    pub char_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocRange {
    pub start: DocPosition,
    pub end: DocPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum Command {
    InsertText {
        run_id: NodeId,
        offset: usize,
        text: String,
    },
    DeleteRange {
        run_id: NodeId,
        start: usize,
        end: usize,
    },
    /// Delete characters across an arbitrary document range (possibly spanning runs).
    DeleteDocRange {
        range: DocRange,
    },
    SetCharFormat {
        run_id: NodeId,
        start: usize,
        end: usize,
        format: CharFormat,
        merge: bool,
    },
    /// Apply a character-format delta across an arbitrary document range
    /// (possibly spanning multiple runs / paragraphs).
    SetCharFormatRange {
        range: DocRange,
        format: CharFormat,
        merge: bool,
    },
    /// Clear direct color/highlight flags across a document range.
    ClearCharFormatFields {
        range: DocRange,
        clear_color: bool,
        clear_highlight: bool,
    },
    SetParaFormat {
        paragraph_id: NodeId,
        format: ParaFormat,
        merge: bool,
    },
    /// Apply a paragraph-format delta to every paragraph touched by `range`.
    SetParaFormatRange {
        range: DocRange,
        format: ParaFormat,
        merge: bool,
    },
    /// Restore previously recorded run formats (undo helper for range edits).
    RestoreRunFormats {
        formats: Vec<(NodeId, CharFormat)>,
    },
    /// Restore previously recorded paragraph formats (undo helper for range edits).
    RestoreParaFormats {
        formats: Vec<(NodeId, ParaFormat)>,
    },
    InsertParagraph {
        after_id: NodeId,
    },
    /// Split the paragraph containing `run_id` at `offset` (Word Enter).
    /// Content after the caret moves into a new paragraph; caret lands at its start.
    SplitParagraphAt {
        run_id: NodeId,
        offset: usize,
    },
    DeleteParagraph {
        id: NodeId,
    },
    /// Undo helper for [`Command::SplitParagraphAt`]: merge a split-off paragraph
    /// back into its predecessor.
    MergeSplitParagraph {
        id: NodeId,
    },
    InsertTable {
        after_block_id: NodeId,
        rows: u32,
        cols: u32,
    },
    InsertImage {
        after_block_id: NodeId,
        width: f32,
        height: f32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        data: Option<tw_model::ImageData>,
    },
    /// Resize an inline/floating image block (F10.S2).
    SetImageSize {
        image_id: NodeId,
        width: f32,
        height: f32,
    },
    /// Replace image bytes while preserving wrap/anchor/size (F10.S2).
    ReplaceImageBytes {
        image_id: NodeId,
        data: tw_model::ImageData,
    },
    /// Change text-wrap mode (F10.S3).
    SetImageWrap {
        image_id: NodeId,
        wrap: tw_model::TextWrap,
    },
    /// Position a floating image (F10.S3).
    SetImageAnchor {
        image_id: NodeId,
        anchor: tw_model::ImageAnchor,
    },
    /// Undo helper restoring wrap and anchor together (F10.S3).
    RestoreImageLayout {
        image_id: NodeId,
        wrap: tw_model::TextWrap,
        anchor: Option<tw_model::ImageAnchor>,
    },
    /// Crop/rotate/opacity transform (F10.S4).
    SetImageTransform {
        image_id: NodeId,
        transform: tw_model::ImageTransform,
    },
    /// Insert a Caption-styled paragraph linked to an image (F10.S4).
    InsertImageCaption {
        image_id: NodeId,
    },
    /// Undo helper removing a linked caption paragraph (F10.S4).
    RemoveImageCaption {
        image_id: NodeId,
        caption_paragraph_id: NodeId,
    },
    /// Re-encode image bytes as JPEG (F10.S4).
    CompressImage {
        image_id: NodeId,
        quality: u8,
    },
    /// Insert a native editable shape block (F11.S2).
    InsertShape {
        after_block_id: NodeId,
        shape_type: tw_model::ShapeKind,
        width: f32,
        height: f32,
        #[serde(default = "default_insert_shape_style")]
        style: tw_model::ShapeStyle,
    },
    /// Insert a text box with an empty paragraph (F11.S3).
    InsertTextBox {
        after_block_id: NodeId,
        width: f32,
        height: f32,
        #[serde(default = "default_insert_shape_style")]
        style: tw_model::ShapeStyle,
    },
    /// Insert decorative WordArt text (F11.S3).
    InsertWordArt {
        after_block_id: NodeId,
        text: String,
        width: f32,
        height: f32,
    },
    /// Insert a read-only SmartArt diagram placeholder (F12.S3).
    InsertDiagram {
        after_block_id: NodeId,
        width: f32,
        height: f32,
        #[serde(default)]
        kind: tw_model::DiagramKind,
    },
    /// Insert a chart placeholder with default sample data (F13.S3).
    InsertChart {
        after_block_id: NodeId,
        width: f32,
        height: f32,
        #[serde(default)]
        kind: tw_model::ChartKind,
    },
    /// Replace the editable dataset on a chart shape (F13.S3).
    SetChartData {
        shape_id: NodeId,
        chart_data: Option<tw_model::ChartData>,
    },
    ApplyParagraphStyle {
        paragraph_id: NodeId,
        style_name: String,
    },
    ApplyParagraphStyleById {
        paragraph_id: NodeId,
        style_id: Option<StyleId>,
    },
    SetNumbering {
        paragraph_id: NodeId,
        numbering: Option<NumberingRef>,
    },
    /// Restart list numbering at this paragraph (`w:numRestart`).
    RestartNumbering {
        paragraph_id: NodeId,
    },
    /// Clear restart marker so numbering continues from the running counter.
    ContinueNumbering {
        paragraph_id: NodeId,
    },
    /// Add a user paragraph style to the document catalog.
    CreateParagraphStyle {
        name: String,
        based_on_name: Option<String>,
        char_format: CharFormat,
        para_format: ParaFormat,
    },
    RenameParagraphStyle {
        style_name: String,
        new_name: String,
    },
    DeleteParagraphStyle {
        style_id: StyleId,
    },
    /// Undo helper: restore a deleted custom style and paragraph assignments.
    RestoreParagraphStyle {
        style: tw_model::ParagraphStyle,
        ooxml_style_id: String,
        paragraph_assignments: Vec<(NodeId, StyleId)>,
    },
    /// Apply a built-in document theme (fonts/colors) from the Design gallery.
    SetDocumentTheme {
        theme_name: String,
    },
    /// Toggle odd/even header/footer variants document-wide — F08.S3.
    SetEvenAndOddHeaders {
        enabled: bool,
    },
    /// Update section page geometry (margins, size, orientation) — F07.S1.
    SetSectionFormat {
        section_index: usize,
        format: tw_model::SectionFormat,
    },
    InsertPageBreak {
        after_block_id: NodeId,
    },
    /// Split the document into a new section after `after_block_id` (next page) — F07.S2.
    InsertSectionBreak {
        after_block_id: NodeId,
    },
    /// Undo helper: merge a section back into the previous one.
    MergeSection {
        section_index: usize,
    },
    /// Ensure a header/footer band exists for editing (F08.S1).
    EnsureHeaderFooter {
        section_index: usize,
        is_header: bool,
        hf_type: tw_model::HeaderFooterType,
    },
    /// Link or unlink a header/footer variant from the previous section — F08.S4.
    SetHeaderFooterLink {
        section_index: usize,
        is_header: bool,
        hf_type: tw_model::HeaderFooterType,
        linked: bool,
    },
    /// Insert a Word field run (PAGE, DATE, …) at the caret — F08.S2.
    InsertField {
        run_id: NodeId,
        offset: usize,
        field_type: tw_model::FieldType,
    },
    /// Insert a footnote reference at the caret — F16.S1.
    InsertFootnote {
        run_id: NodeId,
        offset: usize,
    },
    /// Insert a comment anchor at the caret — F17.S3.
    InsertComment {
        run_id: NodeId,
        offset: usize,
        body_text: String,
    },
    /// Materialize a table of contents after a block — F16.S2.
    InsertTableOfContents {
        after_block_id: NodeId,
        /// Page numbers parallel to [`tw_model::document_outline`] at apply time.
        page_numbers: Vec<u32>,
    },
    /// Register a bibliography source by citation key — F16.S3.
    AddBibliographySource {
        source: tw_model::BibliographySource,
    },
    /// Insert an inline citation reference at the caret — F16.S3.
    InsertCitation {
        run_id: NodeId,
        offset: usize,
        source_key: String,
    },
    /// Materialize a bibliography section after a block — F16.S3.
    InsertBibliography {
        after_block_id: NodeId,
    },
    /// Insert a bookmark anchor at the caret — F16.S4.
    InsertBookmark {
        run_id: NodeId,
        offset: usize,
        name: String,
    },
    /// Insert a REF field pointing at a bookmark — F16.S4.
    InsertCrossReference {
        run_id: NodeId,
        offset: usize,
        bookmark_name: String,
    },
    /// Materialize an index from bookmark targets — F16.S4.
    InsertIndex {
        after_block_id: NodeId,
    },
    /// Insert inline OMML at the caret — F14.S3.
    InsertOfficeMath {
        run_id: NodeId,
        offset: usize,
        xml: String,
    },
    /// Insert a display equation block (`m:oMathPara`) after a block — F14.S3.
    InsertOfficeMathDisplay {
        after_block_id: NodeId,
        xml: String,
    },
    /// Replace OMML on an existing equation run — F14.S3.
    SetOfficeMath {
        run_id: NodeId,
        xml: String,
    },
    MergeTableCells {
        table_id: NodeId,
        start_row: u32,
        start_col: u32,
        end_row: u32,
        end_col: u32,
    },
    /// Reset a merged cell back to 1×1 (F09.S3).
    SplitTableCell {
        table_id: NodeId,
        row: u32,
        col: u32,
    },
    ResizeTableColumn {
        table_id: NodeId,
        column: u32,
        width: f32,
    },
    /// Set the table-wide border (F09.S4).
    SetTableBorder {
        table_id: NodeId,
        border: Option<tw_model::BorderSpec>,
    },
    /// Set background shading on one table cell (F09.S4).
    SetTableCellShading {
        table_id: NodeId,
        row: u32,
        col: u32,
        background: Option<tw_model::Color>,
    },
    /// Scale column widths to fit the text column (F09.S4 AutoFit to window).
    AutoFitTable {
        table_id: NodeId,
        target_width: f32,
    },
    /// Undo helper: restore prior column widths.
    RestoreTableColumnWidths {
        table_id: NodeId,
        column_widths: Vec<f32>,
    },
    /// Sort table rows by a column's text/numeric value (F09.S5).
    SortTableRows {
        table_id: NodeId,
        column: u32,
        ascending: bool,
        skip_header: bool,
    },
    /// Undo helper: restore prior row order after sort.
    RestoreTableRowOrder {
        table_id: NodeId,
        rows: Vec<tw_model::TableRow>,
    },
    /// Insert a nested table inside a cell (F09.S5).
    InsertNestedTable {
        table_id: NodeId,
        row: u32,
        col: u32,
        rows: u32,
        cols: u32,
    },
    /// Undo helper: remove a block from a table cell.
    RemoveTableCellBlock {
        table_id: NodeId,
        row: u32,
        col: u32,
        block_index: usize,
    },
    /// Undo helper: restore a block inside a table cell.
    InsertTableCellBlock {
        table_id: NodeId,
        row: u32,
        col: u32,
        block_index: usize,
        block: tw_model::Block,
    },
    /// Remove one row from a table (F09.S2).
    DeleteTableRow {
        table_id: NodeId,
        row: u32,
    },
    /// Remove one column from a table (F09.S2).
    DeleteTableColumn {
        table_id: NodeId,
        column: u32,
    },
    /// Undo helper: re-insert a deleted table row.
    RestoreTableRow {
        table_id: NodeId,
        row: u32,
        row_data: tw_model::TableRow,
    },
    /// Undo helper: re-insert a deleted table column.
    RestoreTableColumn {
        table_id: NodeId,
        column: u32,
        cells: Vec<tw_model::TableCell>,
        column_width: f32,
    },
    /// Restore a table cell's colspan/rowspan (undo helper for merge).
    SetTableCellSpan {
        table_id: NodeId,
        row: u32,
        col: u32,
        colspan: u32,
        rowspan: u32,
    },
    /// Replace every occurrence of `find` with `replace` inside `range`.
    FindReplace {
        range: DocRange,
        find: String,
        replace: String,
        match_case: bool,
        #[serde(default)]
        use_regex: bool,
        #[serde(default)]
        use_wildcards: bool,
    },
    /// Undo helper for [`Command::FindReplace`].
    RestoreFindReplace {
        segments: Vec<(NodeId, usize, String, String)>,
    },
    DeleteBlock {
        id: NodeId,
    },
    InsertBlock {
        after_block_id: NodeId,
        block: tw_model::Block,
    },
    /// Insert a block immediately before an existing block (undo of deleting the first block).
    InsertBlockBefore {
        before_block_id: NodeId,
        block: tw_model::Block,
    },
    /// Accept the track-change revision on a single run (TC ladder step c).
    AcceptRevision {
        run_id: NodeId,
    },
    /// Reject the track-change revision on a single run.
    RejectRevision {
        run_id: NodeId,
    },
    /// Accept every revision in the document.
    AcceptAllRevisions,
    /// Reject every revision in the document.
    RejectAllRevisions,
    /// Undo helper for accept/reject revision commands.
    RestoreRevisionRuns {
        snapshots: Vec<RevisionRunSnapshot>,
    },
}

/// Snapshot of a run before accept/reject so undo can restore text + revision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevisionRunSnapshot {
    pub run_id: NodeId,
    pub text: String,
    pub revision: Option<Revision>,
}

/// Whether `next` can merge into the previous coalesced `InsertText` undo entry.
pub fn can_coalesce_insert(
    prev_cmd: &Command,
    prev_result: &EditResult,
    next_cmd: &Command,
    now: Instant,
    prev_timestamp: Instant,
    coalesce_window: Duration,
) -> bool {
    let _ = prev_result;
    let Command::InsertText {
        run_id: prev_run,
        offset: prev_offset,
        text: prev_text,
    } = prev_cmd
    else {
        return false;
    };
    let Command::InsertText {
        run_id: next_run,
        offset: next_offset,
        text: next_text,
    } = next_cmd
    else {
        return false;
    };
    if prev_run != next_run {
        return false;
    }
    if now.duration_since(prev_timestamp) > coalesce_window {
        return false;
    }
    if next_text.chars().any(char::is_whitespace) {
        return false;
    }
    let prev_end = *prev_offset + prev_text.chars().count();
    *next_offset == prev_end
}

impl Command {
    pub fn inverse(&self, result: &EditResult) -> Result<Command, EditError> {
        match self {
            Command::InsertText { run_id, offset, text } => Ok(Command::DeleteRange {
                run_id: *run_id,
                start: *offset,
                end: offset + text.chars().count(),
            }),
            Command::DeleteRange { run_id, start, end: _ } => {
                let deleted = result
                    .deleted_text
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "DeleteRange",
                    })?;
                Ok(Command::InsertText {
                    run_id: *run_id,
                    offset: *start,
                    text: deleted,
                })
            }
            Command::DeleteDocRange { .. } => {
                let segments = result
                    .find_replace_undo
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "DeleteDocRange",
                    })?;
                Ok(Command::RestoreFindReplace { segments })
            }
            Command::SetCharFormat { .. } => {
                if !result.old_run_formats.is_empty() {
                    Ok(Command::RestoreRunFormats {
                        formats: result.old_run_formats.clone(),
                    })
                } else {
                    let old = result.old_char_format.clone().ok_or(
                        EditError::InverseNotSupported {
                            command: "SetCharFormat",
                        },
                    )?;
                    let run_id = result.affected_nodes.first().copied().ok_or(
                        EditError::InverseNotSupported {
                            command: "SetCharFormat",
                        },
                    )?;
                    Ok(Command::SetCharFormat {
                        run_id,
                        start: 0,
                        end: usize::MAX,
                        format: old,
                        merge: false,
                    })
                }
            }
            Command::SetCharFormatRange { .. } => {
                if result.old_run_formats.is_empty() {
                    Err(EditError::InverseNotSupported {
                        command: "SetCharFormatRange",
                    })
                } else {
                    Ok(Command::RestoreRunFormats {
                        formats: result.old_run_formats.clone(),
                    })
                }
            }
            Command::ClearCharFormatFields { .. } => {
                if result.old_run_formats.is_empty() {
                    Err(EditError::InverseNotSupported {
                        command: "ClearCharFormatFields",
                    })
                } else {
                    Ok(Command::RestoreRunFormats {
                        formats: result.old_run_formats.clone(),
                    })
                }
            }
            Command::SetParaFormat {
                paragraph_id,
                ..
            } => {
                let old = result
                    .old_para_format
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "SetParaFormat",
                    })?;
                Ok(Command::SetParaFormat {
                    paragraph_id: *paragraph_id,
                    format: old,
                    merge: false,
                })
            }
            Command::SetParaFormatRange { .. } => {
                if result.old_para_formats.is_empty() {
                    Err(EditError::InverseNotSupported {
                        command: "SetParaFormatRange",
                    })
                } else {
                    Ok(Command::RestoreParaFormats {
                        formats: result.old_para_formats.clone(),
                    })
                }
            }
            Command::RestoreRunFormats { .. } => Err(EditError::InverseNotSupported {
                command: "RestoreRunFormats",
            }),
            Command::RestoreParaFormats { .. } => Err(EditError::InverseNotSupported {
                command: "RestoreParaFormats",
            }),
            Command::InsertParagraph { .. } => {
                let new_id = result
                    .created_node_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "InsertParagraph",
                    })?;
                Ok(Command::DeleteParagraph { id: new_id })
            }
            Command::SplitParagraphAt { .. } => {
                let new_id = result
                    .created_node_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "SplitParagraphAt",
                    })?;
                Ok(Command::MergeSplitParagraph { id: new_id })
            }
            Command::MergeSplitParagraph { .. } => {
                let (run_id, offset) = result
                    .split_boundary
                    .ok_or(EditError::InverseNotSupported {
                        command: "MergeSplitParagraph",
                    })?;
                Ok(Command::SplitParagraphAt {
                    run_id,
                    offset,
                })
            }
            Command::DeleteParagraph { .. } => {
                let after_id = result
                    .previous_paragraph_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "DeleteParagraph",
                    })?;
                Ok(Command::InsertParagraph { after_id })
            }
            Command::InsertTable { .. }
            | Command::InsertImage { .. }
            | Command::InsertShape { .. }
            | Command::InsertTextBox { .. }
            | Command::InsertWordArt { .. }
            | Command::InsertDiagram { .. }
            | Command::InsertChart { .. }
            | Command::InsertOfficeMathDisplay { .. }
            | Command::InsertPageBreak { .. } => {
                let new_id = result
                    .created_node_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "InsertBlock",
                    })?;
                Ok(Command::DeleteBlock { id: new_id })
            }
            Command::InsertSectionBreak { .. } => {
                let (idx, _) = result.split_section.clone().ok_or(
                    EditError::InverseNotSupported {
                        command: "InsertSectionBreak",
                    },
                )?;
                Ok(Command::MergeSection { section_index: idx })
            }
            Command::MergeSection { .. } => {
                let after_id = result.previous_block_id.ok_or(
                    EditError::InverseNotSupported {
                        command: "MergeSection",
                    },
                )?;
                Ok(Command::InsertSectionBreak {
                    after_block_id: after_id,
                })
            }
            Command::ApplyParagraphStyle { paragraph_id, .. } => {
                let old = result
                    .old_style_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "ApplyParagraphStyle",
                    })?;
                Ok(Command::ApplyParagraphStyleById {
                    paragraph_id: *paragraph_id,
                    style_id: old,
                })
            }
            Command::ApplyParagraphStyleById { paragraph_id, .. } => {
                let old = result
                    .old_style_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "ApplyParagraphStyleById",
                    })?;
                Ok(Command::ApplyParagraphStyleById {
                    paragraph_id: *paragraph_id,
                    style_id: old,
                })
            }
            Command::SetNumbering {
                paragraph_id,
                ..
            } => {
                let old = result
                    .old_numbering
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "SetNumbering",
                    })?;
                Ok(Command::SetNumbering {
                    paragraph_id: *paragraph_id,
                    numbering: old,
                })
            }
            Command::RestartNumbering { paragraph_id, .. }
            | Command::ContinueNumbering { paragraph_id, .. } => {
                let old = result
                    .old_para_format
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "RestartNumbering",
                    })?;
                Ok(Command::SetParaFormat {
                    paragraph_id: *paragraph_id,
                    format: old,
                    merge: false,
                })
            }
            Command::MergeTableCells {
                table_id,
                start_row,
                start_col,
                ..
            } => {
                let (colspan, rowspan) = result
                    .old_cell_span
                    .ok_or(EditError::InverseNotSupported {
                        command: "MergeTableCells",
                    })?;
                Ok(Command::SetTableCellSpan {
                    table_id: *table_id,
                    row: *start_row,
                    col: *start_col,
                    colspan,
                    rowspan,
                })
            }
            Command::SplitTableCell { table_id, row, col } => {
                let (colspan, rowspan) = result
                    .old_cell_span
                    .ok_or(EditError::InverseNotSupported {
                        command: "SplitTableCell",
                    })?;
                Ok(Command::SetTableCellSpan {
                    table_id: *table_id,
                    row: *row,
                    col: *col,
                    colspan,
                    rowspan,
                })
            }
            Command::SetTableCellSpan {
                table_id,
                row,
                col,
                ..
            } => {
                let (colspan, rowspan) = result
                    .old_cell_span
                    .ok_or(EditError::InverseNotSupported {
                        command: "SetTableCellSpan",
                    })?;
                Ok(Command::SetTableCellSpan {
                    table_id: *table_id,
                    row: *row,
                    col: *col,
                    colspan,
                    rowspan,
                })
            }
            Command::ResizeTableColumn {
                table_id,
                column,
                ..
            } => {
                let width = result
                    .old_column_width
                    .ok_or(EditError::InverseNotSupported {
                        command: "ResizeTableColumn",
                    })?;
                Ok(Command::ResizeTableColumn {
                    table_id: *table_id,
                    column: *column,
                    width,
                })
            }
            Command::SetImageSize { image_id, .. } => {
                let (width, height) = result.old_image_size.ok_or(EditError::InverseNotSupported {
                    command: "SetImageSize",
                })?;
                Ok(Command::SetImageSize {
                    image_id: *image_id,
                    width,
                    height,
                })
            }
            Command::SetChartData { shape_id, .. } => {
                let chart_data = result.old_chart_data.clone().ok_or(EditError::InverseNotSupported {
                    command: "SetChartData",
                })?;
                Ok(Command::SetChartData {
                    shape_id: *shape_id,
                    chart_data,
                })
            }
            Command::SetOfficeMath { run_id, .. } => {
                let xml = result.old_office_math_xml.clone().ok_or(EditError::InverseNotSupported {
                    command: "SetOfficeMath",
                })?;
                Ok(Command::SetOfficeMath {
                    run_id: *run_id,
                    xml,
                })
            }
            Command::ReplaceImageBytes { image_id, .. } => {
                let data = result.old_image_data.clone().ok_or(EditError::InverseNotSupported {
                    command: "ReplaceImageBytes",
                })?;
                Ok(Command::ReplaceImageBytes {
                    image_id: *image_id,
                    data,
                })
            }
            Command::SetImageWrap { image_id, .. } => {
                let wrap = result.old_image_wrap.ok_or(EditError::InverseNotSupported {
                    command: "SetImageWrap",
                })?;
                Ok(Command::RestoreImageLayout {
                    image_id: *image_id,
                    wrap,
                    anchor: result.old_image_anchor.flatten(),
                })
            }
            Command::SetImageAnchor { image_id, .. } => {
                let wrap = result.old_image_wrap.ok_or(EditError::InverseNotSupported {
                    command: "SetImageAnchor",
                })?;
                let anchor = result.old_image_anchor.ok_or(EditError::InverseNotSupported {
                    command: "SetImageAnchor",
                })?;
                Ok(Command::RestoreImageLayout {
                    image_id: *image_id,
                    wrap,
                    anchor,
                })
            }
            Command::RestoreImageLayout { image_id, wrap, anchor } => {
                Ok(Command::RestoreImageLayout {
                    image_id: *image_id,
                    wrap: result.old_image_wrap.ok_or(EditError::InverseNotSupported {
                        command: "RestoreImageLayout",
                    })?,
                    anchor: result.old_image_anchor.flatten(),
                })
            }
            Command::SetImageTransform { image_id, .. } => {
                let transform = result.old_image_transform.ok_or(EditError::InverseNotSupported {
                    command: "SetImageTransform",
                })?;
                Ok(Command::SetImageTransform {
                    image_id: *image_id,
                    transform,
                })
            }
            Command::InsertImageCaption { image_id, .. } => {
                let caption_id = result.created_node_id.ok_or(EditError::InverseNotSupported {
                    command: "InsertImageCaption",
                })?;
                Ok(Command::RemoveImageCaption {
                    image_id: *image_id,
                    caption_paragraph_id: caption_id,
                })
            }
            Command::RemoveImageCaption { image_id, .. } => Ok(Command::InsertImageCaption {
                image_id: *image_id,
            }),
            Command::CompressImage { image_id, .. } => {
                let data = result.old_image_data.clone().ok_or(EditError::InverseNotSupported {
                    command: "CompressImage",
                })?;
                Ok(Command::ReplaceImageBytes {
                    image_id: *image_id,
                    data,
                })
            }
            Command::SetTableBorder { table_id, .. } => {
                let old = result.old_table_border.ok_or(EditError::InverseNotSupported {
                    command: "SetTableBorder",
                })?;
                Ok(Command::SetTableBorder {
                    table_id: *table_id,
                    border: old,
                })
            }
            Command::SetTableCellShading {
                table_id,
                row,
                col,
                ..
            } => {
                let old = result.old_cell_background.ok_or(EditError::InverseNotSupported {
                    command: "SetTableCellShading",
                })?;
                Ok(Command::SetTableCellShading {
                    table_id: *table_id,
                    row: *row,
                    col: *col,
                    background: old,
                })
            }
            Command::AutoFitTable { table_id, .. } => {
                let old = result.old_table_column_widths.clone().ok_or(
                    EditError::InverseNotSupported {
                        command: "AutoFitTable",
                    },
                )?;
                Ok(Command::RestoreTableColumnWidths {
                    table_id: *table_id,
                    column_widths: old,
                })
            }
            Command::RestoreTableColumnWidths { table_id, .. } => {
                let old = result.old_table_column_widths.clone().ok_or(
                    EditError::InverseNotSupported {
                        command: "RestoreTableColumnWidths",
                    },
                )?;
                Ok(Command::RestoreTableColumnWidths {
                    table_id: *table_id,
                    column_widths: old,
                })
            }
            Command::SortTableRows { table_id, .. } => {
                let old = result.old_table_rows.clone().ok_or(EditError::InverseNotSupported {
                    command: "SortTableRows",
                })?;
                Ok(Command::RestoreTableRowOrder {
                    table_id: *table_id,
                    rows: old,
                })
            }
            Command::RestoreTableRowOrder { table_id, .. } => {
                let old = result.old_table_rows.clone().ok_or(EditError::InverseNotSupported {
                    command: "RestoreTableRowOrder",
                })?;
                Ok(Command::RestoreTableRowOrder {
                    table_id: *table_id,
                    rows: old,
                })
            }
            Command::InsertNestedTable {
                table_id,
                row,
                col,
                ..
            } => {
                let (_, _, _, block_index) = result.inserted_cell_block.ok_or(
                    EditError::InverseNotSupported {
                        command: "InsertNestedTable",
                    },
                )?;
                Ok(Command::RemoveTableCellBlock {
                    table_id: *table_id,
                    row: *row,
                    col: *col,
                    block_index,
                })
            }
            Command::RemoveTableCellBlock {
                table_id,
                row,
                col,
                block_index,
            } => {
                let (_, _, _, _, block) = result.removed_cell_block.clone().ok_or(
                    EditError::InverseNotSupported {
                        command: "RemoveTableCellBlock",
                    },
                )?;
                Ok(Command::InsertTableCellBlock {
                    table_id: *table_id,
                    row: *row,
                    col: *col,
                    block_index: *block_index,
                    block,
                })
            }
            Command::InsertTableCellBlock {
                table_id,
                row,
                col,
                block_index,
                ..
            } => Ok(Command::RemoveTableCellBlock {
                table_id: *table_id,
                row: *row,
                col: *col,
                block_index: *block_index,
            }),
            Command::DeleteTableRow { table_id, row } => {
                let (_, _, row_data) = result.deleted_table_row.clone().ok_or(
                    EditError::InverseNotSupported {
                        command: "DeleteTableRow",
                    },
                )?;
                Ok(Command::RestoreTableRow {
                    table_id: *table_id,
                    row: *row,
                    row_data,
                })
            }
            Command::RestoreTableRow { table_id, row, .. } => Ok(Command::DeleteTableRow {
                table_id: *table_id,
                row: *row,
            }),
            Command::DeleteTableColumn { table_id, column } => {
                let (_, _, cells, width) = result.deleted_table_column.clone().ok_or(
                    EditError::InverseNotSupported {
                        command: "DeleteTableColumn",
                    },
                )?;
                Ok(Command::RestoreTableColumn {
                    table_id: *table_id,
                    column: *column,
                    cells,
                    column_width: width,
                })
            }
            Command::RestoreTableColumn { table_id, column, .. } => {
                Ok(Command::DeleteTableColumn {
                    table_id: *table_id,
                    column: *column,
                })
            }
            Command::FindReplace { .. } => result
                .find_replace_undo
                .as_ref()
                .map(|segments| Command::RestoreFindReplace {
                    segments: segments.clone(),
                })
                .ok_or(EditError::InverseNotSupported {
                    command: "FindReplace",
                }),
            Command::RestoreFindReplace { .. } => Err(EditError::InverseNotSupported {
                command: "RestoreFindReplace",
            }),
            Command::DeleteBlock { .. } => {
                let block = result
                    .deleted_block
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "DeleteBlock",
                    })?;
                if let Some(before_id) = result.insert_before_block_id {
                    Ok(Command::InsertBlockBefore {
                        before_block_id: before_id,
                        block,
                    })
                } else {
                    let after_id = result.previous_block_id.ok_or(
                        EditError::InverseNotSupported {
                            command: "DeleteBlock",
                        },
                    )?;
                    Ok(Command::InsertBlock {
                        after_block_id: after_id,
                        block,
                    })
                }
            }
            Command::InsertBlock { .. } | Command::InsertBlockBefore { .. } => {
                let new_id = result
                    .created_node_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "InsertBlock",
                    })?;
                Ok(Command::DeleteBlock { id: new_id })
            }
            Command::AcceptRevision { .. }
            | Command::RejectRevision { .. }
            | Command::AcceptAllRevisions
            | Command::RejectAllRevisions => {
                let snapshots = result
                    .revision_snapshots
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "RevisionResolution",
                    })?;
                Ok(Command::RestoreRevisionRuns { snapshots })
            }
            Command::CreateParagraphStyle { .. } => {
                let style_id = result
                    .created_style_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "CreateParagraphStyle",
                    })?;
                Ok(Command::DeleteParagraphStyle { style_id })
            }
            Command::DeleteParagraphStyle { .. } => {
                let style = result
                    .deleted_style
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "DeleteParagraphStyle",
                    })?;
                let ooxml_id = result
                    .deleted_style_ooxml_id
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "DeleteParagraphStyle",
                    })?;
                let assignments = result
                    .style_usage_updates
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(para_id, old)| (para_id, old.unwrap_or(style.id)))
                    .collect();
                Ok(Command::RestoreParagraphStyle {
                    style,
                    ooxml_style_id: ooxml_id,
                    paragraph_assignments: assignments,
                })
            }
            Command::RenameParagraphStyle { style_name: _, new_name } => {
                let (_style_id, old_name) = result
                    .style_renamed
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "RenameParagraphStyle",
                    })?;
                Ok(Command::RenameParagraphStyle {
                    style_name: new_name.clone(),
                    new_name: old_name,
                })
            }
            Command::RestoreParagraphStyle { .. } => Err(EditError::InverseNotSupported {
                command: "RestoreParagraphStyle",
            }),
            Command::SetDocumentTheme { .. } => {
                let old = result
                    .old_document_theme
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "SetDocumentTheme",
                    })?;
                Ok(Command::SetDocumentTheme {
                    theme_name: old.name,
                })
            }
            Command::SetEvenAndOddHeaders { .. } => {
                let old = result.old_even_and_odd_headers.ok_or(
                    EditError::InverseNotSupported {
                        command: "SetEvenAndOddHeaders",
                    },
                )?;
                Ok(Command::SetEvenAndOddHeaders { enabled: old })
            }
            Command::SetSectionFormat { section_index, .. } => {
                let (idx, old) = result.old_section_format.clone().ok_or(
                    EditError::InverseNotSupported {
                        command: "SetSectionFormat",
                    },
                )?;
                Ok(Command::SetSectionFormat {
                    section_index: idx,
                    format: old,
                })
            }
            Command::RestoreRevisionRuns { .. } => Err(EditError::InverseNotSupported {
                command: "RestoreRevisionRuns",
            }),
            Command::EnsureHeaderFooter { .. } => Err(EditError::InverseNotSupported {
                command: "EnsureHeaderFooter",
            }),
            Command::SetHeaderFooterLink {
                section_index,
                is_header,
                hf_type,
                linked: _,
            } => {
                let (idx, header, old) = result.old_header_footer_links.clone().ok_or(
                    EditError::InverseNotSupported {
                        command: "SetHeaderFooterLink",
                    },
                )?;
                if idx != *section_index || header != *is_header {
                    return Err(EditError::InverseNotSupported {
                        command: "SetHeaderFooterLink",
                    });
                }
                Ok(Command::SetHeaderFooterLink {
                    section_index: idx,
                    is_header: header,
                    hf_type: *hf_type,
                    linked: old.is_linked(*hf_type),
                })
            }
            Command::InsertField { .. } => Err(EditError::InverseNotSupported {
                command: "InsertField",
            }),
            Command::InsertFootnote { .. } => Err(EditError::InverseNotSupported {
                command: "InsertFootnote",
            }),
            Command::InsertComment { .. } => Err(EditError::InverseNotSupported {
                command: "InsertComment",
            }),
            Command::InsertTableOfContents { .. } => Err(EditError::InverseNotSupported {
                command: "InsertTableOfContents",
            }),
            Command::AddBibliographySource { .. } => Err(EditError::InverseNotSupported {
                command: "AddBibliographySource",
            }),
            Command::InsertCitation { .. } => Err(EditError::InverseNotSupported {
                command: "InsertCitation",
            }),
            Command::InsertBibliography { .. } => Err(EditError::InverseNotSupported {
                command: "InsertBibliography",
            }),
            Command::InsertBookmark { .. } => Err(EditError::InverseNotSupported {
                command: "InsertBookmark",
            }),
            Command::InsertCrossReference { .. } => Err(EditError::InverseNotSupported {
                command: "InsertCrossReference",
            }),
            Command::InsertIndex { .. } => Err(EditError::InverseNotSupported {
                command: "InsertIndex",
            }),
            Command::InsertOfficeMath { .. } => Err(EditError::InverseNotSupported {
                command: "InsertOfficeMath",
            }),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditResult {
    pub affected_nodes: Vec<NodeId>,
    pub deleted_text: Option<String>,
    pub old_char_format: Option<CharFormat>,
    pub old_para_format: Option<ParaFormat>,
    /// Per-run format snapshots for multi-run character formatting undo.
    pub old_run_formats: Vec<(NodeId, CharFormat)>,
    /// Per-paragraph format snapshots for multi-paragraph formatting undo.
    pub old_para_formats: Vec<(NodeId, ParaFormat)>,
    pub created_node_id: Option<NodeId>,
    pub previous_paragraph_id: Option<NodeId>,
    pub deleted_paragraph: Option<tw_model::Paragraph>,
    pub old_style_id: Option<Option<StyleId>>,
    pub old_numbering: Option<Option<NumberingRef>>,
    pub previous_block_id: Option<NodeId>,
    /// When set, undo of DeleteBlock inserts before this id (first-block delete).
    pub insert_before_block_id: Option<NodeId>,
    pub deleted_block: Option<tw_model::Block>,
    pub deleted_table_row: Option<(NodeId, usize, tw_model::TableRow)>,
    pub deleted_table_column: Option<(NodeId, usize, Vec<tw_model::TableCell>, f32)>,
    pub old_cell_span: Option<(u32, u32)>,
    pub old_column_width: Option<f32>,
    pub old_table_border: Option<Option<tw_model::BorderSpec>>,
    pub old_cell_background: Option<Option<tw_model::Color>>,
    pub old_table_column_widths: Option<Vec<f32>>,
    pub old_table_rows: Option<Vec<tw_model::TableRow>>,
    pub inserted_cell_block: Option<(NodeId, u32, u32, usize)>,
    pub removed_cell_block: Option<(NodeId, u32, u32, usize, tw_model::Block)>,
    pub old_image_size: Option<(f32, f32)>,
    pub old_image_data: Option<tw_model::ImageData>,
    pub old_image_wrap: Option<tw_model::TextWrap>,
    pub old_image_anchor: Option<Option<tw_model::ImageAnchor>>,
    pub old_image_transform: Option<tw_model::ImageTransform>,
    pub old_image_caption_id: Option<Option<NodeId>>,
    pub find_replace_undo: Option<Vec<(NodeId, usize, String, String)>>,
    /// Number of substitutions performed by the last [`Command::FindReplace`].
    pub replacement_count: usize,
    /// Character boundary to re-split when undoing/redoing paragraph merges.
    pub split_boundary: Option<(NodeId, usize)>,
    /// Run text + revision snapshots for accept/reject undo.
    pub revision_snapshots: Option<Vec<RevisionRunSnapshot>>,
    pub created_style_id: Option<StyleId>,
    pub deleted_style: Option<tw_model::ParagraphStyle>,
    pub deleted_style_ooxml_id: Option<String>,
    pub style_renamed: Option<(StyleId, String)>,
    pub style_usage_updates: Option<Vec<(NodeId, Option<StyleId>)>>,
    pub old_document_theme: Option<tw_model::DocumentTheme>,
    pub old_even_and_odd_headers: Option<bool>,
    pub old_section_format: Option<(usize, tw_model::SectionFormat)>,
    pub old_header_footer_links: Option<(usize, bool, tw_model::HeaderFooterLinks)>,
    /// Section inserted by [`Command::InsertSectionBreak`] for undo/redo.
    pub split_section: Option<(usize, tw_model::Section)>,
    /// Seed run for header/footer editing (F08.S1).
    pub seed_run_id: Option<NodeId>,
    pub old_chart_data: Option<Option<tw_model::ChartData>>,
    pub old_office_math_xml: Option<String>,
}

fn default_insert_shape_style() -> tw_model::ShapeStyle {
    tw_model::ShapeStyle::inserted_default()
}

#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error("run not found: {0}")]
    RunNotFound(NodeId),
    #[error("paragraph not found: {0}")]
    ParagraphNotFound(NodeId),
    #[error("block not found: {0}")]
    BlockNotFound(NodeId),
    #[error("table not found: {0}")]
    TableNotFound(NodeId),
    #[error("style not found: {0}")]
    StyleNotFound(String),
    #[error("duplicate style name: {0}")]
    DuplicateStyleName(String),
    #[error("built-in style is protected: {0}")]
    BuiltinStyleProtected(String),
    #[error("theme not found: {0}")]
    ThemeNotFound(String),
    #[error("invalid range")]
    InvalidRange,
    #[error("invalid regex: {0}")]
    InvalidRegex(String),
    #[error("invalid image data: {0}")]
    InvalidImageData(String),
    #[error("inverse not supported for command: {command}")]
    InverseNotSupported { command: &'static str },
}
