//! R2.1 — document model vocabulary import + retention counts.

use std::collections::HashSet;
use std::io::{Cursor, Write};

use tw_docx::{import_docx, retention::unmapped_corpus_tags, ImportRetentionReport};
use tw_model::{Block, FieldType, HeaderFooterType, RunContent};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn minimal_docx(document_xml: &str, extra_parts: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", options).unwrap();
        zip.write_all(
            br#"<Relationships>
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer2.xml"/>
</Relationships>"#,
        )
        .unwrap();
        for (name, content) in extra_parts {
            zip.start_file(*name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

const VOCABULARY_DOC: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p>
      <w:bookmarkStart w:id="0" w:name="intro"/>
      <w:hyperlink r:id="rId9">
        <w:r><w:t>Visit site</w:t></w:r>
      </w:hyperlink>
      <w:r><w:footnoteReference w:id="1"/></w:r>
      <w:r><w:commentReference w:id="2"/></w:r>
      <w:fldSimple w:instr=" PAGE ">
        <w:r><w:t>1</w:t></w:r>
      </w:fldSimple>
    </w:p>
    <w:p>
      <w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
        <wp:extent cx="914400" cy="457200"/>
        <a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
          <a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
            <wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"/>
          </a:graphicData>
        </a:graphic>
      </wp:inline></w:drawing></w:r>
    </w:p>
    <w:sectPr>
      <w:headerReference w:type="default" r:id="rId1"/>
      <w:headerReference w:type="even" r:id="rId1"/>
      <w:footerReference w:type="default" r:id="rId2"/>
      <w:footerReference w:type="odd" r:id="rId3"/>
    </w:sectPr>
  </w:body>
</w:document>"#;

const DEFAULT_HEADER: &str = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p><w:r><w:t>Default header</w:t></w:r></w:p>
</w:hdr>"#;

const DEFAULT_FOOTER: &str = r#"<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p><w:r><w:t>Default footer</w:t></w:r></w:p>
</w:ftr>"#;

const ODD_FOOTER: &str = r#"<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:p><w:r><w:t>Odd footer</w:t></w:r></w:p>
</w:ftr>"#;

fn run_content_variants(doc: &tw_model::Document) -> HashSet<&'static str> {
    let mut found = HashSet::new();
    for para in doc
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
    {
        for run in &para.runs {
            let tag = match &run.content {
                RunContent::Hyperlink { .. } => "hyperlink",
                RunContent::Field(_) => "fldSimple",
                RunContent::FootnoteRef(_) => "footnoteReference",
                RunContent::CommentRef(_) => "commentReference",
                RunContent::Bookmark(_) => "bookmarkStart",
                RunContent::InlineImage(_) => "drawing",
                _ => continue,
            };
            found.insert(tag);
        }
    }
    for section in &doc.sections {
        if section.headers.contains_key(&HeaderFooterType::Even) {
            found.insert("headerReference");
        }
        if section.footers.contains_key(&HeaderFooterType::Odd) {
            found.insert("footerReference");
        }
    }
    if doc.sections[0]
        .blocks
        .iter()
        .any(|b| matches!(b, Block::ShapeBlock(_)))
    {
        found.insert("drawing");
    }
    found
}

#[test]
fn r2_vocabulary_import_retains_model_variants() {
    let bytes = minimal_docx(
        VOCABULARY_DOC,
        &[
            ("word/header1.xml", DEFAULT_HEADER),
            ("word/footer1.xml", DEFAULT_FOOTER),
            ("word/footer2.xml", ODD_FOOTER),
        ],
    );
    let result = import_docx(&bytes).unwrap();
    let doc = &result.document;
    let report = &result.retention;

    assert!(report.retained_count("hyperlink") > 0, "{}", report.summary());
    assert!(report.retained_count("fldSimple") > 0, "{}", report.summary());
    assert!(report.retained_count("footnoteReference") > 0, "{}", report.summary());
    assert!(report.retained_count("commentReference") > 0, "{}", report.summary());
    assert!(report.retained_count("bookmarkStart") > 0, "{}", report.summary());
    assert!(report.retained_count("headerReference") >= 2, "{}", report.summary());
    assert!(report.retained_count("footerReference") >= 2, "{}", report.summary());

    let section = &doc.sections[0];
    assert!(section.headers.contains_key(&HeaderFooterType::Default));
    assert!(section.headers.contains_key(&HeaderFooterType::Even));
    assert!(section.footers.contains_key(&HeaderFooterType::Default));
    assert!(section.footers.contains_key(&HeaderFooterType::Odd));

    let para = doc.sections[0].blocks[0].paragraph().unwrap();
    assert!(para
        .runs
        .iter()
        .any(|r| matches!(r.content, RunContent::Hyperlink { .. })));
    assert!(para.runs.iter().any(|r| matches!(
        &r.content,
        RunContent::Field(f) if matches!(f.field_type, FieldType::Page)
    )));
    assert!(para
        .runs
        .iter()
        .any(|r| matches!(r.content, RunContent::FootnoteRef(_))));
    assert!(para
        .runs
        .iter()
        .any(|r| matches!(r.content, RunContent::CommentRef(_))));
    assert!(para
        .runs
        .iter()
        .any(|r| matches!(r.content, RunContent::Bookmark(_))));

    assert!(doc.sections[0]
        .blocks
        .iter()
        .any(|b| matches!(b, Block::ShapeBlock(_))));

    let variants = run_content_variants(doc);
    for expected in [
        "hyperlink",
        "fldSimple",
        "footnoteReference",
        "commentReference",
        "bookmarkStart",
        "drawing",
        "headerReference",
        "footerReference",
    ] {
        assert!(variants.contains(expected), "missing variant {expected}");
    }
}

/// Lists OOXML element types encountered but not yet retained in the model.
#[test]
fn r2_corpus_reports_unmapped_ooxml_types() {
    let bytes = minimal_docx(
        VOCABULARY_DOC,
        &[
            ("word/header1.xml", DEFAULT_HEADER),
            ("word/footer1.xml", DEFAULT_FOOTER),
            ("word/footer2.xml", ODD_FOOTER),
        ],
    );
    let result = import_docx(&bytes).unwrap();
    let report = &result.retention;

    const R2_MAPPED: &[&str] = &[
        "p",
        "r",
        "t",
        "pPr",
        "rPr",
        "body",
        "document",
        "sectPr",
        "hyperlink",
        "fldSimple",
        "footnoteReference",
        "commentReference",
        "bookmarkStart",
        "drawing",
        "headerReference",
        "footerReference",
        "hdr",
        "ftr",
    ];

    let unmapped = unmapped_corpus_tags(report.encountered(), report.retained(), R2_MAPPED);
    if !unmapped.is_empty() {
        let mut lines = vec!["Unmapped OOXML types (R2.1 gap report):".to_string()];
        for (tag, count) in &unmapped {
            lines.push(format!("  w:{tag}: {count} encountered, not retained"));
        }
        lines.push(report.summary());
        panic!("{}", lines.join("\n"));
    }
}

#[test]
fn r2_retention_report_format() {
    let mut report = ImportRetentionReport::new();
    report.record_encountered("hyperlink");
    report.record_retained("hyperlink");
    let summary = report.summary();
    assert!(summary.contains("Import retention report"));
    assert!(summary.contains("hyperlink: encountered=1, retained=1"));
}
