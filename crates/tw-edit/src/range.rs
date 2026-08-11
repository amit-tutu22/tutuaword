//! Document-order ranges spanning one or more runs / paragraphs.

use crate::command::{DocPosition, DocRange};
use crate::EditError;
use tw_model::{Block, BlockZone, Document, NodeId, RunLocation};

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

/// Sort key for positions: section, zone band, block, run, char offset.
fn position_ord(
    doc: &Document,
    pos: &DocPosition,
) -> Result<(usize, u8, usize, usize, usize), EditError> {
    let loc = doc
        .find_run_location(pos.run_id)
        .ok_or(EditError::RunNotFound(pos.run_id))?;
    Ok((
        loc.section_index,
        loc.zone.sort_key(),
        loc.block_index,
        loc.run_index,
        pos.char_offset,
    ))
}

/// Paragraph containing `run_id`.
pub fn paragraph_id_for_run(doc: &Document, run_id: NodeId) -> Result<NodeId, EditError> {
    let loc = doc
        .find_run_location(run_id)
        .ok_or(EditError::RunNotFound(run_id))?;
    doc.paragraph_at_loc(loc)
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
        collect_paragraph_ids_in_zone(
            &mut ids,
            si,
            BlockZone::Body,
            &section.blocks,
            start,
            end,
        );
        for (kind, hf) in &section.headers {
            collect_paragraph_ids_in_zone(
                &mut ids,
                si,
                BlockZone::Header(*kind),
                &hf.blocks,
                start,
                end,
            );
        }
        for (kind, hf) in &section.footers {
            collect_paragraph_ids_in_zone(
                &mut ids,
                si,
                BlockZone::Footer(*kind),
                &hf.blocks,
                start,
                end,
            );
        }
    }
    Ok(ids)
}

fn collect_paragraph_ids_in_zone(
    ids: &mut Vec<NodeId>,
    section_index: usize,
    zone: BlockZone,
    blocks: &[Block],
    start: RunLocation,
    end: RunLocation,
) {
    for (bi, block) in blocks.iter().enumerate() {
        let Block::Paragraph(p) = block else {
            continue;
        };
        let loc = RunLocation {
            section_index,
            zone,
            block_index: bi,
            run_index: 0,
            table_cell: None,
            shape_paragraph: None,
        };
        if location_before(loc, start) || location_after(loc, end) {
            continue;
        }
        ids.push(p.id);
    }
}

fn location_before(a: RunLocation, b: RunLocation) -> bool {
    (a.section_index, a.zone.sort_key(), a.block_index) < (b.section_index, b.zone.sort_key(), b.block_index)
}

fn location_after(a: RunLocation, b: RunLocation) -> bool {
    (a.section_index, a.zone.sort_key(), a.block_index) > (b.section_index, b.zone.sort_key(), b.block_index)
}
