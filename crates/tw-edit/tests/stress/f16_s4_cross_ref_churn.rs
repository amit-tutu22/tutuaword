//! Stress: repeated bookmark/cross-reference insert and DOCX round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::{FieldType, INDEX_TITLE, RunContent};

fn tail_insert_pos(session: &EditSession) -> (tw_model::NodeId, usize) {
    let para = session.document.paragraph_at(0, 0).unwrap();
    let run_id = para.runs.last().unwrap().id;
    let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
    (run_id, offset)
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f16_s4_cross_ref_insert_churn() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Topics: ".into(),
        })
        .unwrap();

    for i in 0..20 {
        let (run_id, offset) = tail_insert_pos(&session);
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: format!("Term{i} "),
            })
            .unwrap();
        let (run_id, offset) = tail_insert_pos(&session);
        let name = format!("Term{i}");
        let text_start = offset.saturating_sub(name.len() + 1);
        session
            .apply(Command::InsertBookmark {
                run_id,
                offset: text_start,
                name: name.clone(),
            })
            .unwrap();
        let (run_id, offset) = tail_insert_pos(&session);
        session
            .apply(Command::InsertCrossReference {
                run_id,
                offset,
                bookmark_name: name,
            })
            .unwrap();
    }

    let bookmark_count = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter(|run| matches!(run.content, RunContent::Bookmark(_)))
        .count();
    assert_eq!(bookmark_count, 20);

    let ref_count = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter(|run| matches!(run.content, RunContent::Field(_)))
        .count();
    assert_eq!(ref_count, 20);
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f16_s4_cross_ref_docx_roundtrip() {
    let mut session = EditSession::new();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Alpha".into(),
        })
        .unwrap();
    session
        .apply(Command::InsertBookmark {
            run_id,
            offset: 0,
            name: "AlphaRef".into(),
        })
        .unwrap();
    session
        .apply(Command::InsertCrossReference {
            run_id,
            offset: 0,
            bookmark_name: "AlphaRef".into(),
        })
        .unwrap();
    let after = session.document.paragraph_at(0, 0).unwrap().id;
    session
        .apply(Command::InsertIndex {
            after_block_id: after,
        })
        .unwrap();

    for _ in 0..3 {
        let package = tw_docx::DocxPackage::default();
        let bytes = export(&session.document, &package).expect("export");
        let imported = import(&bytes).expect("import");
        let has_index = imported
            .document
            .sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph())
            .any(|p| p.full_text().contains(INDEX_TITLE));
        assert!(has_index);
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
        assert!(has_ref);
        session.document = imported.document;
    }
}
