use tw_model::{Block, Document, ImageBlock, NodeId, Paragraph, Table};
use tw_text::TextBuffer;

use crate::{EditError, EditResult};

pub fn insert_table(
    doc: &mut Document,
    after_block_id: NodeId,
    rows: u32,
    cols: u32,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let table = Table::new(rows, cols);
    let new_id = table.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::Table(table));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn insert_image(
    doc: &mut Document,
    after_block_id: NodeId,
    width: f32,
    height: f32,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let image = ImageBlock::placeholder(width, height);
    let new_id = image.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::ImageBlock(image));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn insert_page_break(
    doc: &mut Document,
    after_block_id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let mut para = Paragraph::new();
    para.format.page_break_before = Some(true);
    let new_id = para.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::Paragraph(para));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn merge_table_cells(
    doc: &mut Document,
    table_id: NodeId,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc.block_at_mut(si, bi).ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let sr = start_row as usize;
    let sc = start_col as usize;
    let er = end_row as usize;
    let ec = end_col as usize;

    if let Some(row) = table.rows.get_mut(sr) {
        if let Some(cell) = row.cells.get_mut(sc) {
            let old_colspan = cell.format.colspan;
            let old_rowspan = cell.format.rowspan;
            cell.format.colspan = (ec - sc + 1) as u32;
            cell.format.rowspan = (er - sr + 1) as u32;
            return Ok(EditResult {
                affected_nodes: vec![table_id],
                old_cell_span: Some((old_colspan, old_rowspan)),
                ..Default::default()
            });
        }
    }

    Ok(EditResult {
        affected_nodes: vec![table_id],
        ..Default::default()
    })
}

pub fn set_table_cell_span(
    doc: &mut Document,
    table_id: NodeId,
    row: u32,
    col: u32,
    colspan: u32,
    rowspan: u32,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc.block_at_mut(si, bi).ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let ri = row as usize;
    let ci = col as usize;
    if let Some(cell) = table.rows.get_mut(ri).and_then(|r| r.cells.get_mut(ci)) {
        let old_colspan = cell.format.colspan;
        let old_rowspan = cell.format.rowspan;
        cell.format.colspan = colspan.max(1);
        cell.format.rowspan = rowspan.max(1);
        return Ok(EditResult {
            affected_nodes: vec![table_id],
            old_cell_span: Some((old_colspan, old_rowspan)),
            ..Default::default()
        });
    }

    Err(EditError::TableNotFound(table_id))
}

pub fn resize_table_column(
    doc: &mut Document,
    table_id: NodeId,
    column: u32,
    width: f32,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc.block_at_mut(si, bi).ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let col = column as usize;
    if col < table.format.column_widths.len() {
        let old_width = table.format.column_widths[col];
        table.format.column_widths[col] = width;
        table.format.width = Some(table.format.column_widths.iter().sum());
        return Ok(EditResult {
            affected_nodes: vec![table_id],
            old_column_width: Some(old_width),
            ..Default::default()
        });
    }

    Ok(EditResult {
        affected_nodes: vec![table_id],
        ..Default::default()
    })
}

pub fn delete_block(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(id)
        .ok_or(EditError::BlockNotFound(id))?;

    if doc.sections[si].blocks.len() <= 1 {
        return Err(EditError::InvalidRange);
    }

    let after_id = if bi > 0 {
        let prev = &doc.sections[si].blocks[bi - 1];
        match prev {
            Block::Paragraph(p) => p.id,
            Block::Table(t) => t.id,
            Block::ImageBlock(i) => i.id,
        }
    } else {
        return Err(EditError::InvalidRange);
    };

    let block = doc.sections[si].blocks.remove(bi);
    if let Block::Paragraph(ref p) = block {
        for run in &p.runs {
            buffer.unregister(run.id);
        }
    }

    Ok(EditResult {
        affected_nodes: vec![id],
        previous_block_id: Some(after_id),
        deleted_block: Some(block),
        ..Default::default()
    })
}

pub fn insert_block(
    doc: &mut Document,
    buffer: &mut TextBuffer,
    after_block_id: NodeId,
    block: Block,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let new_id = match &block {
        Block::Paragraph(p) => {
            for run in &p.runs {
                buffer.register(run.id, run.text());
            }
            p.id
        }
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
    };

    doc.sections[si].blocks.insert(bi + 1, block);

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}
