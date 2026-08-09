//! F16.S3 — citation + bibliography.xml DOCX round-trip.

use tw_docx::{export, import, BIBLIOGRAPHY_PART};
use tw_edit::{Command, EditSession};
use tw_model::{BibliographySource, RunContent};

fn doc_with_citation() -> EditSession {
    let mut session = EditSession::new();
    session
        .apply(Command::AddBibliographySource {
            source: BibliographySource::new(
                "Smith2020",
                "Smith, John",
                "Example Research",
                "2020",
            ),
        })
        .unwrap();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Prior work ".into(),
        })
        .unwrap();
    let offset = session.document.paragraph_at(0, 0).unwrap().full_text().len();
    session
        .apply(Command::InsertCitation {
            run_id,
            offset,
            source_key: "Smith2020".into(),
        })
        .unwrap();
    session
}

#[test]
fn u_f16_s3_bibliography_xml_roundtrip() {
    let session = doc_with_citation();
    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");
    let imported = import(&bytes).expect("import");

    assert!(imported
        .package
        .parts
        .contains_key(BIBLIOGRAPHY_PART));
    assert_eq!(imported.document.bibliography_sources.len(), 1);
    assert_eq!(imported.document.bibliography_sources[0].key, "Smith2020");

    let has_cite = imported
        .document
        .sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph())
        .flat_map(|p| &p.runs)
        .any(|run| matches!(run.content, RunContent::CitationRef(_)));
    assert!(has_cite, "citation ref missing after round-trip");
}
