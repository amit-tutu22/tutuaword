//! F16.S1 — footnote DOCX export/import round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::RunContent;

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f16_s1_footnote_docx_round_trip() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Claim".into(),
        })
        .unwrap();
    let offset = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .full_text()
        .chars()
        .count();
    session
        .apply(Command::InsertFootnote {
            run_id,
            offset,
        })
        .unwrap();

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();

    assert_eq!(imported.document.footnotes.len(), 1);
    assert!(
        imported
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .runs
            .iter()
            .any(|run| matches!(run.content, RunContent::FootnoteRef(_)))
    );
    assert!(imported.package.parts.contains_key("word/footnotes.xml"));
}
