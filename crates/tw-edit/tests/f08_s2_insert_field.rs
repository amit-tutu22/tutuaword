//! F08.S2 — insert PAGE/DATE field commands.

use tw_edit::{Command, EditSession};
use tw_model::{FieldType, RunContent};

#[test]
fn u_f08_s2_insert_page_field_replaces_empty_run() {
    let mut session = EditSession::from_document(tw_model::Document::new());
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;

    session
        .apply(Command::InsertField {
            run_id,
            offset: 0,
            field_type: FieldType::Page,
        })
        .unwrap();

    let run = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .clone();
    assert!(matches!(run.content, RunContent::Field(_)));
    if let RunContent::Field(field) = run.content {
        assert_eq!(field.field_type, FieldType::Page);
    }
}

#[test]
fn u_f08_s2_insert_date_field() {
    let mut session = EditSession::from_document(tw_model::Document::new());
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;

    session
        .apply(Command::InsertField {
            run_id,
            offset: 0,
            field_type: FieldType::Date,
        })
        .unwrap();

    let run = &session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0];
    assert!(matches!(&run.content, RunContent::Field(f) if f.field_type == FieldType::Date));
}
