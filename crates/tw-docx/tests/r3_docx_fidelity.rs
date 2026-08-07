//! R3.3 DOCX fidelity hardening gates.

use std::io::{Cursor, Write};

use tw_docx::{export, export_docx, import, retention};
use tw_model::Block;
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
        zip.write_all(b"<Relationships/>").unwrap();
        for (name, content) in extra_parts {
            zip.start_file(*name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

/// Tier A element types the corpus exercises and the model round-trips.
pub const TIER_A_CORPUS_TAGS: &[&str] = &[
    "p",
    "r",
    "t",
    "pPr",
    "rPr",
    "tbl",
    "tr",
    "tc",
    "b",
    "i",
    "u",
    "sz",
    "color",
    "highlight",
    "jc",
    "spacing",
    "ind",
    "numPr",
    "ilvl",
    "numId",
    "pageBreakBefore",
    "sectPr",
    "pgSz",
    "pgMar",
];

#[test]
fn r3_corpus_round_trip_retains_tier_a_element_counts() {
    let corpus_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let entries: Vec<_> = std::fs::read_dir(&corpus_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "docx"))
        .collect();
    assert!(entries.len() >= 20, "corpus should have at least 20 docx files");

    let mut failures = Vec::new();
    for entry in entries {
        let path = entry.path();
        let bytes = std::fs::read(&path).unwrap();
        let imported = import(&bytes).expect("import");
        let before_xml = String::from_utf8_lossy(
            imported
                .package
                .parts
                .get("word/document.xml")
                .expect("document.xml"),
        );
        let before = retention::element_counts_from_xml(&before_xml);
        let exported = export(&imported.document, &imported.package).expect("export");
        let reimported = import(&exported).expect("reimport");
        let after_xml = String::from_utf8_lossy(
            reimported
                .package
                .parts
                .get("word/document.xml")
                .expect("document.xml"),
        );
        let after = retention::element_counts_from_xml(&after_xml);
        let lost = retention::tier_a_tags_lost(&before, &after, TIER_A_CORPUS_TAGS);
        if !lost.is_empty() {
            failures.push(format!("{}: {}", path.file_name().unwrap().to_string_lossy(), lost.join(", ")));
        }
    }
    assert!(failures.is_empty(), "Tier A retention losses:\n{}", failures.join("\n"));
}

#[test]
fn r3_tier_b_numbering_bytes_preserved_on_body_only_edit() {
    const NUMBERING: &str = r#"<?xml version="1.0"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="9">
    <w:lvl w:ilvl="0"><w:lvlText w:val="%1."/><w:start w:val="1"/></w:lvl>
  </w:abstractNum>
  <w:num w:numId="9"><w:abstractNumId w:val="9"/></w:num>
</w:numbering>"#;
    const DOC: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>Body text</w:t></w:r></w:p></w:body>
</w:document>"#;
    let bytes = minimal_docx(DOC, &[("word/numbering.xml", NUMBERING)]);
    let imported = import(&bytes).unwrap();
    let original_numbering = imported
        .package
        .parts
        .get("word/numbering.xml")
        .cloned()
        .expect("numbering.xml");

    let mut doc = imported.document.clone();
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        *para.runs[0].text_mut().unwrap() = "Edited body".into();
    }
    let mut package = imported.package.clone();
    package.mark_modified("word/document.xml".into());

    let exported = export_docx(&doc, &package).unwrap();
    let reimported = import(&exported).unwrap();
    let numbering_after = reimported
        .package
        .parts
        .get("word/numbering.xml")
        .expect("numbering.xml after round-trip");
    assert_eq!(numbering_after, &original_numbering);
    assert!(
        !reimported.package.modified_parts.contains("word/numbering.xml"),
        "numbering.xml should not be marked modified when catalog unchanged"
    );
}

#[test]
fn r3_within_part_preserves_unmodeled_paragraph_properties() {
    const DOC: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:outlineLvl w:val="2"/></w:pPr>
      <w:r><w:t>Preserved paragraph</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#;
    let bytes = minimal_docx(DOC, &[]);
    let imported = import(&bytes).unwrap();
    let mut package = imported.package.clone();
    package.mark_modified("word/document.xml".into());

    let exported = export_docx(&imported.document, &package).unwrap();
    let reimported = import(&exported).unwrap();
    let doc_xml = String::from_utf8_lossy(
        reimported
            .package
            .parts
            .get("word/document.xml")
            .unwrap(),
    );
    assert!(
        doc_xml.contains("outlineLvl"),
        "unedited paragraph should retain unmodeled w:pPr children via preserved XML"
    );
}

#[test]
fn r3_styles_fingerprint_changes_when_catalog_edited() {
    let imported = import(&minimal_docx(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>x</w:t></w:r></w:p></w:body></w:document>"#,
        &[(
            "word/styles.xml",
            r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:styleId="Custom"><w:name w:val="Custom"/></w:style></w:styles>"#,
        )],
    ))
    .unwrap();
    let before = imported.package.source_styles_fingerprint;
    let mut doc = imported.document.clone();
    doc.styles
        .paragraph_styles
        .values_mut()
        .next()
        .expect("style")
        .name = "Renamed".into();
    let after = tw_docx::fingerprint::styles_fingerprint(&doc.styles);
    assert_ne!(before, Some(after));
}
