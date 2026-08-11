//! F16.S4 — bookmark, cross-reference, and index commands.

use tw_edit::{Command, EditSession};
use tw_model::{
    bookmark_exists, bookmark_names, FieldType, INDEX_TITLE, RunContent,
};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

fn tail_insert_pos(session: &EditSession) -> (tw_model::NodeId, usize) {
    let para = session.document.paragraph_at(0, 0).unwrap();
    let run_id = para.runs.last().unwrap().id;
    let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
    (run_id, offset)
}

fn seed_bookmarked_text(session: &mut EditSession, name: &str, text: &str) {
    let run_id = first_run(session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: text.into(),
        })
        .unwrap();
    let bookmark_run = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertBookmark {
            run_id: bookmark_run,
            offset: 0,
            name: name.into(),
        })
        .unwrap();
}

#[test]
fn u_f16_s4_insert_bookmark_creates_anchor() {
    let mut session = EditSession::new();
    seed_bookmarked_text(&mut session, "SectionRef", "Introduction");

    assert!(bookmark_exists(&session.document, "SectionRef"));
    let para = session.document.paragraph_at(0, 0).unwrap();
    assert!(
        para.runs
            .iter()
            .any(|run| matches!(&run.content, RunContent::Bookmark(b) if b.name == "SectionRef"))
    );
}

#[test]
fn u_f16_s4_cross_ref_resolves_bookmark_text() {
    let mut session = EditSession::new();
    seed_bookmarked_text(&mut session, "SectionRef", "Introduction");

    let (run_id, offset) = tail_insert_pos(&session);
    session
        .apply(Command::InsertCrossReference {
            run_id,
            offset,
            bookmark_name: "SectionRef".into(),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    let field = para
        .runs
        .iter()
        .find(|run| matches!(run.content, RunContent::Field(_)))
        .expect("cross-reference field");
    if let RunContent::Field(field) = &field.content {
        assert_eq!(field.field_type, FieldType::CrossRef);
        assert_eq!(field.display_text.as_deref(), Some("Introduction"));
        assert!(
            field
                .instruction
                .as_deref()
                .is_some_and(|i| i.contains("SectionRef"))
        );
    }
    assert!(para.full_text().contains("Introduction"));
}

#[test]
fn u_f16_s4_cross_ref_requires_bookmark() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    let err = session
        .apply(Command::InsertCrossReference {
            run_id,
            offset: 0,
            bookmark_name: "Missing".into(),
        })
        .unwrap_err();
    assert!(matches!(err, tw_edit::EditError::InvalidRange));
}

#[test]
fn u_f16_s4_insert_index_from_bookmarks() {
    let mut session = EditSession::new();
    seed_bookmarked_text(&mut session, "Alpha", "Apple");
    let after = session.document.paragraph_at(0, 0).unwrap().id;
    session
        .apply(Command::InsertParagraph { after_id: after })
        .unwrap();
    let second_run = session.document.paragraph_at(0, 1).unwrap().runs[0].id;
    session
        .apply(Command::InsertText {
            run_id: second_run,
            offset: 0,
            text: "Banana".into(),
        })
        .unwrap();
    session
        .apply(Command::InsertBookmark {
            run_id: second_run,
            offset: 0,
            name: "Beta".into(),
        })
        .unwrap();

    assert_eq!(bookmark_names(&session.document).len(), 2);

    let tail = session
        .document
        .paragraph_at(0, 1)
        .unwrap()
        .id;
    session
        .apply(Command::InsertIndex {
            after_block_id: tail,
        })
        .unwrap();

    let texts: Vec<String> = session
        .document
        .sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph())
        .map(|p| p.full_text())
        .collect();
    assert!(texts.iter().any(|t| t.contains(INDEX_TITLE)));
    assert!(texts.iter().any(|t| t.contains("Apple")));
    assert!(texts.iter().any(|t| t.contains("Banana")));
}
