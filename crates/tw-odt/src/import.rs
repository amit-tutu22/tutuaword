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

    // Walk opening tags so `<text:h>` and `<text:p>` both become paragraphs.
    let mut rest = xml;
    while let Some(rel) = find_block_open(rest) {
        let (tag, after_open) = rel;
        let (close, heading_level) = match tag {
            "h" => ("</text:h>", parse_outline_level(after_open)),
            _ => ("</text:p>", None),
        };
        let end = after_open.find(close).unwrap_or(after_open.len());
        let para_xml = &after_open[..end];
        rest = after_open.get(end + close.len()..).unwrap_or("");

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
        if runs.is_empty() {
            let text = extract_text_content(para_xml);
            if !text.is_empty() {
                runs.push(Run::new_text(text));
            }
        }
        if !runs.is_empty() {
            let mut para = Paragraph::new();
            para.runs = runs;
            if let Some(level) = heading_level {
                let name = format!("Heading {level}");
                if let Some(id) = doc.styles.find_style_by_name(&name).map(|s| s.id) {
                    para.style_id = Some(id);
                }
            }
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

fn find_block_open(xml: &str) -> Option<(&'static str, &str)> {
    let p = xml.find("<text:p");
    let h = xml.find("<text:h");
    match (p, h) {
        (Some(pi), Some(hi)) if hi < pi => {
            let after = xml.get(hi + "<text:h".len()..)?;
            Some(("h", after))
        }
        (Some(pi), _) => {
            let after = xml.get(pi + "<text:p".len()..)?;
            Some(("p", after))
        }
        (None, Some(hi)) => {
            let after = xml.get(hi + "<text:h".len()..)?;
            Some(("h", after))
        }
        _ => None,
    }
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
            // Preserve significant spaces inside spans (e.g. " body").
            out.push_str(&decode_xml_entities(&after[..end]));
            rest = &after[end..];
        } else {
            // Final text node with no following tag (typical for a span slice).
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
