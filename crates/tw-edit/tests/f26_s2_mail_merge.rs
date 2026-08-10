//! F26.S2 — InsertMergeField / ApplyMailMergeRow (unit + stress).

use std::collections::BTreeMap;

use tw_edit::{Command, EditSession};
use tw_model::{FieldType, RunContent, parse_mail_merge_csv, generate_mail_merge_documents};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f26_s2_insert_merge_field() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertMergeField {
            run_id,
            offset: 0,
            name: "Name".into(),
        })
        .unwrap();

    let field = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .find_map(|r| match &r.content {
            RunContent::Field(f) if f.field_type == FieldType::MergeField => Some(f),
            _ => None,
        })
        .expect("merge field");
    assert_eq!(field.merge_name.as_deref(), Some("Name"));
    assert_eq!(field.display_text.as_deref(), Some("«Name»"));
}

#[test]
fn u_f26_s2_apply_mail_merge_row_command() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertMergeField {
            run_id,
            offset: 0,
            name: "Name".into(),
        })
        .unwrap();

    let mut values = BTreeMap::new();
    values.insert("Name".into(), "Ada".into());
    session
        .apply(Command::ApplyMailMergeRow { values })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().runs[0].text(),
        "Ada"
    );
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f26_s2_mail_merge_generate_churn() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello ".into(),
        })
        .unwrap();
    let run_id = first_run(&session);
    let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
    session
        .apply(Command::InsertMergeField {
            run_id,
            offset,
            name: "Name".into(),
        })
        .unwrap();

    let mut csv = String::from("Name\n");
    for i in 0..500 {
        csv.push_str(&format!("User{i}\n"));
    }
    let data = parse_mail_merge_csv(&csv).unwrap();
    let docs = generate_mail_merge_documents(&session.document, &data);
    assert_eq!(docs.len(), 500);
    assert!(docs[499]
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .any(|r| r.text().contains("User499")));
}
