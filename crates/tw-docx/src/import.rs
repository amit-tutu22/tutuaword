use std::collections::HashMap;
use std::io::{Cursor, Read};

use tw_model::{Block, Document, Paragraph};
use zip::ZipArchive;

use crate::paragraph::{paragraph_properties_xml, parse_paragraph};
use crate::styles::{
    parse_numbering_xml, parse_para_properties, parse_section_properties, parse_styles_xml,
    parse_theme_xml,
};
use crate::table::{parse_image_block, parse_table, MediaResolver};
use crate::xml_util::{
    extract_body_xml, extract_plain_text, iter_body_blocks, read_attr_value, read_own_attr,
    split_elements, BlockKind,
};
use crate::{DocxError, DocxPackage, ImportResult};

pub fn import_docx(source: &[u8]) -> Result<ImportResult, DocxError> {
    let cursor = Cursor::new(source);
    let mut archive = ZipArchive::new(cursor)?;

    let mut package = DocxPackage {
        original_bytes: Some(source.to_vec()),
        ..DocxPackage::default()
    };

    let mut document_xml = None;
    let mut styles_xml = None;
    let mut numbering_xml = None;
    let mut theme_xml = None;
    let mut header_parts: HashMap<String, String> = HashMap::new();
    let mut footer_parts: HashMap<String, String> = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        match name.as_str() {
            "word/document.xml" => {
                document_xml = Some(String::from_utf8_lossy(&data).into_owned());
            }
            "word/styles.xml" => {
                styles_xml = Some(String::from_utf8_lossy(&data).into_owned());
            }
            "word/numbering.xml" => {
                numbering_xml = Some(String::from_utf8_lossy(&data).into_owned());
            }
            "word/theme/theme1.xml" => {
                theme_xml = Some(String::from_utf8_lossy(&data).into_owned());
            }
            n if n.starts_with("word/header") && n.ends_with(".xml") => {
                header_parts.insert(name.clone(), String::from_utf8_lossy(&data).into_owned());
            }
            n if n.starts_with("word/footer") && n.ends_with(".xml") => {
                footer_parts.insert(name.clone(), String::from_utf8_lossy(&data).into_owned());
            }
            _ => {}
        }
        package.parts.insert(name, data);
    }

    let xml = document_xml.ok_or(DocxError::MissingDocumentPart)?;
    let media = PackageMedia::new(&package);
    let mut document =
        parse_document_xml(&xml, &styles_xml, &numbering_xml, &theme_xml, &media);

    apply_headers_footers(&mut document, &xml, &header_parts, &footer_parts);
    package.source_fingerprint = Some(crate::fingerprint::document_fingerprint(&document));

    Ok(ImportResult { document, package })
}

/// Resolves `r:embed` ids against `word/_rels/document.xml.rels` and pulls the
/// referenced bytes out of the package.
struct PackageMedia<'a> {
    package: &'a DocxPackage,
    relationships: HashMap<String, String>,
}

impl<'a> PackageMedia<'a> {
    fn new(package: &'a DocxPackage) -> Self {
        let relationships = package
            .parts
            .get("word/_rels/document.xml.rels")
            .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
            .unwrap_or_default();
        Self {
            package,
            relationships,
        }
    }
}

impl MediaResolver for PackageMedia<'_> {
    fn resolve(&self, relationship_id: &str) -> Option<tw_model::ImageData> {
        let target = self.relationships.get(relationship_id)?;
        // Relationship targets are relative to the part's folder.
        let part_name = format!("word/{}", target.trim_start_matches("./"));
        let bytes = self.package.parts.get(&part_name)?;
        if bytes.is_empty() {
            return None;
        }
        Some(tw_model::ImageData {
            asset_id: part_name.clone(),
            mime_type: mime_for(&part_name).to_string(),
            width_px: 0,
            height_px: 0,
            bytes: bytes.clone(),
        })
    }
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

fn mime_for(part_name: &str) -> &'static str {
    match part_name.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("webp") => "image/webp",
        Some("tif") | Some("tiff") => "image/tiff",
        Some("emf") => "image/x-emf",
        Some("wmf") => "image/x-wmf",
        _ => "application/octet-stream",
    }
}

fn parse_document_xml(
    xml: &str,
    styles_xml: &Option<String>,
    numbering_xml: &Option<String>,
    theme_xml: &Option<String>,
    media: &dyn MediaResolver,
) -> Document {
    let mut doc = Document::new();

    if let Some(styles) = styles_xml {
        doc.styles = parse_styles_xml(styles);
    }
    if let Some(numbering) = numbering_xml {
        doc.settings.numbering = parse_numbering_xml(numbering);
    }
    if let Some(theme) = theme_xml {
        doc.settings.theme = parse_theme_xml(theme);
    }

    let body = extract_body_xml(xml);
    let mut blocks = Vec::new();
    let mut section_format = None;

    for (chunk, kind) in iter_body_blocks(body) {
        match kind {
            BlockKind::Paragraph => {
                let image = parse_image_block(chunk, media);
                if let Some(para) = parse_paragraph(&doc, chunk) {
                    blocks.push(Block::Paragraph(para));
                } else if image.is_none() {
                    // A paragraph holding only a drawing contributes the image
                    // alone, but an empty one still occupies its own line.
                    let mut para = Paragraph::new();
                    para.format = parse_para_properties(paragraph_properties_xml(chunk));
                    blocks.push(Block::Paragraph(para));
                }
                if let Some(img) = image {
                    blocks.push(Block::ImageBlock(img));
                }
            }
            BlockKind::Table => {
                blocks.push(Block::Table(parse_table(chunk)));
            }
            BlockKind::SectionProps => {
                section_format = Some(parse_section_properties(chunk));
            }
        }
    }

    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }

    if let Some(section) = doc.sections.first_mut() {
        section.blocks = blocks;
        if let Some(format) = section_format {
            section.format = format;
        }
    }

    crate::styles::resolve_theme_fonts(&mut doc);
    doc
}

fn apply_headers_footers(
    doc: &mut Document,
    document_xml: &str,
    headers: &HashMap<String, String>,
    footers: &HashMap<String, String>,
) {
    let body = extract_body_xml(document_xml);
    if let Some((sect_chunk, _)) = iter_body_blocks(body)
        .into_iter()
        .find(|(_, k)| *k == BlockKind::SectionProps)
    {
        if let Some(header_ref) = read_attr_value(sect_chunk, "w:headerReference", "r:id") {
            if let Some((_, text)) = headers.iter().find(|(k, _)| k.contains(&header_ref)) {
                if let Some(section) = doc.sections.first_mut() {
                    section.format.header_text = Some(extract_header_footer_text(text));
                }
            }
        }
        if let Some(footer_ref) = read_attr_value(sect_chunk, "w:footerReference", "r:id") {
            if let Some((_, text)) = footers.iter().find(|(k, _)| k.contains(&footer_ref)) {
                if let Some(section) = doc.sections.first_mut() {
                    section.format.footer_text = Some(extract_header_footer_text(text));
                }
            }
        }
    }
}

fn extract_header_footer_text(xml: &str) -> String {
    split_elements(xml, "w:p")
        .into_iter()
        .map(extract_plain_text)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn minimal_docx(document_xml: &str) -> Vec<u8> {
        minimal_docx_with_parts(document_xml, None, None)
    }

    fn minimal_docx_with_parts(
        document_xml: &str,
        styles_xml: Option<&str>,
        numbering_xml: Option<&str>,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buf));
            let options = SimpleFileOptions::default();
            zip.start_file("word/document.xml", options).unwrap();
            zip.write_all(document_xml.as_bytes()).unwrap();
            if let Some(styles) = styles_xml {
                zip.start_file("word/styles.xml", options).unwrap();
                zip.write_all(styles.as_bytes()).unwrap();
            }
            if let Some(numbering) = numbering_xml {
                zip.start_file("word/numbering.xml", options).unwrap();
                zip.write_all(numbering.as_bytes()).unwrap();
            }
            zip.start_file("[Content_Types].xml", options).unwrap();
            zip.write_all(b"<Types/>").unwrap();
            zip.start_file("word/_rels/document.xml.rels", options).unwrap();
            zip.write_all(b"<Relationships/>").unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn extracts_paragraph_text() {
        let xml = r#"<w:document><w:body>
            <w:p><w:r><w:t>Hello </w:t></w:r><w:r><w:t>World</w:t></w:r></w:p>
            <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
        </w:body></w:document>"#;
        let bytes = minimal_docx(xml);
        let result = import_docx(&bytes).unwrap();
        assert_eq!(
            result.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text(),
            "Hello World"
        );
    }

    #[test]
    fn parses_bold_and_italic_runs() {
        let xml = r#"<w:document><w:body>
            <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Bold</w:t></w:r>
            <w:r><w:rPr><w:i/></w:rPr><w:t> Italic</w:t></w:r></w:p>
        </w:body></w:document>"#;
        let result = import_docx(&minimal_docx(xml)).unwrap();
        let para = result.document.sections[0].blocks[0].paragraph().unwrap();
        assert_eq!(para.runs[0].format.bold, Some(true));
        assert_eq!(para.runs[1].format.italic, Some(true));
    }

    #[test]
    fn parses_font_size_and_alignment() {
        let xml = r#"<w:document><w:body>
            <w:p><w:pPr><w:jc w:val="center"/><w:spacing w:before="240" w:after="120"/>
            </w:pPr><w:r><w:rPr><w:sz w:val="24"/></w:rPr><w:t>Title</w:t></w:r></w:p>
        </w:body></w:document>"#;
        let result = import_docx(&minimal_docx(xml)).unwrap();
        let para = result.document.sections[0].blocks[0].paragraph().unwrap();
        assert_eq!(para.runs[0].format.font_size, Some(12.0));
        assert_eq!(para.format.alignment, Some(tw_model::Alignment::Center));
        assert!(para.format.space_before.is_some());
    }

    #[test]
    fn parses_table_block() {
        let xml = r#"<w:document><w:body>
            <w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
        </w:body></w:document>"#;
        let result = import_docx(&minimal_docx(xml)).unwrap();
        assert!(matches!(result.document.sections[0].blocks[0], Block::Table(_)));
    }

    #[test]
    fn preserves_unknown_parts_in_package() {
        let xml = r#"<w:document><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;
        let bytes = minimal_docx(xml);
        let result = import_docx(&bytes).unwrap();
        assert!(result.package.parts.contains_key("[Content_Types].xml"));
        assert!(result.package.parts.contains_key("word/_rels/document.xml.rels"));
    }
}
