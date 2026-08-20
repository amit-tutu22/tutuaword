//! Track-change revision navigation helpers (F17.S2).

use serde::Serialize;

use crate::{Block, BlockZone, Document, NodeId, RevisionType, RunLocation};

/// One tracked change for the Changes pane / FFI JSON (F17.S2).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RevisionEntry {
    pub run_id: String,
    pub revision_type: String,
    pub author: String,
    pub preview: String,
}

/// JSON array of [`RevisionEntry`] for all revision-marked runs.
pub fn revision_entries_json(doc: &Document) -> Option<String> {
    let entries = revision_entries(doc);
    serde_json::to_string(&entries).ok()
}

pub fn revision_entries(doc: &Document) -> Vec<RevisionEntry> {
    let mut entries = Vec::new();
    for (si, section) in doc.sections.iter().enumerate() {
        collect_revision_entries_in_blocks(&section.blocks, si, BlockZone::Body, &mut entries);
    }
    entries
        .into_iter()
        .map(|(_, entry)| entry)
        .collect()
}

/// Run id if [run_id] carries a revision marker.
pub fn revision_at_run(doc: &Document, run_id: NodeId) -> Option<NodeId> {
    doc.run_by_id(run_id)
        .and_then(|run| run.revision.as_ref().map(|_| run_id))
}

/// Revision-marked runs in stable document order.
pub fn revision_run_ids(doc: &Document) -> Vec<NodeId> {
    let mut entries = Vec::new();
    for (si, section) in doc.sections.iter().enumerate() {
        collect_revision_runs_in_blocks(&section.blocks, si, BlockZone::Body, &mut entries);
    }
    entries.sort_by_key(|(loc, _)| *loc);
    entries.into_iter().map(|(_, id)| id).collect()
}

/// Next or previous revision run relative to the caret (wraps at document ends).
pub fn adjacent_revision_run(
    doc: &Document,
    caret_run_id: NodeId,
    forward: bool,
) -> Option<NodeId> {
    let ids = revision_run_ids(doc);
    if ids.is_empty() {
        return None;
    }
    if let Some(idx) = ids.iter().position(|&id| id == caret_run_id) {
        let next = if forward {
            (idx + 1) % ids.len()
        } else {
            idx.checked_sub(1).unwrap_or(ids.len() - 1)
        };
        return Some(ids[next]);
    }
    let caret_loc = doc.find_run_location(caret_run_id)?;
    if forward {
        for id in &ids {
            let loc = doc.find_run_location(*id)?;
            if loc > caret_loc {
                return Some(*id);
            }
        }
        return Some(ids[0]);
    }
    for id in ids.iter().rev() {
        let loc = doc.find_run_location(*id)?;
        if loc < caret_loc {
            return Some(*id);
        }
    }
    ids.last().copied()
}

fn collect_revision_runs_in_blocks(
    blocks: &[Block],
    section_index: usize,
    zone: BlockZone,
    out: &mut Vec<(RunLocation, NodeId)>,
) {
    for (block_index, block) in blocks.iter().enumerate() {
        match block {
            Block::Paragraph(para) => {
                for (run_index, run) in para.runs.iter().enumerate() {
                    if run.revision.is_some() {
                        out.push((
                            RunLocation {
                                section_index,
                                zone,
                                block_index,
                                run_index,
                                table_cell: None,
                                shape_paragraph: None,
                            },
                            run.id,
                        ));
                    }
                }
            }
            Block::Table(table) => {
                for (row, table_row) in table.rows.iter().enumerate() {
                    for (cell, cell_block) in table_row.cells.iter().enumerate() {
                        for (block_in_cell, cell_block) in cell_block.blocks.iter().enumerate() {
                            if let Block::Paragraph(para) = cell_block {
                                for (run_index, run) in para.runs.iter().enumerate() {
                                    if run.revision.is_some() {
                                        out.push((
                                            RunLocation {
                                                section_index,
                                                zone,
                                                block_index,
                                                run_index,
                                                table_cell: Some((row, cell, block_in_cell)),
                                                shape_paragraph: None,
                                            },
                                            run.id,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Block::ShapeBlock(shape) => {
                for (para_index, para) in shape.paragraphs.iter().enumerate() {
                    for (run_index, run) in para.runs.iter().enumerate() {
                        if run.revision.is_some() {
                            out.push((
                                RunLocation {
                                    section_index,
                                    zone,
                                    block_index,
                                    run_index,
                                    table_cell: None,
                                    shape_paragraph: Some(para_index),
                                },
                                run.id,
                            ));
                        }
                    }
                }
            }
            Block::ImageBlock(_) => {}
        }
    }
}

fn collect_revision_entries_in_blocks(
    blocks: &[Block],
    section_index: usize,
    zone: BlockZone,
    out: &mut Vec<(RunLocation, RevisionEntry)>,
) {
    for (block_index, block) in blocks.iter().enumerate() {
        match block {
            Block::Paragraph(para) => {
                for (run_index, run) in para.runs.iter().enumerate() {
                    if let Some(entry) = revision_entry_from_run(run) {
                        out.push((
                            RunLocation {
                                section_index,
                                zone,
                                block_index,
                                run_index,
                                table_cell: None,
                                shape_paragraph: None,
                            },
                            entry,
                        ));
                    }
                }
            }
            Block::Table(table) => {
                for (row, table_row) in table.rows.iter().enumerate() {
                    for (cell, cell_block) in table_row.cells.iter().enumerate() {
                        for (block_in_cell, cell_block) in cell_block.blocks.iter().enumerate() {
                            if let Block::Paragraph(para) = cell_block {
                                for (run_index, run) in para.runs.iter().enumerate() {
                                    if let Some(entry) = revision_entry_from_run(run) {
                                        out.push((
                                            RunLocation {
                                                section_index,
                                                zone,
                                                block_index,
                                                run_index,
                                                table_cell: Some((row, cell, block_in_cell)),
                                                shape_paragraph: None,
                                            },
                                            entry,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Block::ShapeBlock(shape) => {
                for (para_index, para) in shape.paragraphs.iter().enumerate() {
                    for (run_index, run) in para.runs.iter().enumerate() {
                        if let Some(entry) = revision_entry_from_run(run) {
                            out.push((
                                RunLocation {
                                    section_index,
                                    zone,
                                    block_index,
                                    run_index,
                                    table_cell: None,
                                    shape_paragraph: Some(para_index),
                                },
                                entry,
                            ));
                        }
                    }
                }
            }
            Block::ImageBlock(_) => {}
        }
    }
    out.sort_by_key(|(loc, _)| *loc);
}

fn revision_entry_from_run(run: &crate::Run) -> Option<RevisionEntry> {
    let revision = run.revision.as_ref()?;
    let preview = run.text().chars().take(80).collect::<String>();
    Some(RevisionEntry {
        run_id: run.id.to_string(),
        revision_type: match revision.revision_type {
            RevisionType::Insert => "insert".into(),
            RevisionType::Delete => "delete".into(),
        },
        author: revision.author.clone(),
        preview,
    })
}
