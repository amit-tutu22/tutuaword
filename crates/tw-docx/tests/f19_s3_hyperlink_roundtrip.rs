//! F19.S3 — hyperlink DOCX export/import round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::RunContent;

fn doc_with_hyperlink() -> EditSession {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertHyperlink {
            run_id,
            offset: 0,
            url: "https://example.com/docs".into(),
            text: "Example Docs".into(),
            tooltip: Some("Open docs".into()),
        })
        .unwrap();
    session
}

#[test]
fn u_f19_s3_hyperlink_roundtrip_docx() {
    let session = doc_with_hyperlink();
    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");

    let document_xml = package_part(&bytes, "word/document.xml").expect("document.xml");
    let xml = String::from_utf8_lossy(&document_xml);
    assert!(
        xml.contains("<w:hyperlink") && xml.contains("r:id="),
        "export must emit w:hyperlink with relationship id"
    );
    assert!(
        xml.contains("Example Docs"),
        "display text missing from export: {xml}"
    );

    let rels_xml = package_part(&bytes, "word/_rels/document.xml.rels").expect("rels");
    let rels = String::from_utf8_lossy(&rels_xml);
    assert!(
        rels.contains("https://example.com/docs")
            && rels.contains("TargetMode=\"External\""),
        "hyperlink relationship missing: {rels}"
    );

    let imported = import(&bytes).expect("import");
    let link = imported
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .find_map(|block| {
            block.paragraph().and_then(|para| {
                para.runs.iter().find_map(|run| match &run.content {
                    RunContent::Hyperlink { target, text } => Some((target.clone(), text.clone())),
                    _ => None,
                })
            })
        });
    let (target, text) = link.expect("hyperlink missing after round-trip");
    assert_eq!(text, "Example Docs");
    assert_eq!(target.url, "https://example.com/docs");
    assert_eq!(target.tooltip.as_deref(), Some("Open docs"));
}

#[test]
fn u_f19_s3_internal_anchor_hyperlink_roundtrip() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Heading".into(),
        })
        .unwrap();
    let bookmark_run = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertBookmark {
            run_id: bookmark_run,
            offset: 0,
            name: "Top".into(),
        })
        .unwrap();
    let (run_id, offset) = {
        let para = session.document.paragraph_at(0, 0).unwrap();
        let run_id = para.runs.last().unwrap().id;
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        (run_id, offset)
    };
    session
        .apply(Command::InsertHyperlink {
            run_id,
            offset,
            url: "#Top".into(),
            text: "Go to top".into(),
            tooltip: None,
        })
        .unwrap();

    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");
    let document_xml = package_part(&bytes, "word/document.xml").expect("document.xml");
    let xml = String::from_utf8_lossy(&document_xml);
    assert!(
        xml.contains(r#"w:anchor="Top""#),
        "internal hyperlink must use w:anchor: {xml}"
    );

    let imported = import(&bytes).expect("import");
    let found = imported.document.sections.iter().any(|section| {
        section.blocks.iter().any(|block| {
            block.paragraph().is_some_and(|para| {
                para.runs.iter().any(|run| {
                    matches!(
                        &run.content,
                        RunContent::Hyperlink { target, text }
                            if text == "Go to top"
                                && (target.anchor.as_deref() == Some("Top")
                                    || target.url == "#Top")
                    )
                })
            })
        })
    });
    assert!(found, "internal hyperlink missing after round-trip");
}

fn package_part(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    use std::io::{Cursor, Read};
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).ok()?;
    Some(data)
}
