use std::collections::HashMap;
use std::io::{Cursor, Read};

use tw_model::{Block, Document, Paragraph};
use zip::ZipArchive;

use crate::paragraph::{paragraph_properties_xml, parse_paragraph_with_retention};
use crate::preserve::PreservedParagraph;
use crate::properties::{
    parse_app_properties, parse_core_properties, parse_even_and_odd_headers_from_settings,
    parse_read_only_from_settings,
};
use crate::retention::{scan_ooxml_elements, ImportRetentionReport};
use crate::styles::{
    parse_numbering_xml, parse_para_properties, parse_section_properties, parse_styles_xml,
    parse_theme_xml,
};
use crate::table::{parse_image_block, parse_shape_block, parse_table, MediaResolver};
use crate::xml_util::{
    extract_body_xml, extract_plain_text, iter_body_blocks, read_own_attr,
    split_elements, BlockKind,
};
use crate::{DocxError, DocxPackage, ImportResult};

pub fn import_docx(source: &[u8]) -> Result<ImportResult, DocxError> {
    if crate::encryption::is_password_protected(source)? {
        return Err(DocxError::PasswordProtected);
    }

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
    let mut core_properties_xml = None;
    let mut app_properties_xml = None;
    let mut settings_xml = None;
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
            "word/settings.xml" => {
                settings_xml = Some(String::from_utf8_lossy(&data).into_owned());
            }
            "docProps/core.xml" => {
                core_properties_xml = Some(String::from_utf8_lossy(&data).into_owned());
            }
            "docProps/app.xml" => {
                app_properties_xml = Some(String::from_utf8_lossy(&data).into_owned());
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
    let relationships = package
        .parts
        .get("word/_rels/document.xml.rels")
        .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
        .unwrap_or_default();
    let mut retention = ImportRetentionReport::new();
    scan_ooxml_elements(&xml, &mut retention);
    for hf in header_parts.values().chain(footer_parts.values()) {
        scan_ooxml_elements(hf, &mut retention);
    }
    let mut preserved_paragraphs = crate::preserve::PreservedParagraphMap::new();
    let mut preserved_shapes = crate::preserve::PreservedShapeMap::new();
    let mut document = {
        let media = PackageMedia {
            package: &package,
            relationships: relationships.clone(),
        };
        let mut doc = parse_document_xml(
            &xml,
            &styles_xml,
            &numbering_xml,
            &theme_xml,
            &media,
            &package,
            &mut retention,
            &mut preserved_paragraphs,
            &mut preserved_shapes,
        );
        apply_headers_footers(
            &mut doc,
            &xml,
            &header_parts,
            &footer_parts,
            &relationships,
            &media,
            &package,
            &mut retention,
        );
        doc
    };
    package.preserved_paragraphs = preserved_paragraphs;
    package.preserved_shapes = preserved_shapes;

    for part in package.parts.keys() {
        if part.starts_with("word/diagrams/") {
            retention.record_encountered("diagramPart");
            retention.record_retained("diagramPart");
        }
        if part.starts_with("word/charts/") {
            retention.record_encountered("chartPart");
            retention.record_retained("chartPart");
        }
    }

    let mut props = tw_model::DocumentProperties::default();
    if let Some(core) = core_properties_xml.as_deref() {
        props.merge(parse_core_properties(core));
    }
    if let Some(app) = app_properties_xml.as_deref() {
        props.merge(parse_app_properties(app));
    }
    document.properties = props;

    if let Some(settings) = settings_xml.as_deref() {
        if parse_read_only_from_settings(settings) {
            document.settings.read_only = true;
        }
        if parse_even_and_odd_headers_from_settings(settings) {
            document.settings.even_and_odd_headers = true;
        }
    }

    package.source_fingerprint = Some(crate::fingerprint::document_fingerprint(&document));
    package.source_numbering_fingerprint =
        Some(crate::fingerprint::numbering_fingerprint(&document.settings.numbering));
    package.source_styles_fingerprint =
        Some(crate::fingerprint::styles_fingerprint(&document.styles));

    Ok(ImportResult {
        document,
        package,
        retention,
    })
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
        let data = tw_model::ImageData {
            asset_id: part_name.clone(),
            mime_type: mime_for(&part_name).to_string(),
            width_px: 0,
            height_px: 0,
            bytes: bytes.clone(),
        };
        Some(data)
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
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn parse_document_xml(
    xml: &str,
    styles_xml: &Option<String>,
    numbering_xml: &Option<String>,
    theme_xml: &Option<String>,
    media: &dyn MediaResolver,
    package: &DocxPackage,
    retention: &mut ImportRetentionReport,
    preserved_paragraphs: &mut crate::preserve::PreservedParagraphMap,
    preserved_shapes: &mut crate::preserve::PreservedShapeMap,
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
    let (mut blocks, section_format) = parse_body_blocks(
        body,
        &doc,
        media,
        package,
        retention,
        preserved_paragraphs,
        preserved_shapes,
    );

    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }

    if let Some(section) = doc.sections.first_mut() {
        section.blocks = blocks;
        if let Some(format) = section_format {
            section.format = format;
        }
    }

    crate::styles::resolve_theme(&mut doc);
    doc
}

fn parse_body_blocks(
    body: &str,
    doc: &Document,
    media: &dyn MediaResolver,
    package: &DocxPackage,
    retention: &mut ImportRetentionReport,
    preserved_paragraphs: &mut crate::preserve::PreservedParagraphMap,
    preserved_shapes: &mut crate::preserve::PreservedShapeMap,
) -> (Vec<Block>, Option<tw_model::SectionFormat>) {
    let mut blocks = Vec::new();
    let mut section_format = None;

    for (chunk, kind) in iter_body_blocks(body) {
        match kind {
            BlockKind::Paragraph => {
                let image = parse_image_block(chunk, media);
                let shape = if image.is_none() {
                    parse_shape_block(chunk, media, package)
                } else {
                    None
                };
                if let Some(shape) = shape {
                    retention.record_encountered("drawing");
                    retention.record_retained("drawing");
                    if shape.shape.shape_type == tw_model::ShapeKind::Diagram {
                        retention.record_encountered("diagram");
                        retention.record_retained("diagram");
                        if shape.preview_image.is_some() {
                            retention.record_encountered("diagramPreview");
                            retention.record_retained("diagramPreview");
                        }
                    }
                    if shape.shape.shape_type == tw_model::ShapeKind::Chart {
                        retention.record_encountered("chart");
                        retention.record_retained("chart");
                        if shape.preview_image.is_some() {
                            retention.record_encountered("chartPreview");
                            retention.record_retained("chartPreview");
                        }
                        if shape.chart_data.is_some() {
                            retention.record_encountered("chartData");
                            retention.record_retained("chartData");
                        }
                    }
                    preserved_shapes.insert(
                        shape.id,
                        PreservedParagraph {
                            xml: chunk.to_string(),
                            fingerprint: crate::fingerprint::shape_fingerprint(&shape),
                        },
                    );
                    blocks.push(Block::ShapeBlock(shape));
                } else if image.is_none() {
                    if let Some(para) =
                        parse_paragraph_with_retention(doc, chunk, Some(retention), Some(media))
                    {
                        preserved_paragraphs.insert(
                            para.id,
                            PreservedParagraph {
                                xml: chunk.to_string(),
                                fingerprint: crate::fingerprint::paragraph_fingerprint(&para),
                            },
                        );
                        blocks.push(Block::Paragraph(para));
                    } else {
                        let mut para = Paragraph::new();
                        para.format = parse_para_properties(paragraph_properties_xml(chunk));
                        blocks.push(Block::Paragraph(para));
                    }
                }
                if let Some(img) = image {
                    retention.record_encountered("drawing");
                    retention.record_retained("drawing");
                    blocks.push(Block::ImageBlock(img));
                }
            }
            BlockKind::Table => {
                blocks.push(Block::Table(parse_table(chunk, doc)));
            }
            BlockKind::SectionProps => {
                section_format = Some(parse_section_properties(chunk));
            }
        }
    }

    (blocks, section_format)
}

fn apply_headers_footers(
    doc: &mut Document,
    document_xml: &str,
    headers: &HashMap<String, String>,
    footers: &HashMap<String, String>,
    relationships: &HashMap<String, String>,
    media: &dyn MediaResolver,
    package: &DocxPackage,
    retention: &mut ImportRetentionReport,
) {
    let body = extract_body_xml(document_xml);
    if let Some((sect_chunk, _)) = iter_body_blocks(body)
        .into_iter()
        .find(|(_, k)| *k == BlockKind::SectionProps)
    {
        for element in split_elements(sect_chunk, "w:headerReference") {
            retention.record_encountered("headerReference");
            let ref_type = read_own_attr(element, "w:type").unwrap_or("default");
            let hf_type = tw_model::HeaderFooterType::from_ooxml(ref_type);
            let Some(ref_id) = read_own_attr(element, "r:id") else {
                continue;
            };
            if let Some(part) = part_for_relationship(relationships, &ref_id) {
                if let Some(xml) = headers.get(&part) {
                    let mut discard = crate::preserve::PreservedParagraphMap::new();
                    let mut discard_shapes = crate::preserve::PreservedShapeMap::new();
                    let (blocks, _) = parse_body_blocks(
                        extract_part_body(xml),
                        doc,
                        media,
                        package,
                        retention,
                        &mut discard,
                        &mut discard_shapes,
                    );
                    if let Some(section) = doc.sections.first_mut() {
                        let hf = if blocks.is_empty() {
                            tw_model::HeaderFooter {
                                blocks: Vec::new(),
                                plain_text: Some(extract_header_footer_text(xml)),
                            }
                        } else {
                            tw_model::HeaderFooter {
                                blocks,
                                plain_text: None,
                            }
                        };
                        section.headers.insert(hf_type, hf);
                        retention.record_retained("headerReference");
                    }
                }
            }
        }
        for element in split_elements(sect_chunk, "w:footerReference") {
            retention.record_encountered("footerReference");
            let ref_type = read_own_attr(element, "w:type").unwrap_or("default");
            let hf_type = tw_model::HeaderFooterType::from_ooxml(ref_type);
            let Some(ref_id) = read_own_attr(element, "r:id") else {
                continue;
            };
            if let Some(part) = part_for_relationship(relationships, &ref_id) {
                if let Some(xml) = footers.get(&part) {
                    let mut discard = crate::preserve::PreservedParagraphMap::new();
                    let mut discard_shapes = crate::preserve::PreservedShapeMap::new();
                    let (blocks, _) = parse_body_blocks(
                        extract_part_body(xml),
                        doc,
                        media,
                        package,
                        retention,
                        &mut discard,
                        &mut discard_shapes,
                    );
                    if let Some(section) = doc.sections.first_mut() {
                        let hf = if blocks.is_empty() {
                            tw_model::HeaderFooter {
                                blocks: Vec::new(),
                                plain_text: Some(extract_header_footer_text(xml)),
                            }
                        } else {
                            tw_model::HeaderFooter {
                                blocks,
                                plain_text: None,
                            }
                        };
                        section.footers.insert(hf_type, hf);
                        retention.record_retained("footerReference");
                    }
                }
            }
        }
    }
}

fn part_for_relationship(relationships: &HashMap<String, String>, ref_id: &str) -> Option<String> {
    relationships.get(ref_id).map(|target| {
        if target.starts_with("word/") {
            target.clone()
        } else {
            format!("word/{}", target.trim_start_matches("./"))
        }
    })
}

fn extract_part_body(xml: &str) -> &str {
    for tag in ["w:hdr", "w:ftr", "w:body"] {
        if let Some(start) = xml.find(&format!("<{tag}")) {
            let after = &xml[start..];
            if let Some(gt) = after.find('>') {
                let body = &after[gt + 1..];
                if let Some(end) = body.find(&format!("</{tag}>")) {
                    return &body[..end];
                }
            }
        }
    }
    xml
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
