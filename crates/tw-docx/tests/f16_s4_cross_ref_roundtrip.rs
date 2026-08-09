//! F16.S4 — bookmark and REF field DOCX export/import.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::{FieldType, RunContent};

fn doc_with_cross_ref() -> EditSession {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Introduction".into(),
        })
        .unwrap();
    let bookmark_run = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertBookmark {
            run_id: bookmark_run,
            offset: 0,
            name: "SectionRef".into(),
        })
        .unwrap();
    let (run_id, offset) = {
        let para = session.document.paragraph_at(0, 0).unwrap();
        let run_id = para.runs.last().unwrap().id;
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        (run_id, offset)
    };
    session
        .apply(Command::InsertCrossReference {
            run_id,
            offset,
            bookmark_name: "SectionRef".into(),
        })
        .unwrap();
    session
}

#[test]
fn u_f16_s4_cross_ref_docx_roundtrip() {
    let session = doc_with_cross_ref();
    let package = tw_docx::DocxPackage::default();
    let bytes = export(&session.document, &package).expect("export");

    let imported = import(&bytes).expect("import");
    let has_bookmark = imported.document.sections.iter().any(|section| {
        section.blocks.iter().any(|block| {
            block.paragraph().is_some_and(|para| {
                para.runs.iter().any(|run| {
                    matches!(
                        &run.content,
                        RunContent::Bookmark(b) if b.name == "SectionRef"
                    )
                })
            })
        })
    });
    assert!(has_bookmark, "bookmark missing after round-trip");

    let has_ref = imported.document.sections.iter().any(|section| {
        section.blocks.iter().any(|block| {
            block.paragraph().is_some_and(|para| {
                para.runs.iter().any(|run| {
                    matches!(
                        &run.content,
                        RunContent::Field(f) if f.field_type == FieldType::CrossRef
                    )
                })
            })
        })
    });
    assert!(has_ref, "REF field missing after round-trip");

    let text = imported
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .full_text();
    assert!(text.contains("Introduction"), "display text: {text}");
}
