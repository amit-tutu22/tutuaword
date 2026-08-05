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
    SetCharFormat {
        run_id: NodeId,
        start: usize,
        end: usize,
        format: CharFormat,
        merge: bool,
    },
    SetParaFormat {
        paragraph_id: NodeId,
        format: ParaFormat,
        merge: bool,
    },
    InsertParagraph {
        after_id: NodeId,
    },
    DeleteParagraph {
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
            Command::SetCharFormat {
                run_id,
                start,
                end,
                ..
            } => {
                let old = result.old_char_format.clone()?;
                Some(Command::SetCharFormat {
                    run_id: *run_id,
                    start: *start,
                    end: *end,
                    format: old,
                    merge: false,
                })
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
            Command::InsertParagraph { .. } => {
                let new_id = result.created_node_id?;
                Some(Command::DeleteParagraph { id: new_id })
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
            Command::MergeTableCells { .. } | Command::ResizeTableColumn { .. } => None,
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
    pub created_node_id: Option<NodeId>,
    pub previous_paragraph_id: Option<NodeId>,
    pub deleted_paragraph: Option<tw_model::Paragraph>,
    pub old_style_id: Option<Option<StyleId>>,
    pub old_numbering: Option<Option<NumberingRef>>,
    pub previous_block_id: Option<NodeId>,
    pub deleted_block: Option<tw_model::Block>,
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
