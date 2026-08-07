use tw_model::{
    Block, BorderSpec, Color, Document, ImageBlock, Paragraph, Table, TableCell, TableRow,
    VerticalAlign, NodeId, TableFormat,
};

use crate::paragraph::parse_paragraph;
use crate::xml_util::{
    read_attr_value, read_int_attr, split_elements, twips_to_points,
};

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
        block.wrap = tw_model::TextWrap::Behind;
    }
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
pub fn parse_shape_block(para_xml: &str) -> Option<tw_model::ShapeBlock> {
    if !para_xml.contains("<w:drawing") && !para_xml.contains("<w:pict") {
        return None;
    }
    if picture_relationship_id(para_xml).is_some() {
        return None;
    }
    let width = read_attr_value(para_xml, "wp:extent", "cx")
        .and_then(|v| v.parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(100.0);
    let height = read_attr_value(para_xml, "wp:extent", "cy")
        .and_then(|v| v.parse::<f32>().ok())
        .map(|emu| emu / EMU_PER_POINT)
        .unwrap_or(50.0);
    let shape_type = if para_xml.contains("wps:wsp") || para_xml.contains("wordprocessingShape") {
        tw_model::ShapeKind::TextBox
    } else if para_xml.contains("<v:line") || para_xml.contains("<v:rect") {
        tw_model::ShapeKind::Line
    } else {
        tw_model::ShapeKind::Rectangle
    };
    Some(tw_model::ShapeBlock {
        id: NodeId::new(),
        shape: tw_model::ShapeData {
            shape_type,
            width,
            height,
        },
        wrap: tw_model::TextWrap::Square,
    })
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
        Some("margin") | Some("leftMargin") | Some("topMargin") => tw_model::AnchorOrigin::Margin,
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
}
