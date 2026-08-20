use std::collections::HashMap;
use std::io::{Cursor, Read};

use tw_model::{
    Block, Document, ImageBlock, ImageData, NumberingRef, Paragraph, Run, Table, TableCell,
    TableRow,
};
use zip::ZipArchive;

use crate::{ImportResult, OdtError, OdtPackage};

pub fn import_odt(source: &[u8]) -> Result<ImportResult, OdtError> {
    let cursor = Cursor::new(source);
    let mut archive = ZipArchive::new(cursor)?;

    // Keep parts for round-trip/repack; avoid also cloning the full zip into
    // `original_bytes` (that doubled peak memory for large packages).
    let mut package = OdtPackage::default();

    let mut content_xml = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        if name == "content.xml" {
            content_xml = Some(String::from_utf8_lossy(&data).into_owned());
        }
        package.parts.insert(name, data);
    }

    let xml = content_xml.ok_or(OdtError::MissingContentPart)?;
    let document = parse_content_xml(&xml, &package.parts);

    Ok(ImportResult { document, package })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Heading,
    ListItem,
    Paragraph,
    Table,
    Image,
}

fn parse_content_xml(xml: &str, parts: &HashMap<String, Vec<u8>>) -> Document {
    let mut doc = Document::new();
    let mut blocks = Vec::new();
    let mut rest = xml;

    while let Some((kind, index)) = find_next_block(rest) {
        rest = &rest[index..];
        match kind {
            BlockKind::Heading => {
                if let Some((para, consumed)) = parse_heading_or_paragraph(rest, &doc, true, false)
                {
                    let slice = rest.get(..consumed).unwrap_or(rest);
                    blocks.push(Block::Paragraph(para));
                    for image in extract_images_from_xml(slice, parts) {
                        blocks.push(Block::ImageBlock(image));
                    }
                    rest = rest.get(consumed..).unwrap_or("");
                } else {
                    rest = advance_one(rest);
                }
            }
            BlockKind::ListItem => {
                if let Some((para, consumed)) = parse_list_item(rest, &doc) {
                    blocks.push(Block::Paragraph(para));
                    rest = rest.get(consumed..).unwrap_or("");
                } else {
                    rest = advance_one(rest);
                }
            }
            BlockKind::Paragraph => {
                if let Some((para, consumed)) =
                    parse_heading_or_paragraph(rest, &doc, false, false)
                {
                    let slice = rest.get(..consumed).unwrap_or(rest);
                    blocks.push(Block::Paragraph(para));
                    for image in extract_images_from_xml(slice, parts) {
                        blocks.push(Block::ImageBlock(image));
                    }
                    rest = rest.get(consumed..).unwrap_or("");
                } else {
                    rest = advance_one(rest);
                }
            }
            BlockKind::Table => {
                if let Some(table) = parse_table(rest) {
                    let (table, consumed) = table;
                    blocks.push(Block::Table(table));
                    rest = rest.get(consumed..).unwrap_or("");
                } else {
                    rest = advance_one(rest);
                }
            }
            BlockKind::Image => {
                if let Some(image) = parse_image_frame(rest, parts) {
                    let (image, consumed) = image;
                    blocks.push(Block::ImageBlock(image));
                    rest = rest.get(consumed..).unwrap_or("");
                } else {
                    rest = advance_one(rest);
                }
            }
        }
    }

    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }

    if let Some(section) = doc.sections.first_mut() {
        section.blocks = blocks;
    }
    doc
}

fn find_next_block(xml: &str) -> Option<(BlockKind, usize)> {
    let bytes = xml.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let Some(rel) = bytes[i..].iter().position(|&b| b == b'<') else {
            break;
        };
        let start = i + rel;
        let rest = &xml[start..];
        // Longer prefixes first so `<text:list-item` wins over `<text:p`.
        let kind = if rest.starts_with("<text:list-item") {
            Some(BlockKind::ListItem)
        } else if rest.starts_with("<text:h") {
            Some(BlockKind::Heading)
        } else if rest.starts_with("<text:p") {
            Some(BlockKind::Paragraph)
        } else if rest.starts_with("<table:table") {
            Some(BlockKind::Table)
        } else if rest.starts_with("<draw:frame") {
            Some(BlockKind::Image)
        } else {
            None
        };
        if let Some(kind) = kind {
            return Some((kind, start));
        }
        i = start + 1;
    }
    None
}

fn advance_one(rest: &str) -> &str {
    rest.get(1..).unwrap_or("")
}

fn parse_heading_or_paragraph(
    xml: &str,
    doc: &Document,
    heading: bool,
    list_item: bool,
) -> Option<(Paragraph, usize)> {
    let open = if heading { "<text:h" } else { "<text:p" };
    let close = if heading { "</text:h>" } else { "</text:p>" };
    let start = xml.find(open)?;
    let after_open = &xml[start + open.len()..];
    let end_rel = after_open.find(close).unwrap_or(after_open.len());
    let para_xml = &after_open[..end_rel];
    let consumed = start + open.len() + end_rel + close.len();

    let mut runs = parse_runs_from_xml(para_xml);
    if runs.is_empty() {
        let text = extract_text_content(para_xml);
        if text.is_empty() {
            return None;
        }
        runs.push(Run::new_text(text));
    }

    let mut para = Paragraph::new();
    para.runs = runs;
    if heading {
        if let Some(level) = parse_outline_level(para_xml) {
            let name = format!("Heading {level}");
            if let Some(id) = doc.styles.find_style_by_name(&name).map(|s| s.id) {
                para.style_id = Some(id);
            }
        }
    }
    if list_item {
        para.format.numbering = Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        });
    }
    Some((para, consumed))
}

fn parse_list_item(xml: &str, doc: &Document) -> Option<(Paragraph, usize)> {
    let open = "<text:list-item";
    let close = "</text:list-item>";
    let start = xml.find(open)?;
    let after_open = &xml[start + open.len()..];
    let end_rel = after_open.find(close).unwrap_or(after_open.len());
    let item_xml = &after_open[..end_rel];
    let consumed = start + open.len() + end_rel + close.len();

    if let Some((para, _)) = parse_heading_or_paragraph(item_xml, doc, false, true) {
        return Some((para, consumed));
    }
    let text = extract_text_content(item_xml);
    if text.is_empty() {
        return None;
    }
    let mut para = Paragraph::with_text(text);
    para.format.numbering = Some(NumberingRef {
        numbering_id: 1,
        level: 0,
    });
    Some((para, consumed))
}

fn parse_runs_from_xml(para_xml: &str) -> Vec<Run> {
    let mut runs = Vec::new();
    for span in para_xml.split("<text:span").skip(1) {
        let span_end = span.find("</text:span>").unwrap_or(span.len());
        let span_xml = &span[..span_end];
        let text = extract_text_content(span_xml);
        if text.is_empty() {
            continue;
        }
        let mut run = Run::new_text(text);
        let style_hint = span_xml.to_ascii_lowercase();
        if style_hint.contains("bold") || style_hint.contains("font-weight=\"bold\"") {
            run.format.bold = Some(true);
        }
        if style_hint.contains("italic") || style_hint.contains("font-style=\"italic\"") {
            run.format.italic = Some(true);
        }
        runs.push(run);
    }
    runs
}

fn parse_table(xml: &str) -> Option<(Table, usize)> {
    let open = "<table:table";
    let close = "</table:table>";
    let start = xml.find(open)?;
    let after_open = &xml[start + open.len()..];
    let end_rel = after_open.find(close).unwrap_or(after_open.len());
    let table_xml = &after_open[..end_rel];
    let consumed = start + open.len() + end_rel + close.len();

    let mut rows = Vec::new();
    let mut rest = table_xml;
    while let Some(row_start) = rest.find("<table:table-row") {
        let after_row = &rest[row_start + "<table:table-row".len()..];
        let row_end = after_row.find("</table:table-row>").unwrap_or(after_row.len());
        let row_xml = &after_row[..row_end];
        rest = after_row.get(row_end + "</table:table-row>".len()..).unwrap_or("");

        let mut cells = Vec::new();
        let mut cell_rest = row_xml;
        while let Some(cell_start) = cell_rest.find("<table:table-cell") {
            let after_cell = &cell_rest[cell_start + "<table:table-cell".len()..];
            let cell_end = after_cell.find("</table:table-cell>").unwrap_or(after_cell.len());
            let cell_xml = &after_cell[..cell_end];
            cell_rest = after_cell.get(cell_end + "</table:table-cell>".len()..).unwrap_or("");

            let text = extract_text_content(cell_xml);
            let mut cell = TableCell::new();
            if text.is_empty() {
                cells.push(cell);
                continue;
            }
            let para = Paragraph::with_text(text);
            cell.blocks = vec![Block::Paragraph(para)];
            cells.push(cell);
        }
        if !cells.is_empty() {
            rows.push(TableRow::with_cells(cells));
        }
    }

    if rows.is_empty() {
        return None;
    }
    let cols = rows.iter().map(|r| r.cells.len()).max().unwrap_or(1) as u32;
    let mut table = Table::new(rows.len() as u32, cols);
    for (idx, row) in rows.into_iter().enumerate() {
        if let Some(dest) = table.rows.get_mut(idx) {
            dest.cells = row.cells;
        }
    }
    Some((table, consumed))
}

fn extract_images_from_xml(xml: &str, parts: &HashMap<String, Vec<u8>>) -> Vec<ImageBlock> {
    let mut images = Vec::new();
    let mut rest = xml;
    while rest.contains("<draw:frame") {
        match parse_image_frame(rest, parts) {
            Some((image, consumed)) if consumed > 0 => {
                images.push(image);
                rest = rest.get(consumed..).unwrap_or("");
            }
            _ => {
                if let Some(idx) = rest.find("<draw:frame") {
                    rest = rest.get(idx + 1..).unwrap_or("");
                } else {
                    break;
                }
            }
        }
    }
    images
}

fn parse_image_frame(xml: &str, parts: &HashMap<String, Vec<u8>>) -> Option<(ImageBlock, usize)> {
    let open = "<draw:frame";
    let close = "</draw:frame>";
    let start = xml.find(open)?;
    let after_open = &xml[start + open.len()..];
    let end_rel = after_open.find(close).unwrap_or(after_open.len());
    let frame_xml = &after_open[..end_rel];
    let consumed = start + open.len() + end_rel + close.len();

    let href = extract_attr(frame_xml, "xlink:href")
        .or_else(|| extract_attr(frame_xml, "draw:name"))?;
    let path = href.trim_start_matches("./");
    let bytes = parts.get(path).or_else(|| {
        parts
            .iter()
            .find(|(k, _)| k.ends_with(path))
            .map(|(_, v)| v)
    })?;
    if bytes.is_empty() {
        return None;
    }
    let data = ImageData::from_bytes(bytes.clone(), None);
    let (w, h) = data.display_size(400.0);
    Some((
        ImageBlock {
            id: tw_model::NodeId::new(),
            data,
            display_width: w,
            display_height: h,
            wrap: tw_model::TextWrap::Inline,
            anchor: None,
            transform: Default::default(),
            caption_paragraph_id: None,
            alt_text: extract_attr(frame_xml, "draw:name"),
            wrap_polygon: None,
        },
        consumed,
    ))
}

fn extract_attr(xml: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = xml.find(&needle)? + needle.len();
    let end = xml[start..].find('"')? + start;
    Some(decode_xml_entities(&xml[start..end]))
}

fn parse_outline_level(after_open: &str) -> Option<u8> {
    let marker = "text:outline-level=\"";
    let start = after_open.find(marker)? + marker.len();
    let end = after_open[start..].find('"')? + start;
    after_open[start..end].parse().ok()
}

fn extract_text_content(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find('>') {
        let after = &rest[start + 1..];
        if let Some(end) = after.find('<') {
            out.push_str(&decode_xml_entities(&after[..end]));
            rest = &after[end..];
        } else {
            out.push_str(&decode_xml_entities(after));
            break;
        }
    }
    out
}

fn decode_xml_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn zip_odt(content_xml: &str, extra: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buf));
            let options = SimpleFileOptions::default();
            zip.start_file("content.xml", options).unwrap();
            zip.write_all(content_xml.as_bytes()).unwrap();
            zip.start_file("mimetype", options).unwrap();
            zip.write_all(b"application/vnd.oasis.opendocument.text")
                .unwrap();
            for (name, data) in extra {
                zip.start_file(*name, options).unwrap();
                zip.write_all(data).unwrap();
            }
            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn extracts_odt_paragraphs() {
        let xml = r#"<office:document><office:body>
            <text:p><text:span>Hello ODT</text:span></text:p>
        </office:body></office:document>"#;
        let result = import_odt(&zip_odt(xml, &[])).unwrap();
        assert_eq!(
            result.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text(),
            "Hello ODT"
        );
        assert!(result.package.parts.contains_key("mimetype"));
    }

    #[test]
    fn u_f23_s5_odt_list_item_gets_numbering() {
        let xml = r#"<office:document><office:body>
            <text:list><text:list-item><text:p>Item one</text:p></text:list-item></text:list>
        </office:body></office:document>"#;
        let result = import_odt(&zip_odt(xml, &[])).unwrap();
        let para = result.document.sections[0].blocks[0].paragraph().unwrap();
        assert_eq!(para.full_text(), "Item one");
        assert_eq!(para.format.numbering.map(|n| n.numbering_id), Some(1));
    }

    #[test]
    fn u_f23_s5_odt_table_imports_block() {
        let xml = r#"<office:document><office:body>
            <table:table>
              <table:table-row>
                <table:table-cell><text:p>A</text:p></table:table-cell>
                <table:table-cell><text:p>B</text:p></table:table-cell>
              </table:table-row>
            </table:table>
        </office:body></office:document>"#;
        let result = import_odt(&zip_odt(xml, &[])).unwrap();
        let table = result.document.sections[0].blocks[0].table().unwrap();
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].cells.len(), 2);
    }

    #[test]
    fn u_f23_s5_odt_image_inside_paragraph() {
        let xml = r#"<office:document><office:body>
            <text:p>Before<draw:frame draw:name="Photo"><draw:image xlink:href="Pictures/x.png"/></draw:frame> after</text:p>
        </office:body></office:document>"#;
        let png = b"\x89PNG\r\n\x1a\n";
        let result = import_odt(&zip_odt(xml, &[("Pictures/x.png", png)])).unwrap();
        let has_image = result
            .document
            .sections[0]
            .blocks
            .iter()
            .any(|b| b.image().is_some());
        assert!(has_image, "expected ImageBlock nested in text:p");
    }
}
