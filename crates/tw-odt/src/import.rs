use std::io::{Cursor, Read};

use tw_model::{Block, Document, Paragraph, Run};
use zip::ZipArchive;

use crate::{ImportResult, OdtError, OdtPackage};

pub fn import_odt(source: &[u8]) -> Result<ImportResult, OdtError> {
    let cursor = Cursor::new(source);
    let mut archive = ZipArchive::new(cursor)?;

    let mut package = OdtPackage {
        original_bytes: Some(source.to_vec()),
        ..OdtPackage::default()
    };

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
    let document = parse_content_xml(&xml);

    Ok(ImportResult { document, package })
}

fn parse_content_xml(xml: &str) -> Document {
    let mut doc = Document::new();
    let mut blocks = Vec::new();

    for chunk in xml.split("<text:p").skip(1) {
        let end = chunk.find("</text:p>").unwrap_or(chunk.len());
        let para_xml = &chunk[..end];
        let mut runs = Vec::new();
        for span in para_xml.split("<text:span").skip(1) {
            let span_end = span.find("</text:span>").unwrap_or(span.len());
            let text = extract_text_content(&span[..span_end]);
            if text.is_empty() {
                continue;
            }
            let mut run = Run::new_text(text);
            if span.contains("Bold") || span.contains("bold") {
                run.format.bold = Some(true);
            }
            runs.push(run);
        }
        if runs.is_empty() {
            let text = extract_text_content(para_xml);
            if !text.is_empty() {
                runs.push(Run::new_text(text));
            }
        }
        if !runs.is_empty() {
            let mut para = Paragraph::new();
            para.runs = runs;
            blocks.push(Block::Paragraph(para));
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

fn extract_text_content(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find('>') {
        let after = &rest[start + 1..];
        if let Some(end) = after.find('<') {
            out.push_str(&decode_xml_entities(after[..end].trim()));
            rest = &after[end..];
        } else {
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

    #[test]
    fn extracts_odt_paragraphs() {
        let xml = r#"<office:document><office:body>
            <text:p><text:span>Hello ODT</text:span></text:p>
        </office:body></office:document>"#;
        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buf));
            let options = SimpleFileOptions::default();
            zip.start_file("content.xml", options).unwrap();
            zip.write_all(xml.as_bytes()).unwrap();
            zip.start_file("mimetype", options).unwrap();
            zip.write_all(b"application/vnd.oasis.opendocument.text").unwrap();
            zip.finish().unwrap();
        }
        let result = import_odt(&buf).unwrap();
        assert_eq!(
            result.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text(),
            "Hello ODT"
        );
        assert!(result.package.parts.contains_key("mimetype"));
    }
}
