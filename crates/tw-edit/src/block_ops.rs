use tw_model::{
    cell_text_at_grid, Block, Document, HeaderFooter, HeaderFooterLinks, HeaderFooterType,
    ImageBlock, NodeId, Paragraph, Run, Section, Table, TableRow,
};

use crate::{EditError, EditResult};

pub fn ensure_header_footer(
    doc: &mut Document,
    section_index: usize,
    is_header: bool,
    hf_type: HeaderFooterType,
) -> Result<EditResult, EditError> {
    if section_index > 0 {
        unlink_header_footer_if_needed(doc, section_index, is_header, hf_type)?;
    }

    let section = doc
        .sections
        .get_mut(section_index)
        .ok_or(EditError::InvalidRange)?;
    section.migrate_legacy_headers_footers();

    let map = if is_header {
        &mut section.headers
    } else {
        &mut section.footers
    };

    if !map.contains_key(&hf_type) {
        let para = Paragraph::new();
        let seed_run_id = para.runs[0].id;
        map.insert(
            hf_type,
            HeaderFooter {
                blocks: vec![Block::Paragraph(para)],
                plain_text: None,
            },
        );
        return Ok(EditResult {
            affected_nodes: vec![seed_run_id],
            seed_run_id: Some(seed_run_id),
            ..Default::default()
        });
    }

    let seed_run_id = doc
        .header_footer_seed_run(section_index, is_header, hf_type)
        .ok_or(EditError::InvalidRange)?;
    Ok(EditResult {
        affected_nodes: vec![seed_run_id],
        seed_run_id: Some(seed_run_id),
        ..Default::default()
    })
}

pub fn set_header_footer_link(
    doc: &mut Document,
    section_index: usize,
    is_header: bool,
    hf_type: HeaderFooterType,
    linked: bool,
) -> Result<EditResult, EditError> {
    if section_index == 0 {
        return Err(EditError::InvalidRange);
    }

    let old_links = if is_header {
        doc.sections[section_index].header_links.clone()
    } else {
        doc.sections[section_index].footer_links.clone()
    };

    if linked {
        let section = doc
            .sections
            .get_mut(section_index)
            .ok_or(EditError::InvalidRange)?;
        if is_header {
            section.header_links.set_linked(hf_type, true);
            section.headers.remove(&hf_type);
        } else {
            section.footer_links.set_linked(hf_type, true);
            section.footers.remove(&hf_type);
        }
    } else {
        unlink_header_footer_if_needed(doc, section_index, is_header, hf_type)?;
    }

    Ok(EditResult {
        old_header_footer_links: Some((section_index, is_header, old_links)),
        ..Default::default()
    })
}

fn unlink_header_footer_if_needed(
    doc: &mut Document,
    section_index: usize,
    is_header: bool,
    hf_type: HeaderFooterType,
) -> Result<(), EditError> {
    if section_index == 0 {
        return Ok(());
    }

    let linked = doc.header_footer_linked(section_index, is_header, hf_type);
    if !linked {
        return Ok(());
    }

    let source = if is_header {
        doc.resolved_header(section_index, hf_type)
            .cloned()
            .unwrap_or_default()
    } else {
        doc.resolved_footer(section_index, hf_type)
            .cloned()
            .unwrap_or_default()
    };

    let section = doc
        .sections
        .get_mut(section_index)
        .ok_or(EditError::InvalidRange)?;
    if is_header {
        section.header_links.set_linked(hf_type, false);
        section.headers.entry(hf_type).or_insert(source);
    } else {
        section.footer_links.set_linked(hf_type, false);
        section.footers.entry(hf_type).or_insert(source);
    }
    Ok(())
}

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
    data: Option<tw_model::ImageData>,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let image = if let Some(data) = data {
        ImageBlock {
            id: NodeId::new(),
            data,
            display_width: width,
            display_height: height,
            wrap: tw_model::TextWrap::Inline,
            anchor: None,
            transform: tw_model::ImageTransform::default(),
            caption_paragraph_id: None,
            alt_text: None,
        }
    } else {
        ImageBlock::placeholder(width, height)
    };
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

pub fn insert_shape(
    doc: &mut Document,
    after_block_id: NodeId,
    shape_type: tw_model::ShapeKind,
    width: f32,
    height: f32,
    style: tw_model::ShapeStyle,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let shape = tw_model::ShapeBlock::new(
        shape_type,
        width.max(1.0),
        height.max(1.0),
        style,
    );
    let new_id = shape.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::ShapeBlock(shape));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn insert_text_box(
    doc: &mut Document,
    after_block_id: NodeId,
    width: f32,
    height: f32,
    style: tw_model::ShapeStyle,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let shape = tw_model::ShapeBlock::text_box(width.max(1.0), height.max(1.0), style);
    let new_id = shape.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::ShapeBlock(shape));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn insert_word_art(
    doc: &mut Document,
    after_block_id: NodeId,
    text: String,
    width: f32,
    height: f32,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let shape = tw_model::ShapeBlock::word_art(text, width.max(1.0), height.max(1.0));
    let new_id = shape.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::ShapeBlock(shape));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn insert_diagram(
    doc: &mut Document,
    after_block_id: NodeId,
    width: f32,
    height: f32,
    kind: tw_model::DiagramKind,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let shape =
        tw_model::ShapeBlock::diagram_with_kind(width.max(1.0), height.max(1.0), kind);
    let new_id = shape.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::ShapeBlock(shape));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn insert_chart(
    doc: &mut Document,
    after_block_id: NodeId,
    width: f32,
    height: f32,
    kind: tw_model::ChartKind,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let shape =
        tw_model::ShapeBlock::chart_with_kind(width.max(1.0), height.max(1.0), kind);
    let new_id = shape.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::ShapeBlock(shape));

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn set_chart_data(
    doc: &mut Document,
    shape_id: NodeId,
    chart_data: Option<tw_model::ChartData>,
) -> Result<EditResult, EditError> {
    if let Some(data) = &chart_data {
        data.validate()
            .map_err(|message| EditError::InvalidImageData(message.into()))?;
    }

    let (si, bi) = doc
        .find_block_location(shape_id)
        .ok_or(EditError::BlockNotFound(shape_id))?;
    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(shape_id))?;
    let Some(shape) = block.shape_mut() else {
        return Err(EditError::BlockNotFound(shape_id));
    };
    if shape.shape.shape_type != tw_model::ShapeKind::Chart {
        return Err(EditError::BlockNotFound(shape_id));
    }

    let old_chart_data = shape.chart_data.clone();
    shape.chart_data = chart_data;

    Ok(EditResult {
        affected_nodes: vec![shape_id],
        old_chart_data: Some(old_chart_data),
        ..Default::default()
    })
}

pub fn insert_office_math_display(
    doc: &mut Document,
    after_block_id: NodeId,
    xml: String,
) -> Result<EditResult, EditError> {
    if xml.trim().is_empty() {
        return Err(EditError::InvalidRange);
    }

    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let mut para = tw_model::Paragraph::new();
    let run_id = para.runs[0].id;
    para.runs = vec![tw_model::Run {
        id: run_id,
        format: tw_model::CharFormat::default(),
        content: tw_model::RunContent::OfficeMath { xml },
        revision: None,
    }];
    let para_id = para.id;

    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::Paragraph(para));

    Ok(EditResult {
        affected_nodes: vec![para_id, run_id],
        created_node_id: Some(para_id),
        seed_run_id: Some(run_id),
        ..Default::default()
    })
}

pub fn set_image_size(
    doc: &mut Document,
    image_id: NodeId,
    width: f32,
    height: f32,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?;
    let Some(image) = block.image_mut() else {
        return Err(EditError::BlockNotFound(image_id));
    };

    let old_size = (image.display_width, image.display_height);
    image.display_width = width.max(1.0);
    image.display_height = height.max(1.0);

    Ok(EditResult {
        affected_nodes: vec![image_id],
        old_image_size: Some(old_size),
        ..Default::default()
    })
}

/// Set or clear accessibility alternative text on an image block (F21.S3).
pub fn set_image_alt_text(
    doc: &mut Document,
    image_id: NodeId,
    alt_text: Option<String>,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?;
    let Some(image) = block.image_mut() else {
        return Err(EditError::BlockNotFound(image_id));
    };

    let old_alt = image.alt_text.clone();
    image.alt_text = alt_text
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(EditResult {
        affected_nodes: vec![image_id],
        old_image_alt_text: Some(old_alt),
        ..Default::default()
    })
}

pub fn replace_image_bytes(
    doc: &mut Document,
    image_id: NodeId,
    data: tw_model::ImageData,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?;
    let Some(image) = block.image_mut() else {
        return Err(EditError::BlockNotFound(image_id));
    };

    let old_data = image.data.clone();
    image.data = data;

    Ok(EditResult {
        affected_nodes: vec![image_id],
        old_image_data: Some(old_data),
        ..Default::default()
    })
}

pub fn set_image_wrap(
    doc: &mut Document,
    image_id: NodeId,
    wrap: tw_model::TextWrap,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?;
    let Some(image) = block.image_mut() else {
        return Err(EditError::BlockNotFound(image_id));
    };

    let old_wrap = image.wrap;
    let old_anchor = image.anchor;
    image.wrap = wrap;

    match wrap {
        tw_model::TextWrap::Inline => {
            image.anchor = None;
        }
        _ => {
            if image.anchor.is_none() {
                image.anchor = Some(tw_model::ImageAnchor {
                    x: 0.0,
                    y: 0.0,
                    origin_x: tw_model::AnchorOrigin::Column,
                    origin_y: tw_model::AnchorOrigin::Column,
                });
            }
        }
    }

    Ok(EditResult {
        affected_nodes: vec![image_id],
        old_image_wrap: Some(old_wrap),
        old_image_anchor: Some(old_anchor),
        ..Default::default()
    })
}

pub fn set_image_anchor(
    doc: &mut Document,
    image_id: NodeId,
    anchor: tw_model::ImageAnchor,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?;
    let Some(image) = block.image_mut() else {
        return Err(EditError::BlockNotFound(image_id));
    };

    let old_wrap = image.wrap;
    let old_anchor = image.anchor;
    image.anchor = Some(anchor);
    if image.wrap == tw_model::TextWrap::Inline {
        image.wrap = tw_model::TextWrap::Square;
    }

    Ok(EditResult {
        affected_nodes: vec![image_id],
        old_image_wrap: Some(old_wrap),
        old_image_anchor: Some(old_anchor),
        ..Default::default()
    })
}

pub fn restore_image_layout(
    doc: &mut Document,
    image_id: NodeId,
    wrap: tw_model::TextWrap,
    anchor: Option<tw_model::ImageAnchor>,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?;
    let Some(image) = block.image_mut() else {
        return Err(EditError::BlockNotFound(image_id));
    };

    let old_wrap = image.wrap;
    let old_anchor = image.anchor;
    image.wrap = wrap;
    image.anchor = anchor;

    Ok(EditResult {
        affected_nodes: vec![image_id],
        old_image_wrap: Some(old_wrap),
        old_image_anchor: Some(old_anchor),
        ..Default::default()
    })
}

pub fn set_image_transform(
    doc: &mut Document,
    image_id: NodeId,
    transform: tw_model::ImageTransform,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?;
    let Some(image) = block.image_mut() else {
        return Err(EditError::BlockNotFound(image_id));
    };

    let old_transform = image.transform;
    image.transform = transform.normalized();

    Ok(EditResult {
        affected_nodes: vec![image_id],
        old_image_transform: Some(old_transform),
        ..Default::default()
    })
}

pub fn insert_image_caption(
    doc: &mut Document,
    image_id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    if doc.sections[si].blocks[bi]
        .image()
        .and_then(|image| image.caption_paragraph_id)
        .is_some()
    {
        return Err(EditError::BlockNotFound(image_id));
    }

    let caption_style = doc.styles.find_style_by_name("Caption").map(|s| s.id);
    let mut para = Paragraph::new();
    para.style_id = caption_style;
    para.runs.push(Run::new_text("Figure 1"));
    let caption_id = para.id;
    doc.sections[si]
        .blocks
        .insert(bi + 1, Block::Paragraph(para));

    let image = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?
        .image_mut()
        .ok_or(EditError::BlockNotFound(image_id))?;
    let old_caption = image.caption_paragraph_id;
    image.caption_paragraph_id = Some(caption_id);

    Ok(EditResult {
        affected_nodes: vec![image_id, caption_id],
        created_node_id: Some(caption_id),
        old_image_caption_id: Some(old_caption),
        ..Default::default()
    })
}

pub fn remove_image_caption(
    doc: &mut Document,
    image_id: NodeId,
    caption_paragraph_id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    {
        let image = doc.sections[si].blocks[bi]
            .image()
            .ok_or(EditError::BlockNotFound(image_id))?;
        if image.caption_paragraph_id != Some(caption_paragraph_id) {
            return Err(EditError::BlockNotFound(caption_paragraph_id));
        }
    }

    let (cap_si, cap_bi) = doc
        .find_block_location(caption_paragraph_id)
        .ok_or(EditError::BlockNotFound(caption_paragraph_id))?;
    doc.sections[cap_si].blocks.remove(cap_bi);

    let image = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?
        .image_mut()
        .ok_or(EditError::BlockNotFound(image_id))?;
    image.caption_paragraph_id = None;

    Ok(EditResult {
        affected_nodes: vec![image_id],
        ..Default::default()
    })
}

pub fn compress_image(
    doc: &mut Document,
    image_id: NodeId,
    quality: u8,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(image_id)
        .ok_or(EditError::BlockNotFound(image_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::BlockNotFound(image_id))?;
    let Some(image) = block.image_mut() else {
        return Err(EditError::BlockNotFound(image_id));
    };

    let old_data = image.data.clone();
    let compressed = crate::image_ops::compress_image_data(&old_data, quality)
        .map_err(|e| EditError::InvalidImageData(e))?;
    image.data = compressed;

    Ok(EditResult {
        affected_nodes: vec![image_id],
        old_image_data: Some(old_data),
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

pub fn insert_section_break(
    doc: &mut Document,
    after_block_id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let split_at = bi + 1;
    let trailing = doc.sections[si].blocks.split_off(split_at);
    let mut new_section = Section::new();
    new_section.format = doc.sections[si].format.clone();
    new_section.header_links = HeaderFooterLinks::linked_to_previous();
    new_section.footer_links = HeaderFooterLinks::linked_to_previous();
    new_section.blocks = if trailing.is_empty() {
        vec![Block::Paragraph(Paragraph::new())]
    } else {
        trailing
    };
    let new_id = new_section.id;
    let inserted_index = si + 1;
    doc.sections.insert(inserted_index, new_section.clone());

    Ok(EditResult {
        affected_nodes: vec![new_id],
        split_section: Some((inserted_index, new_section)),
        ..Default::default()
    })
}

pub fn merge_section(
    doc: &mut Document,
    section_index: usize,
) -> Result<EditResult, EditError> {
    if section_index == 0 || section_index >= doc.sections.len() {
        return Err(EditError::InvalidRange);
    }

    let after_id = match doc.sections[section_index - 1].blocks.last() {
        Some(Block::Paragraph(p)) => p.id,
        Some(Block::Table(t)) => t.id,
        Some(Block::ImageBlock(i)) => i.id,
        Some(Block::ShapeBlock(s)) => s.id,
        _ => return Err(EditError::InvalidRange),
    };

    let removed = doc.sections.remove(section_index);
    let removed_id = removed.id;
    let removed_for_undo = removed.clone();
    doc.sections[section_index - 1]
        .blocks
        .extend(removed.blocks);

    Ok(EditResult {
        affected_nodes: vec![removed_id],
        previous_block_id: Some(after_id),
        split_section: Some((section_index, removed_for_undo)),
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

    if sr > er || sc > ec {
        return Err(EditError::InvalidRange);
    }
    if er >= table.rows.len() {
        return Err(EditError::InvalidRange);
    }

    if let Some(row) = table.rows.get_mut(sr) {
        if sc >= row.cells.len() || ec >= row.cells.len() {
            return Err(EditError::InvalidRange);
        }
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

    Err(EditError::TableNotFound(table_id))
}

pub fn split_table_cell(
    doc: &mut Document,
    table_id: NodeId,
    row: u32,
    col: u32,
) -> Result<EditResult, EditError> {
    let ri = row as usize;
    let ci = col as usize;
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let Some(cell) = table.rows.get_mut(ri).and_then(|r| r.cells.get_mut(ci)) else {
        return Err(EditError::InvalidRange);
    };
    if cell.format.colspan <= 1 && cell.format.rowspan <= 1 {
        return Err(EditError::InvalidRange);
    }

    let old_colspan = cell.format.colspan;
    let old_rowspan = cell.format.rowspan;
    cell.format.colspan = 1;
    cell.format.rowspan = 1;
    Ok(EditResult {
        affected_nodes: vec![table_id],
        old_cell_span: Some((old_colspan, old_rowspan)),
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

pub fn set_table_border(
    doc: &mut Document,
    table_id: NodeId,
    border: Option<tw_model::BorderSpec>,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let old = table.format.border;
    table.format.border = border;
    Ok(EditResult {
        affected_nodes: vec![table_id],
        old_table_border: Some(old),
        ..Default::default()
    })
}

pub fn set_table_cell_shading(
    doc: &mut Document,
    table_id: NodeId,
    row: u32,
    col: u32,
    background: Option<tw_model::Color>,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let ri = row as usize;
    let ci = col as usize;
    let cell = table
        .rows
        .get_mut(ri)
        .and_then(|r| r.cells.get_mut(ci))
        .ok_or(EditError::TableNotFound(table_id))?;

    let old = cell.format.background;
    cell.format.background = background;
    Ok(EditResult {
        affected_nodes: vec![table_id],
        old_cell_background: Some(old),
        ..Default::default()
    })
}

pub fn autofit_table_to_width(
    doc: &mut Document,
    table_id: NodeId,
    target_width: f32,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let old_widths = table.format.column_widths.clone();
    if old_widths.is_empty() {
        return Ok(EditResult {
            affected_nodes: vec![table_id],
            ..Default::default()
        });
    }

    let current_sum: f32 = old_widths.iter().sum();
    table.format.column_widths = if current_sum <= 0.0 {
        let w = target_width / old_widths.len() as f32;
        vec![w; old_widths.len()]
    } else {
        let scale = target_width / current_sum;
        old_widths.iter().map(|w| w * scale).collect()
    };
    table.format.width = Some(target_width);

    Ok(EditResult {
        affected_nodes: vec![table_id],
        old_table_column_widths: Some(old_widths),
        ..Default::default()
    })
}

pub fn restore_table_column_widths(
    doc: &mut Document,
    table_id: NodeId,
    column_widths: Vec<f32>,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let old_widths = table.format.column_widths.clone();
    table.format.column_widths = column_widths.clone();
    table.format.width = Some(column_widths.iter().sum());

    Ok(EditResult {
        affected_nodes: vec![table_id],
        old_table_column_widths: Some(old_widths),
        ..Default::default()
    })
}

fn compare_sort_keys(a: &str, b: &str) -> std::cmp::Ordering {
    match (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
        (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
        _ => a
            .trim()
            .to_ascii_lowercase()
            .cmp(&b.trim().to_ascii_lowercase()),
    }
}

pub fn sort_table_rows(
    doc: &mut Document,
    table_id: NodeId,
    column: u32,
    ascending: bool,
    skip_header: bool,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let old_rows = table.rows.clone();
    let grid_col = column as usize;
    let start = if skip_header && table.rows.len() > 1 {
        1
    } else {
        0
    };
    let mut indices: Vec<usize> = (start..table.rows.len()).collect();
    indices.sort_by(|&a, &b| {
        let ka = cell_text_at_grid(table, a, grid_col).unwrap_or_default();
        let kb = cell_text_at_grid(table, b, grid_col).unwrap_or_default();
        let ord = compare_sort_keys(&ka, &kb);
        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });

    let mut new_rows = table.rows[..start].to_vec();
    for i in indices {
        new_rows.push(table.rows[i].clone());
    }
    table.rows = new_rows;

    Ok(EditResult {
        affected_nodes: vec![table_id],
        old_table_rows: Some(old_rows),
        ..Default::default()
    })
}

pub fn restore_table_row_order(
    doc: &mut Document,
    table_id: NodeId,
    rows: Vec<TableRow>,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let old_rows = table.rows.clone();
    table.rows = rows;

    Ok(EditResult {
        affected_nodes: vec![table_id],
        old_table_rows: Some(old_rows),
        ..Default::default()
    })
}

pub fn insert_nested_table(
    doc: &mut Document,
    table_id: NodeId,
    row: u32,
    col: u32,
    rows: u32,
    cols: u32,
) -> Result<EditResult, EditError> {
    let ri = row as usize;
    let ci = col as usize;
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };
    let cell = table
        .rows
        .get_mut(ri)
        .and_then(|r| r.cells.get_mut(ci))
        .ok_or(EditError::TableNotFound(table_id))?;

    let nested = Table::new(rows, cols);
    let new_id = nested.id;
    let block_index = cell.blocks.len();
    cell.blocks.push(Block::Table(nested));

    Ok(EditResult {
        affected_nodes: vec![table_id, new_id],
        created_node_id: Some(new_id),
        inserted_cell_block: Some((table_id, row, col, block_index)),
        ..Default::default()
    })
}

pub fn remove_table_cell_block(
    doc: &mut Document,
    table_id: NodeId,
    row: u32,
    col: u32,
    block_index: usize,
) -> Result<EditResult, EditError> {
    let ri = row as usize;
    let ci = col as usize;
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };
    let cell = table
        .rows
        .get_mut(ri)
        .and_then(|r| r.cells.get_mut(ci))
        .ok_or(EditError::TableNotFound(table_id))?;
    let removed = cell
        .blocks
        .get(block_index)
        .cloned()
        .ok_or(EditError::TableNotFound(table_id))?;
    cell.blocks.remove(block_index);

    Ok(EditResult {
        affected_nodes: vec![table_id],
        removed_cell_block: Some((table_id, row, col, block_index, removed)),
        ..Default::default()
    })
}

pub fn insert_table_cell_block(
    doc: &mut Document,
    table_id: NodeId,
    row: u32,
    col: u32,
    block_index: usize,
    block: Block,
) -> Result<EditResult, EditError> {
    let ri = row as usize;
    let ci = col as usize;
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let table_block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = table_block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };
    let cell = table
        .rows
        .get_mut(ri)
        .and_then(|r| r.cells.get_mut(ci))
        .ok_or(EditError::TableNotFound(table_id))?;
    let index = block_index.min(cell.blocks.len());
    cell.blocks.insert(index, block);

    Ok(EditResult {
        affected_nodes: vec![table_id],
        inserted_cell_block: Some((table_id, row, col, index)),
        ..Default::default()
    })
}

pub fn delete_table_row(
    doc: &mut Document,
    table_id: NodeId,
    row: u32,
) -> Result<EditResult, EditError> {
    let ri = row as usize;
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    if table.rows.len() <= 1 || ri >= table.rows.len() {
        return Err(EditError::InvalidRange);
    }

    for r in 0..ri {
        for cell in &mut table.rows[r].cells {
            let span = cell.format.rowspan.max(1) as usize;
            if r + span > ri && span > 1 {
                cell.format.rowspan -= 1;
            }
        }
    }

    let removed = table.rows.remove(ri);
    Ok(EditResult {
        affected_nodes: vec![table_id],
        deleted_table_row: Some((table_id, ri, removed)),
        ..Default::default()
    })
}

pub fn restore_table_row(
    doc: &mut Document,
    table_id: NodeId,
    row: u32,
    row_data: tw_model::TableRow,
) -> Result<EditResult, EditError> {
    let ri = row as usize;
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let insert_at = ri.min(table.rows.len());
    table.rows.insert(insert_at, row_data);
    Ok(EditResult {
        affected_nodes: vec![table_id],
        ..Default::default()
    })
}

pub fn delete_table_column(
    doc: &mut Document,
    table_id: NodeId,
    column: u32,
) -> Result<EditResult, EditError> {
    let grid_col = column as usize;
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    let grid_cols = table.grid_column_count();
    if grid_cols <= 1 || grid_col >= grid_cols {
        return Err(EditError::InvalidRange);
    }

    let width = table
        .format
        .column_widths
        .get(grid_col)
        .copied()
        .unwrap_or(100.0);
    let mut removed_cells = Vec::with_capacity(table.rows.len());

    for row in &mut table.rows {
        let cell_idx = row
            .cell_index_at_grid_column(grid_col)
            .ok_or(EditError::InvalidRange)?;
        let span = row.cells[cell_idx].format.colspan.max(1) as usize;
        if span > 1 {
            let start = row.grid_column_for_cell_index(cell_idx);
            if grid_col == start || grid_col == start + span - 1 {
                row.cells.get_mut(cell_idx).unwrap().format.colspan -= 1;
                removed_cells.push(tw_model::TableCell::new());
            } else {
                return Err(EditError::InvalidRange);
            }
        } else {
            removed_cells.push(row.cells.remove(cell_idx));
        }
    }

    if grid_col < table.format.column_widths.len() {
        table.format.column_widths.remove(grid_col);
    }
    table.format.width = Some(table.format.column_widths.iter().sum());

    Ok(EditResult {
        affected_nodes: vec![table_id],
        deleted_table_column: Some((table_id, grid_col, removed_cells, width)),
        ..Default::default()
    })
}

pub fn restore_table_column(
    doc: &mut Document,
    table_id: NodeId,
    column: u32,
    cells: Vec<tw_model::TableCell>,
    column_width: f32,
) -> Result<EditResult, EditError> {
    let grid_col = column as usize;
    let (si, bi) = doc
        .find_block_location(table_id)
        .ok_or(EditError::TableNotFound(table_id))?;

    let block = doc
        .block_at_mut(si, bi)
        .ok_or(EditError::TableNotFound(table_id))?;
    let Some(table) = block.table_mut() else {
        return Err(EditError::TableNotFound(table_id));
    };

    if cells.len() != table.rows.len() {
        return Err(EditError::InvalidRange);
    }

    for (row, cell) in table.rows.iter_mut().zip(cells) {
        let insert_at = row.cell_index_at_grid_column(grid_col).unwrap_or(row.cells.len());
        if insert_at < row.cells.len() && row.cells[insert_at].format.colspan > 1 {
            row.cells.get_mut(insert_at).unwrap().format.colspan += 1;
        } else {
            row.cells.insert(insert_at, cell);
        }
    }

    if grid_col <= table.format.column_widths.len() {
        table.format.column_widths.insert(grid_col, column_width);
    } else {
        table.format.column_widths.push(column_width);
    }
    table.format.width = Some(table.format.column_widths.iter().sum());

    Ok(EditResult {
        affected_nodes: vec![table_id],
        ..Default::default()
    })
}

pub fn delete_block(
    doc: &mut Document,
    id: NodeId,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(id)
        .ok_or(EditError::BlockNotFound(id))?;

    if doc.sections[si].blocks.len() <= 1 {
        return Err(EditError::InvalidRange);
    }

    fn block_id(block: &Block) -> Option<NodeId> {
        match block {
            Block::Paragraph(p) => Some(p.id),
            Block::Table(t) => Some(t.id),
            Block::ImageBlock(i) => Some(i.id),
            Block::ShapeBlock(s) => Some(s.id),
            _ => None,
        }
    }

    // Prefer undo via InsertBlock(after prev). When deleting the first block,
    // store the next block id and restore with InsertBlockBefore.
    let previous_block_id = if bi > 0 {
        block_id(&doc.sections[si].blocks[bi - 1])
    } else {
        None
    };
    let insert_before_id = if bi == 0 {
        block_id(&doc.sections[si].blocks[bi + 1])
    } else {
        None
    };

    let block = doc.sections[si].blocks.remove(bi);

    Ok(EditResult {
        affected_nodes: vec![id],
        previous_block_id,
        insert_before_block_id: insert_before_id,
        deleted_block: Some(block),
        ..Default::default()
    })
}

pub fn insert_block(
    doc: &mut Document,
    after_block_id: NodeId,
    block: Block,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(after_block_id)
        .ok_or(EditError::BlockNotFound(after_block_id))?;

    let new_id = match &block {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => return Err(EditError::InvalidRange),
    };

    doc.sections[si].blocks.insert(bi + 1, block);

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}

pub fn insert_block_before(
    doc: &mut Document,
    before_block_id: NodeId,
    block: Block,
) -> Result<EditResult, EditError> {
    let (si, bi) = doc
        .find_block_location(before_block_id)
        .ok_or(EditError::BlockNotFound(before_block_id))?;

    let new_id = match &block {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => return Err(EditError::InvalidRange),
    };

    doc.sections[si].blocks.insert(bi, block);

    Ok(EditResult {
        affected_nodes: vec![new_id],
        created_node_id: Some(new_id),
        ..Default::default()
    })
}
