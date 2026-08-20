use std::collections::HashMap;

use tw_model::{
    Block, BorderSpec, Color, Document, ImageBlock, ImageData, Paragraph, Table, TableCell,
    TableRow, VerticalAlign, NodeId, TableFormat,
};

use crate::paragraph::parse_paragraph;
use crate::xml_util::{
    read_attr_value, read_int_attr, read_own_attr, split_elements, twips_to_points,
};
use crate::DocxPackage;

const FALLBACK_COLUMN_WIDTH: f32 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VMergeKind {
    None,
    Restart,
    Continue,
}

#[derive(Clone)]
struct RawCell {
    cell: TableCell,
    vmerge: VMergeKind,
}

pub fn parse_table(tbl_xml: &str, doc: &Document) -> Table {
    let raw_rows: Vec<(String, Vec<RawCell>)> = split_elements(tbl_xml, "w:tr")
        .into_iter()
        .filter_map(|chunk| {
            parse_table_row(chunk, doc).map(|cells| (chunk.to_string(), cells))
        })
        .collect();

    let mut rows = finalize_vertical_merges(raw_rows.iter().map(|(_, cells)| cells.clone()).collect());
    for (row, (chunk, _)) in rows.iter_mut().zip(raw_rows.iter()) {
        row.height = read_int_attr(chunk, "w:trHeight", "w:val")
            .filter(|v| *v > 0)
            .map(|v| twips_to_points(v as f32));
    }

    let cols = rows
        .iter()
        .map(|r| {
            r.cells
                .iter()
                .map(|c| c.format.colspan.max(1) as usize)
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0)
        .max(1);

    let mut column_widths = parse_grid_columns(tbl_xml);
    if column_widths.is_empty() {
        column_widths = vec![FALLBACK_COLUMN_WIDTH; cols];
    } else if column_widths.len() < cols {
        column_widths.resize(cols, FALLBACK_COLUMN_WIDTH);
    }

    let border = parse_table_border(tbl_xml);

    let mut table = Table {
        id: NodeId::new(),
        format: TableFormat {
            width: Some(column_widths.iter().sum()),
            column_widths,
            border,
        },
        rows,
        style_id: None,
            anchor: None,
    };
    apply_table_style(doc, &mut table, tbl_xml);
    table
}

fn apply_table_style(doc: &Document, table: &mut Table, tbl_xml: &str) {
    let tbl_pr = split_elements(tbl_xml, "w:tblPr")
        .into_iter()
        .next()
        .unwrap_or("");
    let Some(style_str) = read_attr_value(tbl_pr, "w:tblStyle", "w:val") else {
        return;
    };
    let Some(&style_id) = doc.styles.ooxml_table_style_ids.get(&style_str) else {
        return;
    };
    table.style_id = Some(style_id);
    if table.format.border.is_none() {
        if let Some(style) = doc.styles.table_styles.get(&style_id) {
            table.format.border = style.border;
        }
    }
}

fn parse_table_border(tbl_xml: &str) -> Option<BorderSpec> {
    let tcpr = split_elements(tbl_xml, "w:tblPr")
        .into_iter()
        .next()
        .unwrap_or("");
    parse_border_from_edges(tcpr, "w:tblBorders")
}

/// Column widths from `w:tblGrid`, converted from twips to points.
fn parse_grid_columns(tbl_xml: &str) -> Vec<f32> {
    let Some(start) = tbl_xml.find("<w:tblGrid") else {
        return Vec::new();
    };
    let grid = &tbl_xml[start..];
    let end = grid.find("</w:tblGrid>").unwrap_or(grid.len());
    let grid = &grid[..end];

    split_elements(grid, "w:gridCol")
        .into_iter()
        .filter_map(|tag_body| read_tag_attr(tag_body, "w:w"))
        .map(twips_to_points)
        .filter(|w| *w > 0.0)
        .collect()
}

/// Reads a numeric attribute from an already-isolated tag body.
fn read_tag_attr(tag_body: &str, attr: &str) -> Option<f32> {
    let pattern = format!("{attr}=\"");
    let start = tag_body.find(&pattern)? + pattern.len();
    let rest = &tag_body[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

fn parse_table_row(row_xml: &str, doc: &Document) -> Option<Vec<RawCell>> {
    let cells: Vec<RawCell> = split_elements(row_xml, "w:tc")
        .into_iter()
        .map(|chunk| parse_table_cell(chunk, doc))
        .collect();
    if cells.is_empty() {
        return None;
    }

    Some(cells)
}

fn finalize_vertical_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<TableRow> {
    let row_count = raw_rows.len();
    let col_count = raw_rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|raw| raw.cell.format.colspan.max(1) as usize)
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0);

    let mut grid: Vec<Vec<Option<VMergeKind>>> = vec![vec![None; col_count]; row_count];
    for (ri, row) in raw_rows.iter().enumerate() {
        let mut col = 0usize;
        for raw in row {
            let span = raw.cell.format.colspan.max(1) as usize;
            for c in col..col + span {
                grid[ri][c] = Some(raw.vmerge);
            }
            col += span;
        }
    }

    let mut rows = Vec::with_capacity(row_count);
    for (ri, raw_row) in raw_rows.into_iter().enumerate() {
        let mut cells = Vec::new();
        let mut col = 0usize;
        for raw in raw_row {
            let span = raw.cell.format.colspan.max(1) as usize;
            match raw.vmerge {
                VMergeKind::Continue => {
                    col += span;
                    continue;
                }
                VMergeKind::Restart => {
                    let mut cell = raw.cell;
                    let mut rowspan = 1u32;
                    for r in (ri + 1)..row_count {
                        if grid[r][col] == Some(VMergeKind::Continue) {
                            rowspan += 1;
                        } else {
                            break;
                        }
                    }
                    cell.format.rowspan = rowspan;
                    cells.push(cell);
                }
                VMergeKind::None => cells.push(raw.cell),
            }
            col += span;
        }
        rows.push(TableRow::with_cells(cells));
    }

    // Restore row heights from original raw rows - we lost trPr; re-parse not needed if height was on row
    rows
}

fn parse_table_cell(cell_xml: &str, doc: &Document) -> RawCell {
    let mut cell = TableCell::new();
    let tcpr = split_elements(cell_xml, "w:tcPr")
        .into_iter()
        .next()
        .unwrap_or("");

    cell.format.colspan = read_int_attr(tcpr, "w:gridSpan", "w:val")
        .unwrap_or(1)
        .max(1) as u32;
    cell.format = parse_cell_format(tcpr, cell.format);

    let mut blocks = Vec::new();
    for chunk in split_elements(cell_xml, "w:p") {
        if let Some(para) = parse_paragraph(doc, chunk) {
            blocks.push(Block::Paragraph(para));
        }
    }
    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }
    cell.blocks = blocks;

    RawCell {
        cell,
        vmerge: vmerge_kind(tcpr),
    }
}

fn parse_cell_format(tcpr: &str, mut format: tw_model::CellFormat) -> tw_model::CellFormat {
    if let Some(border) = parse_border_from_edges(tcpr, "w:tcBorders") {
        format.border = Some(border);
    }
    if let Some(fill) = read_attr_value(tcpr, "w:shd", "w:fill") {
        format.background = parse_fill_color(&fill);
    }
    format.vertical_align = match read_attr_value(tcpr, "w:vAlign", "w:val").as_deref() {
        Some("center") => VerticalAlign::Middle,
        Some("bottom") => VerticalAlign::Bottom,
        _ => VerticalAlign::Top,
    };
    format
}

fn parse_border_from_edges(xml: &str, container: &str) -> Option<BorderSpec> {
    if !xml.contains(container) {
        return None;
    }
    let width = ["w:top", "w:left", "w:bottom", "w:right"]
        .iter()
        .filter_map(|edge| read_numeric_border_sz(xml, edge))
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))?;
    let color = ["w:top", "w:left", "w:bottom", "w:right"]
        .iter()
        .find_map(|edge| read_border_color(xml, edge))
        .unwrap_or(Color::BLACK);
    Some(BorderSpec { width, color })
}

fn read_numeric_border_sz(xml: &str, edge: &str) -> Option<f32> {
    read_int_attr(xml, edge, "w:sz")
        .filter(|v| *v > 0)
        .map(|v| v as f32 / 8.0)
}

fn read_border_color(xml: &str, edge: &str) -> Option<Color> {
    read_attr_value(xml, edge, "w:color").and_then(|v| parse_fill_color(&v))
}

fn parse_fill_color(value: &str) -> Option<Color> {
    if value == "auto" || value.len() != 6 {
        return None;
    }
    Some(Color {
        r: u8::from_str_radix(&value[0..2], 16).ok()?,
        g: u8::from_str_radix(&value[2..4], 16).ok()?,
        b: u8::from_str_radix(&value[4..6], 16).ok()?,
        a: 255,
    })
}

fn vmerge_kind(tcpr: &str) -> VMergeKind {
    if !tcpr.contains("<w:vMerge") {
        return VMergeKind::None;
    }
    match read_attr_value(tcpr, "w:vMerge", "w:val").as_deref() {
        Some("restart") => VMergeKind::Restart,
        Some("continue") | None => VMergeKind::Continue,
        _ => VMergeKind::Continue,
    }
}

/// English Metric Units per point (914400 EMU per inch, 72 points per inch).
const EMU_PER_POINT: f32 = 12700.0;

pub fn parse_image_block(para_xml: &str, media: &dyn MediaResolver) -> Option<ImageBlock> {
    if !para_xml.contains("<w:drawing") && !para_xml.contains("<w:pict") {
        return None;
    }
    // A drawing container is not necessarily a picture: Word also uses it for
    // shapes, text boxes, and VML rules like the `<v:rect>` horizontal
    // separators that HTML-to-DOCX converters emit. Only something pointing at
    // an image part is one.
    let rel_id = picture_relationship_id(para_xml)?;

    let width = read_attr_value(para_xml, "wp:extent", "cx")
        .and_then(|v| v.parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(200.0);
    let height = read_attr_value(para_xml, "wp:extent", "cy")
        .and_then(|v| v.parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(150.0);

    let mut block = ImageBlock::placeholder(width, height);
    if let Some(asset) = media.resolve(&rel_id) {
        block.data = asset;
    }
    block.anchor = parse_anchor(para_xml);
    if block.anchor.is_some() {
        block.wrap = parse_text_wrap(para_xml);
        block.wrap_polygon = parse_wrap_polygon(para_xml);
    }
    // Word stores alt text on wp:docPr/@descr (preferred) or pic:cNvPr/@descr.
    block.alt_text = read_attr_value(para_xml, "wp:docPr", "descr")
        .or_else(|| read_attr_value(para_xml, "pic:cNvPr", "descr"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some(block)
}

/// Inline image inside a run (`w:r` / `w:drawing`).
pub fn parse_inline_image(run_xml: &str, media: &dyn MediaResolver) -> Option<tw_model::InlineImageRef> {
    let rel_id = picture_relationship_id(run_xml)?;
    let width = read_attr_value(run_xml, "wp:extent", "cx")
        .and_then(|v| v.parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(48.0);
    let height = read_attr_value(run_xml, "wp:extent", "cy")
        .and_then(|v| v.parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(48.0);
    let image = media.resolve(&rel_id).unwrap_or_else(|| tw_model::ImageData {
        asset_id: rel_id.clone(),
        mime_type: "image/png".into(),
        width_px: width as u32,
        height_px: height as u32,
        bytes: Vec::new(),
    });
    Some(tw_model::InlineImageRef {
        image,
        display_width: width,
        display_height: height,
    })
}

/// Block-level shape when drawing is not an image blip.
pub fn parse_shape_block(
    para_xml: &str,
    media: &dyn MediaResolver,
    package: &DocxPackage,
) -> Option<tw_model::ShapeBlock> {
    if !para_xml.contains("<w:drawing") && !para_xml.contains("<w:pict") {
        return None;
    }
    if picture_relationship_id(para_xml).is_some() {
        return None;
    }
    let (vml_w, vml_h) = parse_vml_or_drawing_size(para_xml);
    let mut width = read_attr_value(para_xml, "wp:extent", "cx")
        .and_then(|v| v.parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(vml_w);
    let mut height = read_attr_value(para_xml, "wp:extent", "cy")
        .and_then(|v| v.parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(if vml_h > 0.0 { vml_h } else { 50.0 });
    if para_xml.contains("o:hr") {
        if vml_h > 0.0 {
            height = vml_h;
        } else {
            height = 1.5;
        }
        if vml_w <= 0.0 {
            width = 0.0;
        }
    } else {
        if width <= 0.0 {
            width = 100.0;
        }
        if height <= 0.0 {
            height = 50.0;
        }
    }
    let shape_type = if para_xml.contains("drawingml/2006/chart")
        || para_xml.contains("<c:chart")
        || para_xml.contains("c:chart ")
    {
        tw_model::ShapeKind::Chart
    } else if para_xml.contains("drawingml/2006/diagram")
        || para_xml.contains("<dgm:relIds")
        || para_xml.contains("dgm:relIds")
    {
        tw_model::ShapeKind::Diagram
    } else if para_xml.contains("wps:wsp") || para_xml.contains("wordprocessingShape") {
        tw_model::ShapeKind::TextBox
    } else if para_xml.contains("prst=\"ellipse\"") {
        tw_model::ShapeKind::Ellipse
    } else if para_xml.contains("<v:line") || para_xml.contains("prst=\"line\"") {
        tw_model::ShapeKind::Line
    } else if para_xml.contains("<v:rect") {
        tw_model::ShapeKind::Rectangle
    } else {
        tw_model::ShapeKind::Rectangle
    };
    let preview_image = match shape_type {
        tw_model::ShapeKind::Diagram => resolve_diagram_preview(para_xml, media, package),
        tw_model::ShapeKind::Chart => resolve_chart_preview(para_xml, media, package),
        _ => None,
    };
    let (chart_part, chart_data) = if shape_type == tw_model::ShapeKind::Chart {
        let part = crate::chart::resolve_chart_part_path(para_xml, package);
        let data = part
            .as_ref()
            .and_then(|path| package.parts.get(path))
            .and_then(|bytes| crate::chart::parse_chart_data(&String::from_utf8_lossy(bytes)));
        (part, data)
    } else {
        (None, None)
    };
    let (diagram_data_part, diagram_layout_part) = if shape_type == tw_model::ShapeKind::Diagram {
        let data = crate::diagram::resolve_diagram_data_part(para_xml, package);
        let layout = crate::diagram::resolve_diagram_layout_part(para_xml, package);
        (data, layout)
    } else {
        (None, None)
    };
    Some(tw_model::ShapeBlock {
        id: NodeId::new(),
        shape: tw_model::ShapeData {
            shape_type,
            width,
            height,
        },
        wrap: tw_model::TextWrap::Square,
        anchor: None,
        style: tw_model::ShapeStyle::placeholder(),
        paragraphs: extract_shape_paragraphs(para_xml),
        preview_image,
        chart_data,
        chart_part,
        diagram_kind: Default::default(),
        diagram_data_part,
        diagram_layout_part,
    })
}

fn extract_shape_paragraphs(para_xml: &str) -> Vec<tw_model::Paragraph> {
    // Prefer Word text-box content (`w:txbxContent`) so each paragraph stays separate.
    if let Some(start) = para_xml.find("<w:txbxContent") {
        let after = &para_xml[start..];
        if let Some(gt) = after.find('>') {
            let body = &after[gt + 1..];
            if let Some(end) = body.find("</w:txbxContent>") {
                let paras: Vec<_> = crate::xml_util::split_elements(&body[..end], "w:p")
                    .into_iter()
                    .filter_map(|p| {
                        let text = crate::xml_util::extract_plain_text(p);
                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(tw_model::Paragraph::with_text(trimmed))
                        }
                    })
                    .collect();
                if !paras.is_empty() {
                    return paras;
                }
            }
        }
    }

    let mut text = crate::xml_util::extract_plain_text(para_xml);
    if text.trim().is_empty() {
        // DrawingML text body uses `a:t` instead of `w:t`.
        text = extract_drawingml_text(para_xml);
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    vec![tw_model::Paragraph::with_text(trimmed)]
}

fn extract_drawingml_text(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<a:t") {
        rest = &rest[start..];
        let tag_end = rest.find('>').map(|i| i + 1).unwrap_or(rest.len());
        let after = &rest[tag_end..];
        if let Some(close) = after.find("</a:t>") {
            out.push_str(&crate::xml_util::decode_xml_entities(&after[..close]));
            rest = &after[close + 5..];
        } else {
            break;
        }
    }
    out
}

/// Resolve a PNG/JPEG/EMF preview linked from a SmartArt diagram (F12.S2).
fn resolve_diagram_preview(
    para_xml: &str,
    media: &dyn MediaResolver,
    package: &DocxPackage,
) -> Option<ImageData> {
    if let Some(rel_id) = picture_relationship_id(para_xml) {
        if let Some(image) = media.resolve(&rel_id).filter(|img| !img.bytes.is_empty()) {
            return Some(image);
        }
    }

    let drawing_rel = read_attr_value(para_xml, "dgm:relIds", "r:dr")?;
    let doc_rels = package
        .parts
        .get("word/_rels/document.xml.rels")
        .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
        .unwrap_or_default();
    let drawing_target = doc_rels.get(&drawing_rel)?;
    let drawing_part = normalize_part_path("word", drawing_target);
    let drawing_xml = package.parts.get(&drawing_part)?;
    let drawing_text = String::from_utf8_lossy(drawing_xml);
    let blip_rel = picture_relationship_id(&drawing_text)?;

    let rels_part = part_rels_part_name(&drawing_part);
    resolve_part_image(package, &rels_part, &blip_rel)
        .or_else(|| media.resolve(&blip_rel))
}

/// Resolve a PNG/JPEG/EMF preview linked from a chart (F13.S2).
fn resolve_chart_preview(
    para_xml: &str,
    media: &dyn MediaResolver,
    package: &DocxPackage,
) -> Option<ImageData> {
    if let Some(rel_id) = picture_relationship_id(para_xml) {
        if let Some(image) = media.resolve(&rel_id).filter(|img| !img.bytes.is_empty()) {
            return Some(image);
        }
    }

    let chart_rel = read_attr_value(para_xml, "c:chart", "r:id")?;
    let doc_rels = package
        .parts
        .get("word/_rels/document.xml.rels")
        .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
        .unwrap_or_default();
    let chart_target = doc_rels.get(&chart_rel)?;
    let chart_part = normalize_part_path("word", chart_target);
    let chart_xml = package.parts.get(&chart_part)?;
    let chart_text = String::from_utf8_lossy(chart_xml);
    let chart_rels_part = part_rels_part_name(&chart_part);

    if let Some(blip_rel) = picture_relationship_id(&chart_text) {
        if let Some(image) = resolve_part_image(package, &chart_rels_part, &blip_rel) {
            return Some(image);
        }
        if let Some(image) = media.resolve(&blip_rel).filter(|img| !img.bytes.is_empty()) {
            return Some(image);
        }
    }

    if let Some(user_shapes_rel) = read_attr_value(&chart_text, "c:userShapes", "r:id") {
        if let Some(image) =
            resolve_drawing_preview_from_rels(package, &chart_rels_part, &user_shapes_rel)
        {
            return Some(image);
        }
    }

    resolve_first_media_in_rels(package, &chart_rels_part, chart_part.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("word"))
}

fn part_rels_part_name(part: &str) -> String {
    // word/charts/chart1.xml -> word/charts/_rels/chart1.xml.rels
    if let Some((dir, file)) = part.rsplit_once('/') {
        format!("{dir}/_rels/{file}.rels")
    } else {
        format!("_rels/{part}.rels")
    }
}

fn resolve_drawing_preview_from_rels(
    package: &DocxPackage,
    rels_part: &str,
    relationship_id: &str,
) -> Option<ImageData> {
    let rels_bytes = package.parts.get(rels_part)?;
    let rels = parse_relationships(&String::from_utf8_lossy(rels_bytes));
    let target = rels.get(relationship_id)?;
    let part_dir = rels_part
        .rsplit_once("/_rels/")
        .map(|(dir, _)| dir)
        .unwrap_or("word");
    let drawing_part = normalize_part_path(part_dir, target);
    let drawing_xml = package.parts.get(&drawing_part)?;
    let drawing_text = String::from_utf8_lossy(drawing_xml);
    let blip_rel = picture_relationship_id(&drawing_text)?;
    let drawing_rels = part_rels_part_name(&drawing_part);
    resolve_part_image(package, &drawing_rels, &blip_rel)
}

fn resolve_first_media_in_rels(
    package: &DocxPackage,
    rels_part: &str,
    part_dir: &str,
) -> Option<ImageData> {
    let rels_bytes = package.parts.get(rels_part)?;
    let rels_text = String::from_utf8_lossy(rels_bytes);
    for element in crate::xml_util::split_elements(&rels_text, "Relationship") {
        let Some(target) = crate::xml_util::read_own_attr(element, "Target") else {
            continue;
        };
        if !is_media_relationship_target(target) {
            continue;
        }
        let media_part = normalize_part_path(part_dir, target);
        let bytes = package.parts.get(&media_part)?;
        if bytes.is_empty() {
            continue;
        }
        return Some(ImageData {
            asset_id: media_part.clone(),
            mime_type: mime_for_part(&media_part).to_string(),
            width_px: 0,
            height_px: 0,
            bytes: bytes.clone(),
        });
    }
    None
}

fn is_media_relationship_target(target: &str) -> bool {
    target.contains("media/")
        || target.ends_with(".png")
        || target.ends_with(".jpg")
        || target.ends_with(".jpeg")
        || target.ends_with(".gif")
        || target.ends_with(".bmp")
        || target.ends_with(".webp")
        || target.ends_with(".tif")
        || target.ends_with(".tiff")
        || target.ends_with(".emf")
        || target.ends_with(".wmf")
}

fn normalize_part_path(base: &str, target: &str) -> String {
    let mut path = base.to_string();
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if let Some(idx) = path.rfind('/') {
                    path.truncate(idx);
                }
            }
            part => {
                if !path.is_empty() {
                    path.push('/');
                }
                path.push_str(part);
            }
        }
    }
    path
}

fn resolve_part_image(
    package: &DocxPackage,
    rels_part: &str,
    relationship_id: &str,
) -> Option<ImageData> {
    let rels_bytes = package.parts.get(rels_part)?;
    let rels = parse_relationships(&String::from_utf8_lossy(rels_bytes));
    let target = rels.get(relationship_id)?;
    let part_dir = rels_part
        .rsplit_once("/_rels/")
        .map(|(dir, _)| dir)
        .unwrap_or("word");
    let media_part = normalize_part_path(part_dir, target);
    let bytes = package.parts.get(&media_part)?;
    if bytes.is_empty() {
        return None;
    }
    Some(ImageData {
        asset_id: media_part.clone(),
        mime_type: mime_for_part(&media_part).to_string(),
        width_px: 0,
        height_px: 0,
        bytes: bytes.clone(),
    })
}

fn parse_relationships(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for element in split_elements(xml, "Relationship") {
        let (Some(id), Some(target)) = (
            read_own_attr(element, "Id"),
            read_own_attr(element, "Target"),
        ) else {
            continue;
        };
        map.insert(id.to_string(), target.to_string());
    }
    map
}

fn mime_for_part(part_name: &str) -> &'static str {
    match part_name.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("webp") => "image/webp",
        Some("tif") | Some("tiff") => "image/tiff",
        Some("emf") => "image/x-emf",
        Some("wmf") => "image/x-wmf",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

/// Parse VML `style="width:…;height:…pt"` and DrawingML extent fallbacks.
fn parse_vml_or_drawing_size(para_xml: &str) -> (f32, f32) {
    let style = ["v:rect", "v:line", "v:shape", "v:oval"]
        .iter()
        .find_map(|tag| read_attr_value(para_xml, tag, "style"))
        .or_else(|| {
            para_xml
                .split("style=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .map(|s| s.to_string())
        });
    if let Some(style) = style {
        let w = parse_vml_style_length(&style, "width").unwrap_or(0.0);
        let h = parse_vml_style_length(&style, "height").unwrap_or(0.0);
        return (w, h);
    }
    (100.0, 50.0)
}

fn parse_vml_style_length(style: &str, prop: &str) -> Option<f32> {
    for part in style.split(';') {
        let part = part.trim();
        let Some((key, val)) = part.split_once(':') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case(prop) {
            continue;
        }
        let val = val.trim();
        if let Some(num) = val.strip_suffix("pt") {
            return num.trim().parse().ok();
        }
        if let Some(num) = val.strip_suffix("in") {
            return num.trim().parse::<f32>().ok().map(|v| v * 72.0);
        }
        if let Some(num) = val.strip_suffix("cm") {
            return num.trim().parse::<f32>().ok().map(|v| v * 72.0 / 2.54);
        }
        if val.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return val.parse().ok();
        }
    }
    None
}

/// Parse `wp:wrapPolygon` contour points (EMU → pt, relative to image origin).
fn parse_wrap_polygon(para_xml: &str) -> Option<Vec<(f32, f32)>> {
    if !para_xml.contains("<wp:wrapPolygon") {
        return None;
    }
    let mut points = Vec::new();
    if let (Some(x), Some(y)) = (
        read_attr_value(para_xml, "wp:start", "x").and_then(|v| v.parse::<f32>().ok()),
        read_attr_value(para_xml, "wp:start", "y").and_then(|v| v.parse::<f32>().ok()),
    ) {
        points.push((x / EMU_PER_POINT, y / EMU_PER_POINT));
    }
    for element in split_elements(para_xml, "wp:lineTo") {
        let Some(x) = read_own_attr(element, "x").and_then(|v| v.parse::<f32>().ok()) else {
            continue;
        };
        let Some(y) = read_own_attr(element, "y").and_then(|v| v.parse::<f32>().ok()) else {
            continue;
        };
        points.push((x / EMU_PER_POINT, y / EMU_PER_POINT));
    }
    if points.len() >= 3 {
        Some(points)
    } else {
        None
    }
}

/// Finds the relationship id of the image a drawing displays, if it displays
/// one: `<a:blip>` for DrawingML, `<v:imagedata>` for legacy VML pictures.
fn picture_relationship_id(para_xml: &str) -> Option<String> {
    read_attr_value(para_xml, "a:blip", "r:embed")
        .or_else(|| read_attr_value(para_xml, "a:blip", "r:link"))
        .or_else(|| read_attr_value(para_xml, "v:imagedata", "r:id"))
        .or_else(|| read_attr_value(para_xml, "v:imagedata", "r:embed"))
}

/// Resolves a `r:embed` relationship id to the image bytes it points at.
pub trait MediaResolver {
    fn resolve(&self, relationship_id: &str) -> Option<tw_model::ImageData>;
}

fn parse_anchor(para_xml: &str) -> Option<tw_model::ImageAnchor> {
    if !para_xml.contains("<wp:anchor") {
        return None;
    }
    let (x, origin_x) = parse_position(para_xml, "wp:positionH");
    let (y, origin_y) = parse_position(para_xml, "wp:positionV");
    Some(tw_model::ImageAnchor {
        x,
        y,
        origin_x,
        origin_y,
    })
}

/// Map DrawingML wrap children / `behindDoc` onto our `TextWrap` enum.
fn parse_text_wrap(para_xml: &str) -> tw_model::TextWrap {
    if para_xml.contains("<wp:wrapSquare") {
        return tw_model::TextWrap::Square;
    }
    if para_xml.contains("<wp:wrapTight") {
        return tw_model::TextWrap::Tight;
    }
    if para_xml.contains("<wp:wrapThrough") {
        return tw_model::TextWrap::Through;
    }
    if para_xml.contains("<wp:wrapTopAndBottom") {
        return tw_model::TextWrap::TopBottom;
    }
    if para_xml.contains("<wp:wrapNone") {
        let behind = read_attr_value(para_xml, "wp:anchor", "behindDoc")
            .as_deref()
            == Some("1");
        return if behind {
            tw_model::TextWrap::Behind
        } else {
            tw_model::TextWrap::InFront
        };
    }
    // Anchors without an explicit wrap child still honour behindDoc.
    if read_attr_value(para_xml, "wp:anchor", "behindDoc").as_deref() == Some("1") {
        tw_model::TextWrap::Behind
    } else {
        tw_model::TextWrap::Square
    }
}

fn parse_position(para_xml: &str, tag: &str) -> (f32, tw_model::AnchorOrigin) {
    let Some(start) = para_xml.find(&format!("<{tag}")) else {
        return (0.0, tw_model::AnchorOrigin::Column);
    };
    let fragment = &para_xml[start..];
    let end = fragment
        .find(&format!("</{tag}>"))
        .unwrap_or(fragment.len());
    let fragment = &fragment[..end];

    let origin = match read_attr_value(fragment, tag, "relativeFrom").as_deref() {
        Some("page") => tw_model::AnchorOrigin::Page,
        Some("margin") | Some("leftMargin") | Some("rightMargin") | Some("topMargin")
        | Some("bottomMargin") => tw_model::AnchorOrigin::Margin,
        Some("paragraph") => tw_model::AnchorOrigin::Paragraph,
        _ => tw_model::AnchorOrigin::Column,
    };
    let offset = fragment
        .find("<wp:posOffset>")
        .and_then(|start| {
            let rest = &fragment[start + 14..];
            rest.find("</wp:posOffset>")
                .map(|end| &rest[..end])
        })
        .and_then(|v| v.trim().parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(0.0);
    (offset, origin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_table() {
        let doc = Document::new();
        let xml = r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml, &doc);
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].cells.len(), 2);
    }

    #[test]
    fn reads_column_widths_from_grid_in_points() {
        let doc = Document::new();
        let xml = r#"<w:tbl><w:tblGrid><w:gridCol w:w="2880"/><w:gridCol w:w="1440"/></w:tblGrid>
            <w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml, &doc);
        assert_eq!(table.format.column_widths, vec![144.0, 72.0]);
        assert_eq!(table.format.width, Some(216.0));
    }

    #[test]
    fn falls_back_to_default_widths_without_a_grid() {
        let doc = Document::new();
        let xml = r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml, &doc);
        assert_eq!(table.format.column_widths.len(), 2);
        assert!(table.format.column_widths.iter().all(|w| *w > 0.0));
    }

    #[test]
    fn grid_span_widens_the_column_count() {
        let doc = Document::new();
        let xml = r#"<w:tbl>
            <w:tr><w:tc><w:tcPr><w:gridSpan w:val="3"/></w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
            <w:tr><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>D</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml, &doc);
        assert_eq!(table.rows[0].cells[0].format.colspan, 3);
        assert_eq!(table.format.column_widths.len(), 3);
    }

    #[test]
    fn cell_and_row_properties_do_not_create_phantom_cells() {
        let doc = Document::new();
        // Shape Word actually emits: every row and cell carries a `*Pr` child.
        let xml = r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr>
            <w:tblGrid><w:gridCol w:w="4680"/><w:gridCol w:w="4680"/></w:tblGrid>
            <w:tr><w:trPr><w:trHeight w:val="300"/></w:trPr>
              <w:tc><w:tcPr><w:tcW w:w="4680" w:type="dxa"/></w:tcPr>
                <w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>COMPETENCY</w:t></w:r></w:p></w:tc>
              <w:tc><w:tcPr><w:tcW w:w="4680" w:type="dxa"/></w:tcPr>
                <w:p><w:r><w:t>SKILL SETS</w:t></w:r></w:p></w:tc>
            </w:tr></w:tbl>"#;
        let table = parse_table(xml, &doc);

        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].cells.len(), 2, "no phantom cells from w:tcPr");
        assert_eq!(table.format.column_widths.len(), 2);
        for cell in &table.rows[0].cells {
            assert_eq!(cell.blocks.len(), 1, "no phantom paragraphs from w:pPr");
        }
    }

    #[test]
    fn reads_row_height_in_points() {
        let doc = Document::new();
        let xml = r#"<w:tbl><w:tr><w:trPr><w:trHeight w:val="480"/></w:trPr>
            <w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml, &doc);
        assert_eq!(table.rows[0].height, Some(24.0));
    }

    #[test]
    fn reads_cell_shading_and_borders() {
        let doc = Document::new();
        let xml = r#"<w:tbl><w:tr><w:tc><w:tcPr>
            <w:tcBorders><w:top w:val="single" w:sz="8" w:color="000000"/></w:tcBorders>
            <w:shd w:val="clear" w:fill="FFEE00"/>
            </w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml, &doc);
        let cell = &table.rows[0].cells[0];
        assert_eq!(cell.format.background.map(|c| c.r), Some(255));
        assert_eq!(cell.format.background.map(|c| c.g), Some(238));
        assert!(cell.format.border.is_some());
    }

    #[test]
    fn vertical_merge_sets_rowspan_and_drops_continue_cells() {
        let doc = Document::new();
        let xml = r#"<w:tbl>
            <w:tr><w:tc><w:tcPr><w:vMerge w:val="restart"/></w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr>
            <w:tr><w:tc><w:tcPr><w:vMerge/></w:tcPr><w:p/></w:tc>
            <w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc></w:tr>
            </w:tbl>"#;
        let table = parse_table(xml, &doc);
        assert_eq!(table.rows[0].cells[0].format.rowspan, 2);
        assert_eq!(table.rows[1].cells.len(), 1);
    }

    struct NoMedia;
    impl MediaResolver for NoMedia {
        fn resolve(&self, _: &str) -> Option<ImageData> {
            None
        }
    }

    #[test]
    fn vml_horizontal_rule_uses_style_height() {
        let xml = r##"<w:p><w:r><w:pict><v:rect o:hr="t" style="width:0;height:1.5pt"/></w:pict></w:r></w:p>"##;
        let shape = parse_shape_block(xml, &NoMedia, &DocxPackage::default()).expect("shape");
        assert!(
            (shape.shape.height - 1.5).abs() < 0.01,
            "height {}",
            shape.shape.height
        );
        assert_eq!(shape.shape.width, 0.0);
    }
}
