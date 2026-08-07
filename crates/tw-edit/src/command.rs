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
            | Command::InsertPageBreak { .. } => {
                let new_id = result
                    .created_node_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "InsertBlock",
                    })?;
                Ok(Command::DeleteBlock { id: new_id })
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
                let after_id = result
                    .previous_block_id
                    .ok_or(EditError::InverseNotSupported {
                        command: "DeleteBlock",
                    })?;
                let block = result
                    .deleted_block
                    .clone()
                    .ok_or(EditError::InverseNotSupported {
                        command: "DeleteBlock",
                    })?;
                Ok(Command::InsertBlock {
                    after_block_id: after_id,
                    block,
                })
            }
            Command::InsertBlock { .. } => {
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
            Command::RestoreRevisionRuns { .. } => Err(EditError::InverseNotSupported {
                command: "RestoreRevisionRuns",
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
    pub deleted_block: Option<tw_model::Block>,
    pub old_cell_span: Option<(u32, u32)>,
    pub old_column_width: Option<f32>,
    pub find_replace_undo: Option<Vec<(NodeId, usize, String, String)>>,
    /// Character boundary to re-split when undoing/redoing paragraph merges.
    pub split_boundary: Option<(NodeId, usize)>,
    /// Run text + revision snapshots for accept/reject undo.
    pub revision_snapshots: Option<Vec<RevisionRunSnapshot>>,
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
    #[error("inverse not supported for command: {command}")]
    InverseNotSupported { command: &'static str },
}
