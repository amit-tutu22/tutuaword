use tw_model::{CharFormat, NodeId, NumberingRef, ParaFormat, StyleId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocPosition {
    pub run_id: NodeId,
    pub char_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocRange {
    pub start: DocPosition,
    pub end: DocPosition,
}

#[derive(Debug, Clone)]
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
    InsertPageBreak {
        after_block_id: NodeId,
    },
    MergeTableCells {
        table_id: NodeId,
        start_row: u32,
        start_col: u32,
        end_row: u32,
        end_col: u32,
    },
    ResizeTableColumn {
        table_id: NodeId,
        column: u32,
        width: f32,
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
}

impl Command {
    pub fn inverse(&self, result: &EditResult) -> Option<Command> {
        match self {
            Command::InsertText { run_id, offset, text } => Some(Command::DeleteRange {
                run_id: *run_id,
                start: *offset,
                end: offset + text.chars().count(),
            }),
            Command::DeleteRange { run_id, start, end: _ } => {
                let deleted = result.deleted_text.clone()?;
                Some(Command::InsertText {
                    run_id: *run_id,
                    offset: *start,
                    text: deleted,
                })
            }
            Command::DeleteDocRange { .. } => {
                let segments = result.find_replace_undo.clone()?;
                Some(Command::RestoreFindReplace { segments })
            }
            Command::SetCharFormat { .. } => {
                if !result.old_run_formats.is_empty() {
                    Some(Command::RestoreRunFormats {
                        formats: result.old_run_formats.clone(),
                    })
                } else {
                    let old = result.old_char_format.clone()?;
                    let run_id = result.affected_nodes.first().copied()?;
                    Some(Command::SetCharFormat {
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
                    None
                } else {
                    Some(Command::RestoreRunFormats {
                        formats: result.old_run_formats.clone(),
                    })
                }
            }
            Command::SetParaFormat {
                paragraph_id,
                ..
            } => {
                let old = result.old_para_format.clone()?;
                Some(Command::SetParaFormat {
                    paragraph_id: *paragraph_id,
                    format: old,
                    merge: false,
                })
            }
            Command::SetParaFormatRange { .. } => {
                if result.old_para_formats.is_empty() {
                    None
                } else {
                    Some(Command::RestoreParaFormats {
                        formats: result.old_para_formats.clone(),
                    })
                }
            }
            Command::RestoreRunFormats { .. } | Command::RestoreParaFormats { .. } => None,
            Command::InsertParagraph { .. } => {
                let new_id = result.created_node_id?;
                Some(Command::DeleteParagraph { id: new_id })
            }
            // Undo Enter by merging the new paragraph back into its predecessor.
            Command::SplitParagraphAt { .. } => {
                let new_id = result.created_node_id?;
                Some(Command::MergeSplitParagraph { id: new_id })
            }
            Command::MergeSplitParagraph { .. } => {
                let (run_id, offset) = result.split_boundary?;
                Some(Command::SplitParagraphAt {
                    run_id,
                    offset,
                })
            }
            Command::DeleteParagraph { .. } => {
                let after_id = result.previous_paragraph_id?;
                Some(Command::InsertParagraph { after_id })
            }
            Command::InsertTable { .. }
            | Command::InsertImage { .. }
            | Command::InsertPageBreak { .. } => {
                let new_id = result.created_node_id?;
                Some(Command::DeleteBlock { id: new_id })
            }
            Command::ApplyParagraphStyle { paragraph_id, .. } => {
                let old = result.old_style_id?;
                Some(Command::ApplyParagraphStyleById {
                    paragraph_id: *paragraph_id,
                    style_id: old,
                })
            }
            Command::ApplyParagraphStyleById { .. } => None,
            Command::SetNumbering {
                paragraph_id,
                ..
            } => {
                let old = result.old_numbering.clone()?;
                Some(Command::SetNumbering {
                    paragraph_id: *paragraph_id,
                    numbering: old,
                })
            }
            Command::MergeTableCells {
                table_id,
                start_row,
                start_col,
                ..
            } => {
                let (colspan, rowspan) = result.old_cell_span?;
                Some(Command::SetTableCellSpan {
                    table_id: *table_id,
                    row: *start_row,
                    col: *start_col,
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
                let (colspan, rowspan) = result.old_cell_span?;
                Some(Command::SetTableCellSpan {
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
                let width = result.old_column_width?;
                Some(Command::ResizeTableColumn {
                    table_id: *table_id,
                    column: *column,
                    width,
                })
            }
            Command::FindReplace { .. } => result.find_replace_undo.as_ref().map(|segments| {
                Command::RestoreFindReplace {
                    segments: segments.clone(),
                }
            }),
            Command::RestoreFindReplace { .. } => None,
            Command::DeleteBlock { .. } => {
                let after_id = result.previous_block_id?;
                let block = result.deleted_block.clone()?;
                Some(Command::InsertBlock {
                    after_block_id: after_id,
                    block,
                })
            }
            Command::InsertBlock { .. } => None,
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
    pub deleted_block: Option<tw_model::Block>,
    pub old_cell_span: Option<(u32, u32)>,
    pub old_column_width: Option<f32>,
    pub find_replace_undo: Option<Vec<(NodeId, usize, String, String)>>,
    /// Character boundary to re-split when undoing/redoing paragraph merges.
    pub split_boundary: Option<(NodeId, usize)>,
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
    #[error("invalid range")]
    InvalidRange,
}
