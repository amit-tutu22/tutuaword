use tw_model::{Block, ImageBlock, Paragraph, Table, TableCell, TableRow, BorderSpec, NodeId, TableFormat};

use crate::paragraph::parse_paragraph_xml;
use crate::xml_util::{
    read_attr_value, read_int_attr, read_tag_text, split_elements, twips_to_points,
};

const FALLBACK_COLUMN_WIDTH: f32 = 100.0;

pub fn parse_table(tbl_xml: &str) -> Table {
    let mut rows = Vec::new();
    for chunk in split_elements(tbl_xml, "w:tr") {
        if let Some(row) = parse_table_row(chunk) {
            rows.push(row);
        }
    }

    // Widest row wins: the first row may merge cells via gridSpan.
    let cols = rows
        .iter()
        .map(|r| r.cells.iter().map(|c| c.format.colspan.max(1) as usize).sum::<usize>())
        .max()
        .unwrap_or(0)
        .max(1);

    let mut column_widths = parse_grid_columns(tbl_xml);
    if column_widths.is_empty() {
        column_widths = vec![FALLBACK_COLUMN_WIDTH; cols];
    } else if column_widths.len() < cols {
        column_widths.resize(cols, FALLBACK_COLUMN_WIDTH);
    }

    tw_model::Table {
        id: NodeId::new(),
        format: TableFormat {
            width: Some(column_widths.iter().sum()),
            column_widths,
            border: Some(BorderSpec::default()),
        },
        rows,
    }
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

fn parse_table_row(row_xml: &str) -> Option<TableRow> {
    let cells: Vec<TableCell> = split_elements(row_xml, "w:tc")
        .into_iter()
        .map(parse_table_cell)
        .collect();
    if cells.is_empty() {
        return None;
    }

    let mut row = TableRow::with_cells(cells);
    // `w:hRule="atLeast"` is a minimum; exact heights are handled the same way
    // since layout grows a row when its content does not fit.
    row.height = read_int_attr(row_xml, "w:trHeight", "w:val")
        .filter(|v| *v > 0)
        .map(|v| twips_to_points(v as f32));
    Some(row)
}

fn parse_table_cell(cell_xml: &str) -> TableCell {
    let mut cell = TableCell::new();
    let colspan = read_int_attr(cell_xml, "w:gridSpan", "w:val")
        .unwrap_or(1)
        .max(1) as u32;
    cell.format.colspan = colspan;

    let mut blocks = Vec::new();
    for chunk in split_elements(cell_xml, "w:p") {
        if let Some(para) = parse_paragraph_xml(chunk) {
            blocks.push(Block::Paragraph(para));
        }
    }
    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }
    cell.blocks = blocks;
    cell
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
    let offset = read_tag_text(fragment, "wp:posOffset")
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
        let xml = r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml);
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].cells.len(), 2);
    }

    #[test]
    fn reads_column_widths_from_grid_in_points() {
        let xml = r#"<w:tbl><w:tblGrid><w:gridCol w:w="2880"/><w:gridCol w:w="1440"/></w:tblGrid>
            <w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml);
        assert_eq!(table.format.column_widths, vec![144.0, 72.0]);
        assert_eq!(table.format.width, Some(216.0));
    }

    #[test]
    fn falls_back_to_default_widths_without_a_grid() {
        let xml = r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml);
        assert_eq!(table.format.column_widths.len(), 2);
        assert!(table.format.column_widths.iter().all(|w| *w > 0.0));
    }

    #[test]
    fn grid_span_widens_the_column_count() {
        let xml = r#"<w:tbl>
            <w:tr><w:tc><w:tcPr><w:gridSpan w:val="3"/></w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
            <w:tr><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>D</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml);
        assert_eq!(table.rows[0].cells[0].format.colspan, 3);
        assert_eq!(table.format.column_widths.len(), 3);
    }

    #[test]
    fn cell_and_row_properties_do_not_create_phantom_cells() {
        // Shape Word actually emits: every row and cell carries a `*Pr` child.
        let xml = r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr>
            <w:tblGrid><w:gridCol w:w="4680"/><w:gridCol w:w="4680"/></w:tblGrid>
            <w:tr><w:trPr><w:trHeight w:val="300"/></w:trPr>
              <w:tc><w:tcPr><w:tcW w:w="4680" w:type="dxa"/></w:tcPr>
                <w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>COMPETENCY</w:t></w:r></w:p></w:tc>
              <w:tc><w:tcPr><w:tcW w:w="4680" w:type="dxa"/></w:tcPr>
                <w:p><w:r><w:t>SKILL SETS</w:t></w:r></w:p></w:tc>
            </w:tr></w:tbl>"#;
        let table = parse_table(xml);

        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].cells.len(), 2, "no phantom cells from w:tcPr");
        assert_eq!(table.format.column_widths.len(), 2);
        for cell in &table.rows[0].cells {
            assert_eq!(cell.blocks.len(), 1, "no phantom paragraphs from w:pPr");
        }
    }

    #[test]
    fn reads_row_height_in_points() {
        let xml = r#"<w:tbl><w:tr><w:trPr><w:trHeight w:val="480"/></w:trPr>
            <w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let table = parse_table(xml);
        assert_eq!(table.rows[0].height, Some(24.0));
    }
}
