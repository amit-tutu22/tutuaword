//! U-F14-S1 — OMML (`m:oMath` / `m:oMathPara`) survives import → export, including
//! after nearby paragraph edits (real-use gate).

use std::io::{Cursor, Read, Write};

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::{Block, RunContent};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const OMML_TOKEN: &str = "OMML_TOKEN_f14_s1_preserve";
const OMML_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

const INLINE_MATH_PARAGRAPH: &str = r#"<w:p>
  <w:r><w:t>Before </w:t></w:r>
  <m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
    <m:r><m:t>OMML_TOKEN_f14_s1_preserve</m:t></m:r>
  </m:oMath>
  <w:r><w:t> after</w:t></w:r>
</w:p>"#;

const MATH_PARA_BLOCK: &str = r#"<m:oMathPara xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
  <m:oMath>
    <m:r><m:t>OMML_TOKEN_f14_s1_preserve_block</m:t></m:r>
  </m:oMath>
</m:oMathPara>"#;

const NEIGHBOR_DOC_BODY: &str = r#"<w:p>
  <w:r><w:t>Plain neighbor</w:t></w:r>
</w:p>
<w:p>
  <w:r><w:t>Prefix </w:t></w:r>
  <m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
    <m:r><m:t>OMML_TOKEN_f14_s1_preserve</m:t></m:r>
  </m:oMath>
  <w:r><w:t> suffix</w:t></w:r>
</w:p>"#;

fn minimal_docx(body: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();
        zip.start_file("word/document.xml", opts).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
  xmlns:m="{OMML_NS}">
  <w:body>{body}<w:sectPr/></w:body>
</w:document>"#
        )
        .unwrap();
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
        )
        .unwrap();
        zip.finish().unwrap();
    }
    buf
}

fn exported_document_xml(docx: &[u8]) -> String {
    String::from_utf8(exported_package_part(docx, "word/document.xml")).unwrap()
}

fn exported_package_part(docx: &[u8], name: &str) -> Vec<u8> {
    let cursor = Cursor::new(docx);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();
    let mut file = archive.by_name(name).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    data
}

fn find_office_math<'a>(blocks: &'a [Block]) -> Option<&'a str> {
    for block in blocks {
        if let Block::Paragraph(para) = block {
            for run in &para.runs {
                if let RunContent::OfficeMath { xml } = &run.content {
                    return Some(xml);
                }
            }
        }
    }
    None
}

#[test]
fn u_f14_s1_omml_passthrough() {
    let source = minimal_docx(INLINE_MATH_PARAGRAPH);
    let imported = import(&source).unwrap();

    let xml = find_office_math(&imported.document.sections[0].blocks).expect("OfficeMath run");
    assert!(xml.contains("<m:oMath"), "model should store oMath element: {xml}");
    assert!(xml.contains(OMML_TOKEN), "token missing from model: {xml}");
    assert!(imported.retention.retained_count("oMath") >= 1);

    let exported = export(&imported.document, &imported.package).unwrap();
    let document_xml = exported_document_xml(&exported);

    assert!(
        document_xml.contains("<m:oMath"),
        "oMath was dropped on export: {document_xml}"
    );
    assert!(
        document_xml.contains(OMML_TOKEN),
        "OMML token was lost on export: {document_xml}"
    );
    assert!(
        document_xml.contains(r#"xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math""#),
        "math namespace missing on export: {document_xml}"
    );
}

#[test]
fn u_f14_s1_omath_para_not_dropped() {
    let source = minimal_docx(MATH_PARA_BLOCK);
    let imported = import(&source).unwrap();

    assert!(
        imported.retention.retained_count("oMathPara") >= 1,
        "oMathPara should be retained"
    );

    let block = imported.document.sections[0]
        .blocks
        .first()
        .expect("math para block");
    let para = block.paragraph().expect("paragraph wrapper");
    let RunContent::OfficeMath { xml } = &para.runs[0].content else {
        panic!("expected OfficeMath run, got {:?}", para.runs[0].content);
    };
    assert!(xml.contains("<m:oMathPara"), "expected oMathPara xml: {xml}");
    assert!(xml.contains("OMML_TOKEN_f14_s1_preserve_block"));

    let exported = export(&imported.document, &imported.package).unwrap();
    let document_xml = exported_document_xml(&exported);
    assert!(
        document_xml.contains("<m:oMathPara"),
        "oMathPara dropped on export: {document_xml}"
    );
    assert!(document_xml.contains("OMML_TOKEN_f14_s1_preserve_block"));
}

#[test]
fn u_f14_s1_neighbor_edit_keeps_omml() {
    let source = minimal_docx(NEIGHBOR_DOC_BODY);
    let imported = import(&source).unwrap();

    let mut session = EditSession::from_document(imported.document);
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 14, // end of "Plain neighbor"
            text: " edited".into(),
        })
        .unwrap();

    let exported = export(&session.document, &imported.package).unwrap();
    let document_xml = exported_document_xml(&exported);

    assert!(
        document_xml.contains(OMML_TOKEN),
        "OMML lost after neighbor edit: {document_xml}"
    );
    assert!(
        document_xml.contains("<m:oMath"),
        "oMath element lost after neighbor edit: {document_xml}"
    );
    assert!(
        document_xml.contains("Plain neighbor edited"),
        "neighbor edit should appear: {document_xml}"
    );
}

#[test]
fn u_f14_s1_same_para_text_edit_keeps_omml() {
    let source = minimal_docx(INLINE_MATH_PARAGRAPH);
    let imported = import(&source).unwrap();

    let mut session = EditSession::from_document(imported.document);
    let para = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    let before_run = para
        .runs
        .iter()
        .find(|r| matches!(&r.content, RunContent::Text(t) if t.starts_with("Before")))
        .expect("before text run");
    let run_id = before_run.id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 6, // end of "Before"
            text: "!".into(),
        })
        .unwrap();

    let exported = export(&session.document, &imported.package).unwrap();
    let document_xml = exported_document_xml(&exported);

    assert!(
        document_xml.contains(OMML_TOKEN),
        "OMML lost after same-paragraph edit: {document_xml}"
    );
    assert!(
        document_xml.contains("<m:oMath"),
        "oMath element lost after same-paragraph edit: {document_xml}"
    );
    assert!(
        document_xml.contains("Before!"),
        "edited prefix text missing: {document_xml}"
    );
}

#[test]
fn u_f14_s1_layout_survives_omml_paragraph() {
    use tw_layout::LayoutEngine;

    let source = minimal_docx(INLINE_MATH_PARAGRAPH);
    let imported = import(&source).unwrap();
    let mut engine = LayoutEngine::new();
    let result = engine.layout_document(&imported.document);
    assert!(
        !result.pages.is_empty(),
        "layout should produce at least one page for OMML paragraph"
    );
}
