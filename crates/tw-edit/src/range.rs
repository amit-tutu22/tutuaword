//! Document-order ranges spanning one or more runs / paragraphs.

use crate::command::{DocPosition, DocRange};
use crate::EditError;
use tw_model::{Block, Document, NodeId};

/// Order `range` so `start` precedes `end` in document order.
pub fn normalize_range(doc: &Document, range: &DocRange) -> Result<DocRange, EditError> {
    let start_ord = position_ord(doc, &range.start)?;
    let end_ord = position_ord(doc, &range.end)?;
    if start_ord <= end_ord {
        Ok(range.clone())
    } else {
        Ok(DocRange {
            start: range.end.clone(),
            end: range.start.clone(),
        })
    }
}

pub fn positions_equal(a: &DocPosition, b: &DocPosition) -> bool {
    a.run_id == b.run_id && a.char_offset == b.char_offset
}

/// `(section, block, run_index, char_offset)` for sorting.
fn position_ord(
    doc: &Document,
    pos: &DocPosition,
) -> Result<(usize, usize, usize, usize), EditError> {
    let (si, bi, ri) = doc
        .find_run_location(pos.run_id)
        .ok_or(EditError::RunNotFound(pos.run_id))?;
    Ok((si, bi, ri, pos.char_offset))
}

/// Paragraph containing `run_id`.
pub fn paragraph_id_for_run(doc: &Document, run_id: NodeId) -> Result<NodeId, EditError> {
    let (si, bi, _) = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;
    doc.paragraph_at(si, bi)
        .map(|p| p.id)
        .ok_or(EditError::RunNotFound(run_id))
}

/// Every paragraph id touched by `range` (already normalized).
pub fn paragraph_ids_in_range(doc: &Document, range: &DocRange) -> Result<Vec<NodeId>, EditError> {
    let start = doc
        .find_run_location(range.start.run_id)
        .ok_or(EditError::RunNotFound(range.start.run_id))?;
    let end = doc
        .find_run_location(range.end.run_id)
        .ok_or(EditError::RunNotFound(range.end.run_id))?;

    let mut ids = Vec::new();
    for (si, section) in doc.sections.iter().enumerate() {
        for (bi, block) in section.blocks.iter().enumerate() {
            let Block::Paragraph(p) = block else {
                continue;
            };
            let before_start = (si, bi) < (start.0, start.1);
            let after_end = (si, bi) > (end.0, end.1);
            if before_start || after_end {
                continue;
            }
            ids.push(p.id);
        }
    }
    Ok(ids)
}
