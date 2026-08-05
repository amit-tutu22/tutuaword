use tw_model::{Block, CharFormat, Document, Paragraph, Run, UnderlineStyle};

use crate::{DocxError, DocxPackage};

pub fn export_docx(doc: &Document, package: &DocxPackage) -> Result<Vec<u8>, DocxError> {
    let mut pkg = package.clone();
    let document_xml = serialize_document_xml(doc);
    pkg.parts
        .insert("word/document.xml".into(), document_xml.into_bytes());
    pkg.mark_modified("word/document.xml".into());
    crate::opc::repack(&pkg)
}

fn serialize_document_xml(doc: &Document) -> String {
    let mut body = String::new();
    if let Some(section) = doc.sections.first() {
        for block in &section.blocks {
            match block {
                Block::Paragraph(para) => {
                    body.push_str(&serialize_paragraph(para, doc));
                }
                Block::Table(_) | Block::ImageBlock(_) => {
                    body.push_str("<w:p><w:r><w:t></w:t></w:r></w:p>");
                }
            }
        }
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}</w:body>
</w:document>"#
    )
}

fn serialize_paragraph(para: &Paragraph, doc: &Document) -> String {
    let mut xml = String::from("<w:p>");
    if let Some(style_id) = para.style_id {
        if let Some(style) = doc.styles.paragraph_styles.get(&style_id) {
            xml.push_str(&format!(
                r#"<w:pPr><w:pStyle w:val="{}"/></w:pPr>"#,
                escape_xml(&style.name.replace(' ', ""))
            ));
        }
    } else if para.runs.iter().any(|r| r.format.bold == Some(true)) {
        xml.push_str(r#"<w:pPr><w:pStyle w:val="Heading1"/></w:pPr>"#);
    }

    for run in &para.runs {
        xml.push_str(&serialize_run(run));
    }
    xml.push_str("</w:p>");
    xml
}

fn serialize_run(run: &Run) -> String {
    let text = run.text();
    if text.is_empty() {
        return String::new();
    }

    let mut xml = String::from("<w:r>");
    if run.format.bold == Some(true)
        || run.format.italic == Some(true)
        || run.format.underline.is_some()
        || run.revision.is_some()
    {
        xml.push_str("<w:rPr>");
        if run.format.bold == Some(true) {
            xml.push_str("<w:b/>");
        }
        if run.format.italic == Some(true) {
            xml.push_str("<w:i/>");
        }
        if run.format.underline.is_some() {
            xml.push_str("<w:u w:val=\"single\"/>");
        }
        if let Some(rev) = &run.revision {
            match rev.revision_type {
                tw_model::RevisionType::Insert => {
                    xml.push_str(&format!(
                        r#"<w:ins w:id="{}" w:author="{}" w:date="{}"/>"#,
                        rev.id,
                        escape_xml(&rev.author),
                        rev.timestamp.to_rfc3339()
                    ));
                }
                tw_model::RevisionType::Delete => {
                    xml.push_str(&format!(
                        r#"<w:del w:id="{}" w:author="{}" w:date="{}"/>"#,
                        rev.id,
                        escape_xml(&rev.author),
                        rev.timestamp.to_rfc3339()
                    ));
                }
            }
        }
        xml.push_str("</w:rPr>");
    }
    xml.push_str(&format!("<w:t>{}</w:t>", escape_xml(text)));
    xml.push_str("</w:r>");
    xml
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::{Block, Document};

    #[test]
    fn serializes_bold_run() {
        let mut doc = Document::with_paragraph("Hello");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.runs[0].format = CharFormat {
                bold: Some(true),
                ..Default::default()
            };
        }
        let xml = serialize_document_xml(&doc);
        assert!(xml.contains("<w:b/>"));
        assert!(xml.contains("Hello"));
    }
}
