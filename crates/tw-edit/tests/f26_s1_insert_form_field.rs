//! F26.S1 — InsertFormField / SetFormFieldValue (unit + stress).

use tw_edit::{Command, EditSession};
use tw_model::{FieldType, FormFieldKind, RunContent, evaluate_field, FieldEvalContext};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

fn find_form_field(session: &EditSession) -> Option<&tw_model::FieldData> {
    session
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .find_map(|block| {
            block.paragraph().and_then(|para| {
                para.runs.iter().find_map(|run| match &run.content {
                    RunContent::Field(f)
                        if matches!(
                            f.field_type,
                            FieldType::FormText | FieldType::FormCheckbox
                        ) =>
                    {
                        Some(f)
                    }
                    _ => None,
                })
            })
        })
}

fn form_field_run_id(session: &EditSession) -> Option<tw_model::NodeId> {
    session
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .find_map(|block| {
            block.paragraph().and_then(|para| {
                para.runs.iter().find_map(|run| match &run.content {
                    RunContent::Field(f)
                        if matches!(
                            f.field_type,
                            FieldType::FormText | FieldType::FormCheckbox
                        ) =>
                    {
                        Some(run.id)
                    }
                    _ => None,
                })
            })
        })
}

#[test]
fn u_f26_s1_insert_form_text() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertFormField {
            run_id,
            offset: 0,
            kind: FormFieldKind::PlainText,
            name: Some("FullName".into()),
            initial_value: Some("Ada".into()),
        })
        .unwrap();

    let field = find_form_field(&session).expect("form text field");
    assert_eq!(field.field_type, FieldType::FormText);
    assert_eq!(field.display_text.as_deref(), Some("Ada"));
    assert_eq!(
        field.form.as_ref().and_then(|f| f.name.as_deref()),
        Some("FullName")
    );
    assert_eq!(
        evaluate_field(field, &FieldEvalContext::for_page(0, 1)),
        "Ada"
    );
}

#[test]
fn u_f26_s1_insert_form_checkbox() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertFormField {
            run_id,
            offset: 0,
            kind: FormFieldKind::Checkbox,
            name: Some("Agree".into()),
            initial_value: Some("true".into()),
        })
        .unwrap();

    let field = find_form_field(&session).expect("checkbox field");
    assert_eq!(field.field_type, FieldType::FormCheckbox);
    assert_eq!(field.form.as_ref().and_then(|f| f.checked), Some(true));
    assert_eq!(
        evaluate_field(field, &FieldEvalContext::for_page(0, 1)),
        "☑"
    );
}

#[test]
fn u_f26_s1_set_form_text_value() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertFormField {
            run_id,
            offset: 0,
            kind: FormFieldKind::PlainText,
            name: None,
            initial_value: Some("old".into()),
        })
        .unwrap();
    let field_id = form_field_run_id(&session).unwrap();
    session
        .apply(Command::SetFormFieldValue {
            run_id: field_id,
            value: "new".into(),
        })
        .unwrap();

    let field = find_form_field(&session).unwrap();
    assert_eq!(field.display_text.as_deref(), Some("new"));
    assert_eq!(
        field.form.as_ref().and_then(|f| f.default_text.as_deref()),
        Some("new")
    );
}

#[test]
fn u_f26_s1_toggle_form_checkbox() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    session
        .apply(Command::InsertFormField {
            run_id,
            offset: 0,
            kind: FormFieldKind::Checkbox,
            name: None,
            initial_value: Some("false".into()),
        })
        .unwrap();
    let field_id = form_field_run_id(&session).unwrap();
    session
        .apply(Command::SetFormFieldValue {
            run_id: field_id,
            value: "toggle".into(),
        })
        .unwrap();

    let field = find_form_field(&session).unwrap();
    assert_eq!(field.form.as_ref().and_then(|f| f.checked), Some(true));
    assert_eq!(field.display_text.as_deref(), Some("☑"));
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f26_s1_form_field_insert_churn() {
    let mut session = EditSession::new();
    for i in 0..200 {
        let para = session.document.paragraph_at(0, 0).unwrap();
        let run_id = para.runs.last().unwrap().id;
        let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
        let kind = if i % 2 == 0 {
            FormFieldKind::PlainText
        } else {
            FormFieldKind::Checkbox
        };
        session
            .apply(Command::InsertFormField {
                run_id,
                offset,
                kind,
                name: Some(format!("f{i}")),
                initial_value: Some(if matches!(kind, FormFieldKind::Checkbox) {
                    (i % 4 == 1).to_string()
                } else {
                    format!("v{i}")
                }),
            })
            .unwrap();
    }
    let count = session
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .flat_map(|b| b.paragraph().into_iter())
        .flat_map(|p| p.runs.iter())
        .filter(|r| {
            matches!(
                &r.content,
                RunContent::Field(f)
                    if matches!(f.field_type, FieldType::FormText | FieldType::FormCheckbox)
            )
        })
        .count();
    assert_eq!(count, 200);
}
