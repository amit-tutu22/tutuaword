//! Serializes a [`Document`] back to `word/document.xml`, together with any
//! media parts and relationships its images need.
//!
//! Element order inside `w:pPr`, `w:rPr`, `w:tcPr` and `w:sectPr` follows the
//! OOXML schema sequences. Word rejects properties that appear out of order.

use tw_model::{
    Alignment, Block, BorderSpec, CellFormat, CharFormat, Color, Document, ImageBlock, LineSpacing,
    NodeId, Paragraph, ParaFormat, Run, RunContent, StyleId, StyleSheet, TabAlignment, Table,
    TableCell, TableRow, TextWrap, UnderlineStyle, VerticalAlign,
};

use std::collections::HashMap;

use crate::media::MediaWriter;
use crate::{DocxError, DocxPackage, MINIMAL_CONTENT_TYPES};

const TWIPS_PER_POINT: f32 = 20.0;
/// English Metric Units per point (914400 per inch, 72 points per inch).
const EMU_PER_POINT: f32 = 12700.0;
/// `w:sz` on a border is measured in eighths of a point.
const BORDER_EIGHTHS_PER_POINT: f32 = 8.0;

/// `w:sectPr` children we do not model. Copied across from the source document
/// so that headers, footers, and column setup survive a save.
const PRESERVED_SECTION_CHILDREN: &[&str] = &[
    "w:headerReference",
    "w:footerReference",
    "w:footnotePr",
    "w:endnotePr",
    "w:type",
    "w:paperSrc",
    "w:pgBorders",
    "w:lnNumType",
    "w:pgNumType",
    "w:cols",
    "w:formProt",
    "w:vAlign",
    "w:noEndnote",
    "w:textDirection",
    "w:bidi",
    "w:rtlGutter",
    "w:docGrid",
];

pub fn export_docx(doc: &Document, package: &DocxPackage) -> Result<Vec<u8>, DocxError> {
    let mut pkg = package.clone();
    let mut media = MediaWriter::new(&pkg);
    let mut charts = crate::chart::ChartWriter::new(&pkg);
    let mut diagrams = crate::diagram::DiagramWriter::new(&pkg);
    let hyperlinks = crate::hyperlink::HyperlinkRels::build(doc, &pkg);
    for section in &doc.sections {
        for block in &section.blocks {
            if let Block::ShapeBlock(shape) = block {
                if shape.shape.shape_type == tw_model::ShapeKind::Chart {
                    if let Some(data) = &shape.chart_data {
                        charts.reference(shape, data);
                    }
                }
                if shape.shape.shape_type == tw_model::ShapeKind::Diagram {
                    let preserved = package.preserved_shapes.get(&shape.id);
                    let use_preserved = preserved
                        .map(|p| p.fingerprint == crate::fingerprint::shape_fingerprint(shape))
                        .unwrap_or(false);
                    if !use_preserved {
                        diagrams.reference(shape);
                    }
                }
            }
        }
    }
    let document_xml =
        serialize_document_xml(doc, package, &mut media, &charts, &diagrams, &hyperlinks);

    pkg.parts
        .insert("word/document.xml".into(), document_xml.into_bytes());
    pkg.mark_modified("word/document.xml".into());

    let numbering_changed = package
        .source_numbering_fingerprint
        .map(|f| f != crate::fingerprint::numbering_fingerprint(&doc.settings.numbering))
        .unwrap_or(true);
    if numbering_changed {
        let numbering_xml = crate::numbering::serialize_numbering_xml(&doc.settings.numbering);
        if !numbering_xml.is_empty() {
            pkg.parts
                .insert("word/numbering.xml".into(), numbering_xml.into_bytes());
            pkg.mark_modified("word/numbering.xml".into());
            ensure_numbering_content_type(&mut pkg);
            ensure_numbering_relationship(&mut pkg);
        }
    }

    let styles_changed = package
        .source_styles_fingerprint
        .map(|f| f != crate::fingerprint::styles_fingerprint(&doc.styles))
        .unwrap_or(true);
    if styles_changed {
        let styles_xml = serialize_styles_xml(&doc.styles);
        pkg.parts
            .insert("word/styles.xml".into(), styles_xml.into_bytes());
        pkg.mark_modified("word/styles.xml".into());
        ensure_styles_content_type(&mut pkg);
        ensure_styles_relationship(&mut pkg);
    }

    if !doc.footnotes.is_empty() {
        let footnotes_xml = serialize_footnotes_xml(
            doc,
            &pkg,
            &mut media,
            &charts,
            &diagrams,
            &hyperlinks,
            &mut RevisionIdAllocator::new(),
        );
        pkg.parts
            .insert("word/footnotes.xml".into(), footnotes_xml.into_bytes());
        pkg.mark_modified("word/footnotes.xml".into());
        ensure_footnotes_content_type(&mut pkg);
        ensure_footnotes_relationship(&mut pkg);
    }

    if !doc.endnotes.is_empty() {
        let endnotes_xml = serialize_endnotes_xml(
            doc,
            &pkg,
            &mut media,
            &charts,
            &diagrams,
            &hyperlinks,
            &mut RevisionIdAllocator::new(),
        );
        pkg.parts
            .insert("word/endnotes.xml".into(), endnotes_xml.into_bytes());
        pkg.mark_modified("word/endnotes.xml".into());
        ensure_endnotes_content_type(&mut pkg);
        ensure_endnotes_relationship(&mut pkg);
    }

    if !doc.bibliography_sources.is_empty() {
        let bibliography_xml = crate::bibliography::serialize_bibliography_xml(&doc.bibliography_sources);
        pkg.parts.insert(
            crate::bibliography::BIBLIOGRAPHY_PART.into(),
            bibliography_xml.into_bytes(),
        );
        pkg.mark_modified(crate::bibliography::BIBLIOGRAPHY_PART.into());
        ensure_bibliography_content_type(&mut pkg);
        ensure_bibliography_relationship(&mut pkg);
    }

    if !doc.comments.is_empty() {
        let comments_xml = crate::comments::serialize_comments_xml(&doc.comments);
        pkg.parts
            .insert(crate::comments::COMMENTS_PART.into(), comments_xml.into_bytes());
        pkg.mark_modified(crate::comments::COMMENTS_PART.into());
        ensure_comments_content_type(&mut pkg);
        ensure_comments_relationship(&mut pkg);
    } else {
        // Document Inspector may have cleared comments (F22.S3).
        strip_comments_part(&mut pkg);
    }

    if !doc.signatures.is_empty() {
        let signatures_xml = crate::signatures::serialize_signatures_xml(&doc.signatures);
        pkg.parts.insert(
            crate::signatures::SIGNATURES_PART.into(),
            signatures_xml.into_bytes(),
        );
        pkg.mark_modified(crate::signatures::SIGNATURES_PART.into());
        ensure_signatures_content_type(&mut pkg);
        ensure_signatures_relationship(&mut pkg);
    } else {
        strip_signatures_part(&mut pkg);
    }

    write_core_properties(&mut pkg, &doc.properties);

    media.commit(&mut pkg);
    charts.commit(&mut pkg);
    diagrams.commit(&mut pkg);
    hyperlinks.commit(&mut pkg);

    patch_settings_xml(&mut pkg, doc);

    crate::opc::repack(&pkg)
}

fn ensure_numbering_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg.parts.get(part).cloned().unwrap_or_else(|| {
        MINIMAL_CONTENT_TYPES.to_vec()
    });
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>"#;
    if !xml.contains("/word/numbering.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_numbering_relationship(pkg: &mut DocxPackage) {
    let part = "word/_rels/document.xml.rels";
    let bytes = pkg.parts.get(part).cloned().unwrap_or_else(|| b"<Relationships/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("numbering.xml") {
        let rel = r#"<Relationship Id="rIdNumbering" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!("<Relationships>{rel}</Relationships>");
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_styles_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg.parts.get(part).cloned().unwrap_or_else(|| {
        MINIMAL_CONTENT_TYPES.to_vec()
    });
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>"#;
    if !xml.contains("/word/styles.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_styles_relationship(pkg: &mut DocxPackage) {
    let part = "word/_rels/document.xml.rels";
    let bytes = pkg.parts.get(part).cloned().unwrap_or_else(|| b"<Relationships/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("styles.xml") {
        let rel = r#"<Relationship Id="rIdStyles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!("<Relationships>{rel}</Relationships>");
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_footnotes_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| MINIMAL_CONTENT_TYPES.to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/word/footnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"/>"#;
    if !xml.contains("/word/footnotes.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_footnotes_relationship(pkg: &mut DocxPackage) {
    let part = "word/_rels/document.xml.rels";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| b"<Relationships/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("footnotes.xml") {
        let rel = r#"<Relationship Id="rIdFootnotes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!("<Relationships>{rel}</Relationships>");
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_endnotes_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| MINIMAL_CONTENT_TYPES.to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/word/endnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml"/>"#;
    if !xml.contains("/word/endnotes.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_endnotes_relationship(pkg: &mut DocxPackage) {
    let part = "word/_rels/document.xml.rels";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| b"<Relationships/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("endnotes.xml") {
        let rel = r#"<Relationship Id="rIdEndnotes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes" Target="endnotes.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!("<Relationships>{rel}</Relationships>");
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_bibliography_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| MINIMAL_CONTENT_TYPES.to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/word/bibliography.xml" ContentType="application/xml"/>"#;
    if !xml.contains("/word/bibliography.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_bibliography_relationship(pkg: &mut DocxPackage) {
    let part = "word/_rels/document.xml.rels";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| b"<Relationships/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("bibliography.xml") {
        let rel = r#"<Relationship Id="rIdBibliography" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" Target="bibliography.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!("<Relationships>{rel}</Relationships>");
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_comments_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| MINIMAL_CONTENT_TYPES.to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/word/comments.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml"/>"#;
    if !xml.contains("/word/comments.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_comments_relationship(pkg: &mut DocxPackage) {
    let part = "word/_rels/document.xml.rels";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| b"<Relationships/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("comments.xml") {
        let rel = r#"<Relationship Id="rIdComments" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!("<Relationships>{rel}</Relationships>");
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn strip_comments_part(pkg: &mut DocxPackage) {
    let comments_part = crate::comments::COMMENTS_PART;
    if pkg.parts.remove(comments_part).is_some() {
        pkg.mark_modified(comments_part.into());
    }

    let ct_part = "[Content_Types].xml";
    if let Some(bytes) = pkg.parts.get(ct_part).cloned() {
        let mut xml = String::from_utf8_lossy(&bytes).into_owned();
        let before = xml.clone();
        // Remove Override for comments (self-closing or paired).
        while let Some(start) = xml.find(r#"PartName="/word/comments.xml""#) {
            let open = xml[..start].rfind("<Override").unwrap_or(start);
            let end = xml[start..]
                .find("/>")
                .map(|i| start + i + 2)
                .or_else(|| {
                    xml[start..]
                        .find("</Override>")
                        .map(|i| start + i + "</Override>".len())
                });
            if let Some(end) = end {
                xml.replace_range(open..end, "");
            } else {
                break;
            }
        }
        if xml != before {
            pkg.parts.insert(ct_part.into(), xml.into_bytes());
            pkg.mark_modified(ct_part.into());
        }
    }

    let rels_part = "word/_rels/document.xml.rels";
    if let Some(bytes) = pkg.parts.get(rels_part).cloned() {
        let mut xml = String::from_utf8_lossy(&bytes).into_owned();
        let before = xml.clone();
        while let Some(start) = xml.find("Target=\"comments.xml\"") {
            let open = xml[..start].rfind("<Relationship").unwrap_or(start);
            let end = xml[start..]
                .find("/>")
                .map(|i| start + i + 2)
                .or_else(|| {
                    xml[start..]
                        .find("</Relationship>")
                        .map(|i| start + i + "</Relationship>".len())
                });
            if let Some(end) = end {
                xml.replace_range(open..end, "");
            } else {
                break;
            }
        }
        if xml != before {
            pkg.parts.insert(rels_part.into(), xml.into_bytes());
            pkg.mark_modified(rels_part.into());
        }
    }
}

fn ensure_signatures_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| MINIMAL_CONTENT_TYPES.to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/customXml/digitalSignatures.xml" ContentType="application/xml"/>"#;
    if !xml.contains("/customXml/digitalSignatures.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_signatures_relationship(pkg: &mut DocxPackage) {
    let part = "word/_rels/document.xml.rels";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| b"<Relationships/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("digitalSignatures.xml") {
        let rel = r#"<Relationship Id="rIdDigitalSignatures" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" Target="../customXml/digitalSignatures.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!("<Relationships>{rel}</Relationships>");
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn strip_signatures_part(pkg: &mut DocxPackage) {
    let signatures_part = crate::signatures::SIGNATURES_PART;
    if pkg.parts.remove(signatures_part).is_some() {
        pkg.mark_modified(signatures_part.into());
    }

    let ct_part = "[Content_Types].xml";
    if let Some(bytes) = pkg.parts.get(ct_part).cloned() {
        let mut xml = String::from_utf8_lossy(&bytes).into_owned();
        let before = xml.clone();
        while let Some(start) = xml.find(r#"PartName="/customXml/digitalSignatures.xml""#) {
            let open = xml[..start].rfind("<Override").unwrap_or(start);
            let end = xml[start..]
                .find("/>")
                .map(|i| start + i + 2)
                .or_else(|| {
                    xml[start..]
                        .find("</Override>")
                        .map(|i| start + i + "</Override>".len())
                });
            if let Some(end) = end {
                xml.replace_range(open..end, "");
            } else {
                break;
            }
        }
        if xml != before {
            pkg.parts.insert(ct_part.into(), xml.into_bytes());
            pkg.mark_modified(ct_part.into());
        }
    }

    let rels_part = "word/_rels/document.xml.rels";
    if let Some(bytes) = pkg.parts.get(rels_part).cloned() {
        let mut xml = String::from_utf8_lossy(&bytes).into_owned();
        let before = xml.clone();
        while let Some(start) = xml.find("digitalSignatures.xml") {
            let open = xml[..start].rfind("<Relationship").unwrap_or(start);
            let end = xml[start..]
                .find("/>")
                .map(|i| start + i + 2)
                .or_else(|| {
                    xml[start..]
                        .find("</Relationship>")
                        .map(|i| start + i + "</Relationship>".len())
                });
            if let Some(end) = end {
                xml.replace_range(open..end, "");
            } else {
                break;
            }
        }
        if xml != before {
            pkg.parts.insert(rels_part.into(), xml.into_bytes());
            pkg.mark_modified(rels_part.into());
        }
    }
}

fn write_core_properties(pkg: &mut DocxPackage, props: &tw_model::DocumentProperties) {
    let part = crate::properties::CORE_PROPERTIES_PART;
    let xml = crate::properties::serialize_core_properties(props);
    pkg.parts.insert(part.into(), xml.into_bytes());
    pkg.mark_modified(part.into());
    ensure_core_properties_content_type(pkg);
    ensure_core_properties_relationship(pkg);
}

fn ensure_core_properties_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| MINIMAL_CONTENT_TYPES.to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#;
    if !xml.contains("/docProps/core.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_core_properties_relationship(pkg: &mut DocxPackage) {
    let part = "_rels/.rels";
    let bytes = pkg
        .parts
        .get(part)
        .cloned()
        .unwrap_or_else(|| b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("docProps/core.xml") {
        let rel = r#"<Relationship Id="rIdCore" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!(
                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{rel}</Relationships>"#
            );
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn serialize_footnotes_xml(
    doc: &Document,
    package: &DocxPackage,
    media: &mut MediaWriter,
    charts: &crate::chart::ChartWriter,
    diagrams: &crate::diagram::DiagramWriter,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    );
    xml.push_str(
        r#"<w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>"#,
    );
    xml.push_str(
        r#"<w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>"#,
    );
    for footnote in &doc.footnotes {
        xml.push_str(&format!(r#"<w:footnote w:id="{}">"#, footnote.id));
        for block in &footnote.blocks {
            xml.push_str(&serialize_block(
                block, doc, package, media, charts, diagrams, hyperlinks, revision_ids,
            ));
        }
        xml.push_str("</w:footnote>");
    }
    xml.push_str("</w:footnotes>");
    xml
}

fn serialize_endnotes_xml(
    doc: &Document,
    package: &DocxPackage,
    media: &mut MediaWriter,
    charts: &crate::chart::ChartWriter,
    diagrams: &crate::diagram::DiagramWriter,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:endnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    );
    xml.push_str(
        r#"<w:endnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:endnote>"#,
    );
    xml.push_str(
        r#"<w:endnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:endnote>"#,
    );
    for endnote in &doc.endnotes {
        xml.push_str(&format!(r#"<w:endnote w:id="{}">"#, endnote.id));
        for block in &endnote.blocks {
            xml.push_str(&serialize_block(
                block, doc, package, media, charts, diagrams, hyperlinks, revision_ids,
            ));
        }
        xml.push_str("</w:endnote>");
    }
    xml.push_str("</w:endnotes>");
    xml
}

fn serialize_document_xml(
    doc: &Document,
    source: &DocxPackage,
    media: &mut MediaWriter,
    charts: &crate::chart::ChartWriter,
    diagrams: &crate::diagram::DiagramWriter,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
) -> String {
    let original_sect_pr = original_section_properties(source);
    let mut body = String::new();
    let mut revision_ids = RevisionIdAllocator::new();

    let last = doc.sections.len().saturating_sub(1);
    for (index, section) in doc.sections.iter().enumerate() {
        for block in &section.blocks {
            body.push_str(&serialize_block(
                block,
                doc,
                source,
                media,
                charts,
                diagrams,
                hyperlinks,
                &mut revision_ids,
            ));
        }
        let sect_pr = serialize_section_properties(
            index,
            section,
            doc,
            original_sect_pr.as_deref(),
        );
        if index == last {
            body.push_str(&sect_pr);
        } else {
            // A section break before the last section is carried by the
            // paragraph that ends the section, which is how Word records it.
            body.push_str(&format!("<w:p><w:pPr>{sect_pr}</w:pPr></w:p>"));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <w:body>{body}</w:body>
</w:document>"#
    )
}

fn serialize_block(
    block: &Block,
    doc: &Document,
    package: &DocxPackage,
    media: &mut MediaWriter,
    charts: &crate::chart::ChartWriter,
    diagrams: &crate::diagram::DiagramWriter,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    match block {
        Block::Paragraph(para) => {
            serialize_paragraph(para, doc, package, hyperlinks, revision_ids)
        }
        Block::Table(table) => serialize_table(
            table,
            doc,
            package,
            media,
            charts,
            diagrams,
            hyperlinks,
            revision_ids,
        ),
        Block::ImageBlock(image) => serialize_image_paragraph(image, media),
        Block::ShapeBlock(shape) => serialize_shape_paragraph(shape, package, charts, diagrams),
        _ => String::from("<w:p/>"),
    }
}

// -- paragraphs ---------------------------------------------------------------

fn serialize_paragraph(
    para: &Paragraph,
    doc: &Document,
    package: &DocxPackage,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    if para.runs.len() == 1 {
        if let RunContent::OfficeMath { xml } = &para.runs[0].content {
            if xml.contains("<m:oMathPara") {
                if let Some(preserved) = package.preserved_paragraphs.get(&para.id) {
                    if preserved.fingerprint == crate::fingerprint::paragraph_fingerprint(para) {
                        return preserved.xml.clone();
                    }
                }
                return xml.clone();
            }
        }
    }

    if let Some(preserved) = package.preserved_paragraphs.get(&para.id) {
        if preserved.fingerprint == crate::fingerprint::paragraph_fingerprint(para) {
            return preserved.xml.clone();
        }
    }
    let mut xml = String::from("<w:p>");
    xml.push_str(&serialize_paragraph_properties(para, doc));
    for run in &para.runs {
        xml.push_str(&serialize_run(run, revision_ids, hyperlinks));
    }
    xml.push_str("</w:p>");
    xml
}

fn serialize_paragraph_properties(para: &Paragraph, doc: &Document) -> String {
    let format = &para.format;
    let mut props = String::new();

    if let Some(style_id) = para.style_id.and_then(|id| ooxml_style_id(doc, id)) {
        props.push_str(&format!(
            r#"<w:pStyle w:val="{}"/>"#,
            escape_xml(&style_id)
        ));
    }
    if format.keep_together == Some(true) {
        props.push_str("<w:keepLines/>");
    }
    if format.keep_with_next == Some(true) {
        props.push_str("<w:keepNext/>");
    }
    if let Some(widow) = format.widow_orphan_control {
        if widow {
            props.push_str("<w:widowControl/>");
        } else {
            props.push_str(r#"<w:widowControl w:val="0"/>"#);
        }
    }
    if format.page_break_before == Some(true) {
        props.push_str("<w:pageBreakBefore/>");
    }
    if let Some(numbering) = format.numbering {
        props.push_str(&format!(
            r#"<w:numPr><w:ilvl w:val="{}"/><w:numId w:val="{}"/>"#,
            numbering.level, numbering.numbering_id
        ));
        if format.num_restart == Some(true) {
            props.push_str(r#"<w:numRestart w:val="1"/>"#);
        }
        props.push_str("</w:numPr>");
    }
    if let Some(level) = format.outline_level {
        if level <= 8 {
            props.push_str(&format!(r#"<w:outlineLvl w:val="{}"/>"#, level));
        }
    }
    props.push_str(&serialize_spacing(format));
    props.push_str(&serialize_indent(format));
    props.push_str(&serialize_tab_stops(format));
    props.push_str(&serialize_para_shading(format));
    props.push_str(&serialize_para_borders(format));
    if let Some(alignment) = format.alignment {
        props.push_str(&format!(
            r#"<w:jc w:val="{}"/>"#,
            alignment_value(alignment)
        ));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<w:pPr>{props}</w:pPr>")
    }
}

fn serialize_spacing(format: &ParaFormat) -> String {
    let mut attrs = String::new();
    if let Some(before) = format.space_before {
        attrs.push_str(&format!(r#" w:before="{}""#, to_twips(before)));
    }
    if let Some(after) = format.space_after {
        attrs.push_str(&format!(r#" w:after="{}""#, to_twips(after)));
    }
    match &format.line_spacing {
        // `w:line` is in 240ths of a line for the `auto` rule, and in twips for
        // the two fixed rules.
        Some(LineSpacing::Single) => attrs.push_str(r#" w:line="240" w:lineRule="auto""#),
        Some(LineSpacing::Double) => attrs.push_str(r#" w:line="480" w:lineRule="auto""#),
        Some(LineSpacing::Multiple(m)) => attrs.push_str(&format!(
            r#" w:line="{}" w:lineRule="auto""#,
            (m * 240.0).round() as i32
        )),
        Some(LineSpacing::AtLeast(points)) => attrs.push_str(&format!(
            r#" w:line="{}" w:lineRule="atLeast""#,
            to_twips(*points)
        )),
        Some(LineSpacing::Exactly(points)) => attrs.push_str(&format!(
            r#" w:line="{}" w:lineRule="exact""#,
            to_twips(*points)
        )),
        None => {}
        Some(_) => {}
    }
    if attrs.is_empty() {
        String::new()
    } else {
        format!("<w:spacing{attrs}/>")
    }
}

fn serialize_indent(format: &ParaFormat) -> String {
    let mut attrs = String::new();
    if let Some(left) = format.indent_left {
        attrs.push_str(&format!(r#" w:left="{}""#, to_twips(left)));
    }
    if let Some(right) = format.indent_right {
        attrs.push_str(&format!(r#" w:right="{}""#, to_twips(right)));
    }
    // A negative first-line indent is a hanging indent; the two are exclusive.
    match format.indent_first_line {
        Some(first) if first < 0.0 => {
            attrs.push_str(&format!(r#" w:hanging="{}""#, to_twips(-first)))
        }
        Some(first) => attrs.push_str(&format!(r#" w:firstLine="{}""#, to_twips(first))),
        None => {}
    }
    if attrs.is_empty() {
        String::new()
    } else {
        format!("<w:ind{attrs}/>")
    }
}

fn serialize_tab_stops(format: &ParaFormat) -> String {
    let Some(stops) = format.tab_stops.as_ref().filter(|s| !s.is_empty()) else {
        return String::new();
    };
    let mut xml = String::from("<w:tabs>");
    for stop in stops {
        let align = match stop.alignment {
            TabAlignment::Center => "center",
            TabAlignment::Right => "right",
            TabAlignment::Decimal => "decimal",
            TabAlignment::Bar => "bar",
            TabAlignment::Left => "left",
            _ => "left",
        };
        let leader = match stop.leader {
            tw_model::TabLeader::Dots => r#" w:leader="dot""#,
            tw_model::TabLeader::MiddleDot => r#" w:leader="middleDot""#,
            tw_model::TabLeader::Hyphen => r#" w:leader="hyphen""#,
            tw_model::TabLeader::Underscore => r#" w:leader="underscore""#,
            tw_model::TabLeader::None => "",
            _ => "",
        };
        xml.push_str(&format!(
            r#"<w:tab w:val="{align}" w:pos="{}"{leader}/>"#,
            to_twips(stop.position)
        ));
    }
    xml.push_str("</w:tabs>");
    xml
}

fn serialize_para_shading(format: &ParaFormat) -> String {
    let Some(shading) = format.shading.filter(|c| c.a > 0) else {
        return String::new();
    };
    format!(
        r#"<w:shd w:val="clear" w:color="auto" w:fill="{}"/>"#,
        hex_rgb(shading)
    )
}

fn serialize_para_borders(format: &ParaFormat) -> String {
    let Some(borders) = format.borders.as_ref().filter(|b| b.any()) else {
        return String::new();
    };
    let mut xml = String::from("<w:pBdr>");
    xml.push_str(&serialize_para_border_edge("w:top", borders.top.as_ref()));
    xml.push_str(&serialize_para_border_edge("w:left", borders.left.as_ref()));
    xml.push_str(&serialize_para_border_edge("w:bottom", borders.bottom.as_ref()));
    xml.push_str(&serialize_para_border_edge("w:right", borders.right.as_ref()));
    xml.push_str("</w:pBdr>");
    xml
}

fn serialize_para_border_edge(tag: &str, spec: Option<&BorderSpec>) -> String {
    let Some(spec) = spec else {
        return String::new();
    };
    let size = (spec.width * BORDER_EIGHTHS_PER_POINT).round().max(1.0) as i32;
    format!(
        r#"<{tag} w:val="single" w:sz="{size}" w:space="0" w:color="{}"/>"#,
        hex_rgb(spec.color)
    )
}

/// The `w:styleId` the style was imported under, so a saved document keeps
/// referring to the style definition already in `styles.xml`.
fn ooxml_style_id(doc: &Document, style_id: StyleId) -> Option<String> {
    if let Some((ooxml_id, _)) = doc
        .styles
        .ooxml_style_ids
        .iter()
        .find(|(_, id)| **id == style_id)
    {
        return Some(ooxml_id.clone());
    }
    doc.styles
        .paragraph_styles
        .get(&style_id)
        .map(|style| style.name.replace(' ', ""))
}

// -- runs ---------------------------------------------------------------------

fn serialize_run(
    run: &Run,
    revision_ids: &mut RevisionIdAllocator,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
) -> String {
    let deleted = matches!(
        run.revision.as_ref().map(|r| r.revision_type),
        Some(tw_model::RevisionType::Delete)
    );

    let content = match &run.content {
        RunContent::OfficeMath { xml } => {
            return match &run.revision {
                Some(rev) => {
                    let deleted = matches!(rev.revision_type, tw_model::RevisionType::Delete);
                    let tag = if deleted { "w:del" } else { "w:ins" };
                    format!(
                        r#"<{tag} w:id="{}" w:author="{}" w:date="{}">{xml}</{tag}>"#,
                        revision_ids.id_for(&rev.id),
                        escape_xml(&rev.author),
                        rev.timestamp.to_rfc3339()
                    )
                }
                None => xml.clone(),
            };
        }
        RunContent::Text(text) => serialize_text(text, deleted),
        RunContent::Tab => "<w:tab/>".to_string(),
        RunContent::Break(tw_model::BreakType::Page) => r#"<w:br w:type="page"/>"#.to_string(),
        RunContent::Break(tw_model::BreakType::Column) => r#"<w:br w:type="column"/>"#.to_string(),
        RunContent::Break(tw_model::BreakType::Line) => "<w:br/>".to_string(),
        RunContent::Hyperlink { text, target } => {
            return serialize_hyperlink_run(run, text, target, deleted, revision_ids, hyperlinks);
        }
        RunContent::Field(field) => {
            let instr = field
                .instruction
                .clone()
                .unwrap_or_else(|| tw_model::field_instruction(&field.field_type));
            let display = field.display_text.as_deref().unwrap_or("");
            let inner = format!(
                "<w:r>{}{}</w:r>",
                serialize_run_properties(&run.format),
                serialize_text(display, deleted)
            );
            return format!(
                r#"<w:fldSimple w:instr="{}">{}</w:fldSimple>"#,
                escape_xml(instr.trim()),
                inner
            );
        }
        RunContent::InlineImage(_) => "<w:t>[image]</w:t>".to_string(),
        RunContent::FootnoteRef(note) => {
            format!(r#"<w:footnoteReference w:id="{}"/>"#, note.note_id)
        }
        RunContent::EndnoteRef(note) => {
            format!(r#"<w:endnoteReference w:id="{}"/>"#, note.note_id)
        }
        RunContent::CitationRef(cite) => {
            let instr = format!(" CITATION {} \\l 1033 ", cite.source_key);
            let display = cite
                .display_text
                .as_deref()
                .unwrap_or("(?)");
            let inner = format!(
                "<w:r>{}{}</w:r>",
                serialize_run_properties(&run.format),
                serialize_text(display, deleted)
            );
            return format!(
                r#"<w:fldSimple w:instr="{}">{}</w:fldSimple>"#,
                escape_xml(instr.trim()),
                inner
            );
        }
        RunContent::CommentRef(c) => {
            format!(r#"<w:commentReference w:id="{}"/>"#, c.comment_id)
        }
        RunContent::Bookmark(b) => {
            let id = b.bookmark_id.unwrap_or(0);
            return format!(
                r#"<w:bookmarkStart w:id="{id}" w:name="{}"/>"#,
                escape_xml(&b.name)
            );
        }
        _ => serialize_text(run.content.display_text(), deleted),
    };

    let xml = format!(
        "<w:r>{}{}</w:r>",
        serialize_run_properties(&run.format),
        content
    );

    // Revision marks wrap the run; they are not run properties.
    match &run.revision {
        Some(rev) => {
            let tag = if deleted { "w:del" } else { "w:ins" };
            format!(
                r#"<{tag} w:id="{}" w:author="{}" w:date="{}">{xml}</{tag}>"#,
                revision_ids.id_for(&rev.id),
                escape_xml(&rev.author),
                rev.timestamp.to_rfc3339()
            )
        }
        None => xml,
    }
}

fn serialize_hyperlink_run(
    run: &Run,
    text: &str,
    target: &tw_model::HyperlinkTarget,
    deleted: bool,
    revision_ids: &mut RevisionIdAllocator,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
) -> String {
    let inner = format!(
        "<w:r>{}{}</w:r>",
        serialize_run_properties(&run.format),
        serialize_text(text, deleted)
    );

    let mut attrs = String::new();
    if let Some(anchor) = crate::hyperlink::hyperlink_anchor(&target.url, &target.anchor) {
        attrs.push_str(&format!(r#" w:anchor="{}""#, escape_xml(&anchor)));
    } else if let Some(rid) = target.url.strip_prefix("r:id:") {
        attrs.push_str(&format!(r#" r:id="{}""#, escape_xml(rid)));
    } else if let Some(rid) = hyperlinks.rid_for(&target.url) {
        attrs.push_str(&format!(r#" r:id="{rid}""#));
    } else if crate::hyperlink::is_external_url(&target.url) {
        // Relationship should have been pre-allocated; fall back to plain text.
        return format!(
            "<w:r>{}{}</w:r>",
            serialize_run_properties(&run.format),
            serialize_text(text, deleted)
        );
    } else {
        return format!(
            "<w:r>{}{}</w:r>",
            serialize_run_properties(&run.format),
            serialize_text(text, deleted)
        );
    }
    if let Some(tooltip) = &target.tooltip {
        if !tooltip.is_empty() {
            attrs.push_str(&format!(r#" w:tooltip="{}""#, escape_xml(tooltip)));
        }
    }

    let xml = format!("<w:hyperlink{attrs}>{inner}</w:hyperlink>");
    match &run.revision {
        Some(rev) => {
            let tag = if deleted { "w:del" } else { "w:ins" };
            format!(
                r#"<{tag} w:id="{}" w:author="{}" w:date="{}">{xml}</{tag}>"#,
                revision_ids.id_for(&rev.id),
                escape_xml(&rev.author),
                rev.timestamp.to_rfc3339()
            )
        }
        None => xml,
    }
}

/// Text content of a run. Tabs and newlines have to become `w:tab` and `w:br`
/// elements: left as literal characters inside `w:t`, Word renders them as
/// ordinary spaces.
fn serialize_text(text: &str, deleted: bool) -> String {
    let tag = if deleted { "w:delText" } else { "w:t" };
    let mut out = String::new();
    let mut pending = String::new();

    for ch in text.chars() {
        let element = match ch {
            '\t' => "<w:tab/>",
            '\n' => "<w:br/>",
            _ => {
                pending.push(ch);
                continue;
            }
        };
        if !pending.is_empty() {
            // `xml:space` keeps leading and trailing spaces, which Word would
            // otherwise collapse.
            out.push_str(&format!(
                r#"<{tag} xml:space="preserve">{}</{tag}>"#,
                escape_xml(&pending)
            ));
            pending.clear();
        }
        out.push_str(element);
    }

    if !pending.is_empty() || out.is_empty() {
        out.push_str(&format!(
            r#"<{tag} xml:space="preserve">{}</{tag}>"#,
            escape_xml(&pending)
        ));
    }
    out
}

struct RevisionIdAllocator {
    next: u32,
    ids: HashMap<NodeId, u32>,
}

impl RevisionIdAllocator {
    fn new() -> Self {
        Self {
            next: 1,
            ids: HashMap::new(),
        }
    }

    fn id_for(&mut self, node_id: &NodeId) -> u32 {
        *self
            .ids
            .entry(*node_id)
            .or_insert_with(|| {
                let id = self.next;
                self.next += 1;
                id
            })
    }
}

fn theme_variant_tint(variant: u8) -> Option<u8> {
    match variant {
        0 => Some(0xCC),
        1 => Some(0x8C),
        2 => Some(0x66),
        _ => None,
    }
}

fn theme_variant_shade(variant: u8) -> Option<u8> {
    match variant {
        4 => Some(0xBF),
        5 => Some(0x80),
        _ => None,
    }
}

fn serialize_run_properties(format: &CharFormat) -> String {
    let mut props = String::new();

    if let Some(family) = &format.font_family {
        let family = escape_xml(family);
        props.push_str(&format!(
            r#"<w:rFonts w:ascii="{family}" w:hAnsi="{family}" w:cs="{family}"/>"#
        ));
    }
    // An explicit `false` has to be written out: it turns off a toggle the
    // paragraph style switched on.
    props.push_str(&toggle("w:b", format.bold));
    props.push_str(&toggle("w:i", format.italic));
    props.push_str(&toggle("w:strike", format.strikethrough));
    props.push_str(&toggle("w:caps", format.all_caps));
    props.push_str(&toggle("w:smallCaps", format.small_caps));
    props.push_str(&toggle("w:vanish", format.hidden));
    if let Some(reference) = format.theme_color {
        let color = format.color.unwrap_or(Color::BLACK);
        let mut attrs = format!(
            r#"w:val="{}" w:themeColor="{}""#,
            hex_rgb(color),
            reference.ooxml_name()
        );
        if let Some(tint) = theme_variant_tint(reference.variant) {
            attrs.push_str(&format!(r#" w:themeTint="{:02X}""#, tint));
        } else if let Some(shade) = theme_variant_shade(reference.variant) {
            attrs.push_str(&format!(r#" w:themeShade="{:02X}""#, shade));
        }
        props.push_str(&format!(r#"<w:color {attrs}/>"#));
    } else if let Some(color) = format.color {
        props.push_str(&format!(r#"<w:color w:val="{}"/>"#, hex_rgb(color)));
    }
    if let Some(size) = format.font_size {
        let half_points = (size * 2.0).round() as i32;
        props.push_str(&format!(
            r#"<w:sz w:val="{half_points}"/><w:szCs w:val="{half_points}"/>"#
        ));
    }
    if let Some(name) = format.highlight.and_then(highlight_name) {
        props.push_str(&format!(r#"<w:highlight w:val="{name}"/>"#));
    }
    if let Some(underline) = format.underline {
        props.push_str(&format!(
            r#"<w:u w:val="{}"/>"#,
            underline_value(underline)
        ));
    }
    if let Some(spacing) = format.character_spacing {
        let twips = crate::export::to_twips(spacing);
        if twips != 0 {
            props.push_str(&format!(r#"<w:spacing w:val="{twips}"/>"#));
        }
    }
    if format.superscript == Some(true) {
        props.push_str(r#"<w:vertAlign w:val="superscript"/>"#);
    } else if format.subscript == Some(true) {
        props.push_str(r#"<w:vertAlign w:val="subscript"/>"#);
    }
    if let Some(language) = &format.language {
        props.push_str(&format!(r#"<w:lang w:val="{}"/>"#, escape_xml(language)));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{props}</w:rPr>")
    }
}

fn toggle(tag: &str, value: Option<bool>) -> String {
    match value {
        Some(true) => format!("<{tag}/>"),
        Some(false) => format!(r#"<{tag} w:val="0"/>"#),
        None => String::new(),
    }
}

// -- tables -------------------------------------------------------------------

fn serialize_table(
    table: &Table,
    doc: &Document,
    package: &DocxPackage,
    media: &mut MediaWriter,
    charts: &crate::chart::ChartWriter,
    diagrams: &crate::diagram::DiagramWriter,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let widths = &table.format.column_widths;
    let mut xml = String::from("<w:tbl><w:tblPr>");

    let width = table
        .format
        .width
        .unwrap_or_else(|| widths.iter().sum::<f32>());
    xml.push_str(&format!(
        r#"<w:tblW w:w="{}" w:type="dxa"/>"#,
        to_twips(width)
    ));
    if let Some(border) = table.format.border {
        xml.push_str(&serialize_table_borders(border));
    }
    xml.push_str("</w:tblPr><w:tblGrid>");
    for column in widths {
        xml.push_str(&format!(r#"<w:gridCol w:w="{}"/>"#, to_twips(*column)));
    }
    xml.push_str("</w:tblGrid>");

    for row in &table.rows {
        xml.push_str(&serialize_table_row(
            row,
            widths,
            doc,
            package,
            media,
            charts,
            diagrams,
            hyperlinks,
            revision_ids,
        ));
    }
    xml.push_str("</w:tbl>");
    xml
}

fn serialize_table_borders(border: BorderSpec) -> String {
    let size = (border.width * BORDER_EIGHTHS_PER_POINT).round().max(1.0) as i32;
    let color = hex_rgb(border.color);
    let edge = |name: &str| {
        format!(r#"<w:{name} w:val="single" w:sz="{size}" w:space="0" w:color="{color}"/>"#)
    };
    format!(
        "<w:tblBorders>{}{}{}{}{}{}</w:tblBorders>",
        edge("top"),
        edge("left"),
        edge("bottom"),
        edge("right"),
        edge("insideH"),
        edge("insideV"),
    )
}

fn serialize_table_row(
    row: &TableRow,
    widths: &[f32],
    doc: &Document,
    package: &DocxPackage,
    media: &mut MediaWriter,
    charts: &crate::chart::ChartWriter,
    diagrams: &crate::diagram::DiagramWriter,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let mut xml = String::from("<w:tr>");
    if let Some(height) = row.height {
        xml.push_str(&format!(
            r#"<w:trPr><w:trHeight w:val="{}"/></w:trPr>"#,
            to_twips(height)
        ));
    }

    let mut column = 0usize;
    for cell in &row.cells {
        let span = cell.format.colspan.max(1) as usize;
        let width: f32 = widths
            .iter()
            .skip(column)
            .take(span)
            .copied()
            .sum::<f32>();
        xml.push_str(&serialize_table_cell(
            cell,
            width,
            doc,
            package,
            media,
            charts,
            diagrams,
            hyperlinks,
            revision_ids,
        ));
        column += span;
    }
    xml.push_str("</w:tr>");
    xml
}

fn serialize_table_cell(
    cell: &TableCell,
    width: f32,
    doc: &Document,
    package: &DocxPackage,
    media: &mut MediaWriter,
    charts: &crate::chart::ChartWriter,
    diagrams: &crate::diagram::DiagramWriter,
    hyperlinks: &crate::hyperlink::HyperlinkRels,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let mut xml = String::from("<w:tc>");
    xml.push_str(&serialize_cell_properties(&cell.format, width));

    let mut has_paragraph = false;
    for block in &cell.blocks {
        has_paragraph |= matches!(block, Block::Paragraph(_));
        xml.push_str(&serialize_block(
            block,
            doc,
            package,
            media,
            charts,
            diagrams,
            hyperlinks,
            revision_ids,
        ));
    }
    // A cell must end with a paragraph or Word treats the file as corrupt.
    if !has_paragraph {
        xml.push_str("<w:p/>");
    }

    xml.push_str("</w:tc>");
    xml
}

fn serialize_cell_properties(format: &CellFormat, width: f32) -> String {
    let mut props = String::new();
    if width > 0.0 {
        props.push_str(&format!(
            r#"<w:tcW w:w="{}" w:type="dxa"/>"#,
            to_twips(width)
        ));
    }
    if format.colspan > 1 {
        props.push_str(&format!(r#"<w:gridSpan w:val="{}"/>"#, format.colspan));
    }
    if format.rowspan > 1 {
        props.push_str(r#"<w:vMerge w:val="restart"/>"#);
    }
    if let Some(border) = format.border {
        let size = (border.width * BORDER_EIGHTHS_PER_POINT).round().max(1.0) as i32;
        let color = hex_rgb(border.color);
        let edge = |name: &str| {
            format!(r#"<w:{name} w:val="single" w:sz="{size}" w:space="0" w:color="{color}"/>"#)
        };
        props.push_str(&format!(
            "<w:tcBorders>{}{}{}{}</w:tcBorders>",
            edge("top"),
            edge("left"),
            edge("bottom"),
            edge("right"),
        ));
    }
    if let Some(background) = format.background {
        props.push_str(&format!(
            r#"<w:shd w:val="clear" w:color="auto" w:fill="{}"/>"#,
            hex_rgb(background)
        ));
    }
    if format.vertical_align != VerticalAlign::Top {
        let value = match format.vertical_align {
            VerticalAlign::Middle => "center",
            VerticalAlign::Bottom => "bottom",
            VerticalAlign::Top => "top",
        };
        props.push_str(&format!(r#"<w:vAlign w:val="{value}"/>"#));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<w:tcPr>{props}</w:tcPr>")
    }
}

// -- images -------------------------------------------------------------------

fn serialize_image_paragraph(image: &ImageBlock, media: &mut MediaWriter) -> String {
    // Without bytes there is nothing to point a relationship at, so the block
    // can only survive as the empty paragraph it occupies.
    let Some(relationship_id) = media.reference(&image.data) else {
        return "<w:p/>".to_string();
    };

    let name = escape_xml(&image.data.asset_id);
    let id = media.next_drawing_id();
    let (frame_w, frame_h) = image.effective_display_size();
    let cx = to_emu(frame_w);
    let cy = to_emu(frame_h);
    let rot = (image.transform.rotation_deg.rem_euclid(360.0) * 60000.0).round() as i64;
    let src_rect = image_src_rect(&image.transform);

    let descr_attr = image
        .alt_text
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!(r#" descr="{}""#, escape_xml(s)))
        .unwrap_or_default();
    let graphic = format!(
        r#"<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="{id}" name="{name}"{descr_attr}/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{relationship_id}"/>{src_rect}<a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm rot="{rot}"><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic>"#
    );
    let doc_pr = format!(
        r#"<wp:docPr id="{id}" name="Picture {id}"{descr_attr}/><wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect="1"/></wp:cNvGraphicFramePr>"#
    );

    let drawing = match image.anchor {
        Some(anchor) => {
            let behind = matches!(image.wrap, TextWrap::Behind);
            format!(
                r#"<wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" relativeHeight="1" behindDoc="{}" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="{}"><wp:posOffset>{}</wp:posOffset></wp:positionH><wp:positionV relativeFrom="{}"><wp:posOffset>{}</wp:posOffset></wp:positionV><wp:extent cx="{cx}" cy="{cy}"/><wp:effectExtent l="0" t="0" r="0" b="0"/>{}{doc_pr}{graphic}</wp:anchor>"#,
                if behind { 1 } else { 0 },
                anchor_origin_value(anchor.origin_x),
                to_emu(anchor.x),
                anchor_origin_value(anchor.origin_y),
                to_emu(anchor.y),
                wrap_element(image.wrap, image.wrap_polygon.as_deref()),
            )
        }
        None => format!(
            r#"<wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:effectExtent l="0" t="0" r="0" b="0"/>{doc_pr}{graphic}</wp:inline>"#
        ),
    };

    format!("<w:p><w:r><w:drawing>{drawing}</w:drawing></w:r></w:p>")
}

fn serialize_shape_paragraph(
    shape: &tw_model::ShapeBlock,
    package: &DocxPackage,
    charts: &crate::chart::ChartWriter,
    diagrams: &crate::diagram::DiagramWriter,
) -> String {
    if let Some(preserved) = package.preserved_shapes.get(&shape.id) {
        if preserved.fingerprint == crate::fingerprint::shape_fingerprint(shape) {
            return preserved.xml.clone();
        }
    }
    let cx = to_emu(shape.shape.width);
    let cy = to_emu(shape.shape.height);
    if shape.shape.shape_type == tw_model::ShapeKind::Diagram {
        let (dm, lo) = diagrams
            .relationships_for(&shape.id)
            .map(|r| (r.data_rel.as_str(), r.layout_rel.as_str()))
            .unwrap_or(("rId1", "rId2"));
        return format!(
            r#"<w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="SmartArt Diagram"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/diagram"><dgm:relIds xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:dm="{dm}" r:lo="{lo}"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
        );
    }
    if shape.shape.shape_type == tw_model::ShapeKind::Chart {
        let rel_id = charts
            .relationship_for(&shape.id)
            .unwrap_or("rId1");
        return format!(
            r#"<w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Chart"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id="{rel_id}"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
        );
    }
    let preset = match shape.shape.shape_type {
        tw_model::ShapeKind::Line => "line",
        tw_model::ShapeKind::Ellipse => "ellipse",
        _ => "rect",
    };
    let tx_body = shape_tx_body(shape);
    let fill = shape
        .style
        .fill
        .map(|c| format!(r#"<a:solidFill><a:srgbClr val="{:06X}"/></a:solidFill>"#, c & 0xFFFFFF))
        .unwrap_or_default();
    let stroke = shape.style.stroke.map(|c| {
        format!(
            r#"<a:ln w="12700"><a:solidFill><a:srgbClr val="{:06X}"/></a:solidFill></a:ln>"#,
            c & 0xFFFFFF
        )
    }).unwrap_or_default();
    format!(
        r#"<w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Shape"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="{preset}"><a:avLst/></a:prstGeom>{fill}{stroke}</wps:spPr>{tx_body}</wps:wsp></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
    )
}

fn shape_tx_body(shape: &tw_model::ShapeBlock) -> String {
    if shape.paragraphs.is_empty() {
        return String::new();
    }
    let mut xml = String::from("<wps:txBody><a:bodyPr/><w:txbxContent>");
    for para in &shape.paragraphs {
        xml.push_str("<w:p><w:r><w:t xml:space=\"preserve\">");
        xml.push_str(&escape_xml(&para.full_text()));
        xml.push_str("</w:t></w:r></w:p>");
    }
    xml.push_str("</w:txbxContent></wps:txBody>");
    xml
}

fn wrap_element(wrap: TextWrap, polygon: Option<&[(f32, f32)]>) -> String {
    let tag = match wrap {
        TextWrap::Square => return r#"<wp:wrapSquare wrapText="bothSides"/>"#.to_string(),
        TextWrap::TopBottom => return "<wp:wrapTopAndBottom/>".to_string(),
        TextWrap::Tight => "wp:wrapTight",
        TextWrap::Through => "wp:wrapThrough",
        TextWrap::Inline | TextWrap::Behind | TextWrap::InFront => {
            return "<wp:wrapNone/>".to_string()
        }
        _ => return "<wp:wrapNone/>".to_string(),
    };
    // Word treats a tight/through wrap without a polygon as its bounding box,
    // so an absent contour still round-trips.
    let Some(points) = polygon.filter(|p| p.len() >= 3) else {
        return format!(r#"<{tag} wrapText="bothSides"/>"#);
    };
    let mut xml = format!(r#"<{tag} wrapText="bothSides"><wp:wrapPolygon edited="0">"#);
    for (index, (x, y)) in points.iter().enumerate() {
        let element = if index == 0 { "wp:start" } else { "wp:lineTo" };
        xml.push_str(&format!(
            r#"<{element} x="{}" y="{}"/>"#,
            to_emu(*x),
            to_emu(*y)
        ));
    }
    // OOXML requires the contour to close back on its start point.
    xml.push_str(&format!(
        r#"<wp:lineTo x="{}" y="{}"/>"#,
        to_emu(points[0].0),
        to_emu(points[0].1)
    ));
    xml.push_str(&format!("</wp:wrapPolygon></{tag}>"));
    xml
}

fn image_src_rect(t: &tw_model::ImageTransform) -> String {
    if t.crop_left <= 0.0 && t.crop_top <= 0.0 && t.crop_right <= 0.0 && t.crop_bottom <= 0.0 {
        return String::new();
    }
    let pct = |v: f32| ((v.clamp(0.0, 0.95) * 100_000.0).round() as i64).to_string();
    format!(
        r#"<a:srcRect l="{}" t="{}" r="{}" b="{}"/>"#,
        pct(t.crop_left),
        pct(t.crop_top),
        pct(t.crop_right),
        pct(t.crop_bottom),
    )
}

fn anchor_origin_value(origin: tw_model::AnchorOrigin) -> &'static str {
    match origin {
        tw_model::AnchorOrigin::Column => "column",
        tw_model::AnchorOrigin::Page => "page",
        tw_model::AnchorOrigin::Margin => "margin",
        tw_model::AnchorOrigin::Paragraph => "paragraph",
    }
}

// -- sections -----------------------------------------------------------------

fn serialize_section_properties(
    section_index: usize,
    section: &tw_model::Section,
    _doc: &Document,
    original: Option<&str>,
) -> String {
    let format = &section.format;
    let mut xml = String::from("<w:sectPr>");

    if let Some(original) = original {
        if section_index == 0 {
            xml.push_str(&copy_children(original, PRESERVED_SECTION_CHILDREN));
        } else {
            xml.push_str(&copy_children(
                original,
                &PRESERVED_SECTION_CHILDREN
                    .iter()
                    .copied()
                    .filter(|tag| *tag != "w:headerReference" && *tag != "w:footerReference")
                    .collect::<Vec<_>>(),
            ));
        }
    }

    for hf_type in [
        tw_model::HeaderFooterType::Default,
        tw_model::HeaderFooterType::First,
        tw_model::HeaderFooterType::Even,
        tw_model::HeaderFooterType::Odd,
    ] {
        if section_index > 0 && section.header_links.is_linked(hf_type) {
            continue;
        }
        if section.headers.get(&hf_type).is_some_and(|hf| !hf.blocks.is_empty()) {
            xml.push_str(&format!(
                r#"<w:headerReference w:type="{}" w:linked="0"/>"#,
                ooxml_hf_type(hf_type)
            ));
        }
        if section_index > 0 && section.footer_links.is_linked(hf_type) {
            continue;
        }
        if section.footers.get(&hf_type).is_some_and(|hf| !hf.blocks.is_empty()) {
            xml.push_str(&format!(
                r#"<w:footerReference w:type="{}" w:linked="0"/>"#,
                ooxml_hf_type(hf_type)
            ));
        }
    }
    xml.push_str(&format!(
        r#"<w:pgSz w:w="{}" w:h="{}"/>"#,
        to_twips(format.page_width),
        to_twips(format.page_height)
    ));
    xml.push_str(&format!(
        r#"<w:pgMar w:top="{}" w:right="{}" w:bottom="{}" w:left="{}" w:header="720" w:footer="720" w:gutter="0"/>"#,
        to_twips(format.margin_top),
        to_twips(format.margin_right),
        to_twips(format.margin_bottom),
        to_twips(format.margin_left)
    ));
    if format.columns.count > 1 {
        xml.push_str(&format!(
            r#"<w:cols w:num="{}" w:space="{}"/>"#,
            format.columns.count,
            to_twips(format.columns.gap)
        ));
    }
    if format.different_first_page {
        xml.push_str("<w:titlePg/>");
    }
    xml.push_str("</w:sectPr>");
    xml
}

fn ooxml_hf_type(kind: tw_model::HeaderFooterType) -> &'static str {
    match kind {
        tw_model::HeaderFooterType::Default => "default",
        tw_model::HeaderFooterType::First => "first",
        tw_model::HeaderFooterType::Even => "even",
        tw_model::HeaderFooterType::Odd => "odd",
        _ => "default",
    }
}

fn patch_settings_xml(pkg: &mut DocxPackage, doc: &Document) {
    let path = "word/settings.xml";
    let xml = pkg
        .parts
        .get(path)
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_else(minimal_settings_xml);
    let patched = set_even_and_odd_headers_in_settings(&xml, doc.settings.even_and_odd_headers);
    if patched != xml {
        pkg.parts.insert(path.into(), patched.into_bytes());
        pkg.mark_modified(path.into());
    }
}

fn minimal_settings_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
</w:settings>"#
        .into()
}

fn set_even_and_odd_headers_in_settings(xml: &str, enabled: bool) -> String {
    let tag = "<w:evenAndOddHeaders";
    if enabled {
        if xml.contains(tag) {
            return xml.to_string();
        }
        if let Some(end) = xml.rfind("</w:settings>") {
            let mut out = xml.to_string();
            out.insert_str(end, "<w:evenAndOddHeaders/>");
            return out;
        }
        return minimal_settings_xml().replace(
            "</w:settings>",
            "<w:evenAndOddHeaders/></w:settings>",
        );
    }

    if !xml.contains(tag) {
        return xml.to_string();
    }
    let mut out = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find(tag) {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        let end = if after.starts_with("<w:evenAndOddHeaders/>") {
            "<w:evenAndOddHeaders/>".len()
        } else if let Some(close) = after.find("</w:evenAndOddHeaders>") {
            close + "</w:evenAndOddHeaders>".len()
        } else if let Some(head_end) = after.find('>') {
            head_end + 1
        } else {
            break;
        };
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

fn original_section_properties(package: &DocxPackage) -> Option<String> {
    let bytes = package.parts.get("word/document.xml")?;
    let xml = String::from_utf8_lossy(bytes);
    let start = xml.find("<w:sectPr")?;
    let end = xml[start..].find("</w:sectPr>")? + start + "</w:sectPr>".len();
    Some(xml[start..end].to_string())
}

/// Copies the named child elements out of an element, start tag to end tag,
/// preserving whatever attributes and content they carried.
fn copy_children(xml: &str, tags: &[&str]) -> String {
    let mut out = String::new();
    for tag in tags {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let mut rest = xml;
        while let Some(start) = rest.find(&open) {
            let after = &rest[start..];
            let Some(head_end) = after.find('>') else {
                break;
            };
            // Distinguish `<w:cols/>` from `<w:cols>...</w:cols>`, and skip
            // tags that only share a name prefix such as `w:type` vs `w:typo`.
            if !matches!(
                after.as_bytes().get(open.len()),
                Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
            ) {
                rest = &after[head_end + 1..];
                continue;
            }
            let end = if after[..head_end].ends_with('/') {
                head_end + 1
            } else {
                match after.find(&close) {
                    Some(i) => i + close.len(),
                    None => break,
                }
            };
            out.push_str(&after[..end]);
            rest = &after[end..];
        }
    }
    out
}

// -- primitives ---------------------------------------------------------------

fn alignment_value(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Left => "left",
        Alignment::Center => "center",
        Alignment::Right => "right",
        Alignment::Justify => "both",
        _ => "left",
    }
}

fn underline_value(style: UnderlineStyle) -> &'static str {
    match style {
        UnderlineStyle::None => "none",
        UnderlineStyle::Single => "single",
        UnderlineStyle::Double => "double",
        UnderlineStyle::Dotted => "dotted",
        UnderlineStyle::Dashed => "dash",
        UnderlineStyle::Wave => "wave",
        _ => "single",
    }
}

/// Word's highlight is a named palette, not a colour; anything outside it
/// cannot be expressed as `w:highlight`.
fn highlight_name(color: Color) -> Option<&'static str> {
    match (color.r, color.g, color.b) {
        (255, 255, 0) => Some("yellow"),
        (0, 255, 0) => Some("green"),
        (0, 255, 255) => Some("cyan"),
        (255, 0, 255) => Some("magenta"),
        (255, 0, 0) => Some("red"),
        (0, 0, 255) => Some("blue"),
        _ => None,
    }
}

fn hex_rgb(color: Color) -> String {
    format!("{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

pub(crate) fn to_twips(points: f32) -> i32 {
    (points * TWIPS_PER_POINT).round() as i32
}

fn to_emu(points: f32) -> i64 {
    (points * EMU_PER_POINT).round() as i64
}

pub(crate) fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Serialize the full paragraph-style catalog to `word/styles.xml`.
pub fn serialize_styles_xml(styles: &StyleSheet) -> String {
    let mut xml = String::from(
        r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    );
    let defaults = serialize_run_properties(&styles.defaults.char_format);
    if !defaults.is_empty() {
        xml.push_str("<w:docDefaults>");
        xml.push_str(&defaults);
        xml.push_str("</w:docDefaults>");
    }
    for style_id in style_export_order(styles) {
        let Some(style) = styles.paragraph_styles.get(&style_id) else {
            continue;
        };
        let ooxml_id = styles
            .ooxml_id_for(style_id)
            .unwrap_or_else(|| style.name.replace(' ', ""));
        let default_attr = if style.name == "Normal" {
            r#" w:default="1""#
        } else {
            ""
        };
        xml.push_str(&format!(
            r#"<w:style w:type="paragraph" w:styleId="{}"{default_attr}>"#,
            escape_xml(&ooxml_id)
        ));
        xml.push_str(&format!(
            r#"<w:name w:val="{}"/>"#,
            escape_xml(&style.name)
        ));
        if let Some(base) = style.based_on.and_then(|id| styles.ooxml_id_for(id)) {
            xml.push_str(&format!(r#"<w:basedOn w:val="{}"/>"#, escape_xml(&base)));
        }
        if let Some(next) = style.next_style.and_then(|id| styles.ooxml_id_for(id)) {
            xml.push_str(&format!(r#"<w:next w:val="{}"/>"#, escape_xml(&next)));
        }
        let para_props = serialize_style_para_properties(&style.para_format);
        if !para_props.is_empty() {
            xml.push_str(&format!("<w:pPr>{para_props}</w:pPr>"));
        }
        let run_props = serialize_run_properties(&style.char_format);
        if !run_props.is_empty() {
            xml.push_str(&run_props);
        }
        xml.push_str("</w:style>");
    }
    xml.push_str("</w:styles>");
    xml
}

fn style_export_order(styles: &StyleSheet) -> Vec<StyleId> {
    let mut order = Vec::new();
    let mut seen = HashMap::new();
    for &id in styles.paragraph_styles.keys() {
        visit_style_order(id, styles, &mut order, &mut seen);
    }
    order
}

fn visit_style_order(
    id: StyleId,
    styles: &StyleSheet,
    order: &mut Vec<StyleId>,
    seen: &mut HashMap<StyleId, ()>,
) {
    if seen.contains_key(&id) {
        return;
    }
    if let Some(style) = styles.paragraph_styles.get(&id) {
        if let Some(base) = style.based_on {
            visit_style_order(base, styles, order, seen);
        }
    }
    seen.insert(id, ());
    order.push(id);
}

fn serialize_style_para_properties(format: &ParaFormat) -> String {
    let mut props = String::new();
    if format.keep_together == Some(true) {
        props.push_str("<w:keepLines/>");
    }
    if format.keep_with_next == Some(true) {
        props.push_str("<w:keepNext/>");
    }
    if let Some(widow) = format.widow_orphan_control {
        if widow {
            props.push_str("<w:widowControl/>");
        } else {
            props.push_str(r#"<w:widowControl w:val="0"/>"#);
        }
    }
    if let Some(level) = format.outline_level {
        if level <= 8 {
            props.push_str(&format!(r#"<w:outlineLvl w:val="{}"/>"#, level));
        }
    }
    props.push_str(&serialize_spacing(format));
    props.push_str(&serialize_indent(format));
    props.push_str(&serialize_tab_stops(format));
    props.push_str(&serialize_para_shading(format));
    props.push_str(&serialize_para_borders(format));
    if let Some(alignment) = format.alignment {
        props.push_str(&format!(
            r#"<w:jc w:val="{}"/>"#,
            alignment_value(alignment)
        ));
    }
    props
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::Document;

    fn xml_for(doc: &Document) -> String {
        let package = DocxPackage::minimal();
        let mut media = MediaWriter::new(&package);
        let charts = crate::chart::ChartWriter::new(&package);
        let diagrams = crate::diagram::DiagramWriter::new(&package);
        let hyperlinks = crate::hyperlink::HyperlinkRels::build(doc, &package);
        serialize_document_xml(doc, &package, &mut media, &charts, &diagrams, &hyperlinks)
    }

    #[test]
    fn serializes_bold_run() {
        let mut doc = Document::with_paragraph("Hello");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.runs[0].format = CharFormat {
                bold: Some(true),
                ..Default::default()
            };
        }
        let xml = xml_for(&doc);
        assert!(xml.contains("<w:b/>"));
        assert!(xml.contains("Hello"));
    }

    #[test]
    fn bold_turned_off_is_written_out() {
        // A run that overrides its style's bold has to say so explicitly.
        let mut doc = Document::with_paragraph("Plain");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.runs[0].format.bold = Some(false);
        }
        assert!(xml_for(&doc).contains(r#"<w:b w:val="0"/>"#));
    }

    #[test]
    fn leading_whitespace_is_preserved() {
        let doc = Document::with_paragraph("  indented by spaces");
        assert!(xml_for(&doc).contains(r#"xml:space="preserve""#));
    }

    #[test]
    fn a_deletion_wraps_the_run_and_uses_del_text() {
        let mut doc = Document::with_paragraph("Gone");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.runs[0].revision = Some(tw_model::Revision::delete("Reviewer"));
        }
        let xml = xml_for(&doc);
        assert!(xml.contains("<w:del "), "expected a wrapping w:del");
        assert!(xml.contains("<w:delText"), "deleted text uses w:delText");
        assert!(
            xml.find("<w:del ").unwrap() < xml.find("<w:r>").unwrap(),
            "w:del must wrap the run, not sit inside w:rPr"
        );
    }

    #[test]
    fn copy_children_takes_whole_elements() {
        let sect = r#"<w:sectPr><w:headerReference r:id="rId7"/><w:cols w:num="2"><w:col/></w:cols><w:pgSz w:w="1"/></w:sectPr>"#;
        let copied = copy_children(sect, &["w:headerReference", "w:cols"]);
        assert!(copied.contains(r#"<w:headerReference r:id="rId7"/>"#));
        assert!(copied.contains("<w:cols w:num=\"2\"><w:col/></w:cols>"));
        assert!(!copied.contains("pgSz"));
    }
}
