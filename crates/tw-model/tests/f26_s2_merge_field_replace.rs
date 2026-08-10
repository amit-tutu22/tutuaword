//! F26.S2 — merge field replace (`«Name»` → row value).

use std::collections::BTreeMap;

use tw_model::{
    apply_mail_merge_row, evaluate_field, generate_mail_merge_documents, merge_field_data,
    parse_mail_merge_csv, Document, FieldEvalContext, FieldType, Run, RunContent,
};

fn field_run(name: &str) -> Run {
    Run {
        id: tw_model::NodeId::new(),
        format: Default::default(),
        content: RunContent::Field(merge_field_data(name)),
        revision: None,
    }
}

#[test]
fn u_f26_s2_merge_field_replace() {
    let mut doc = Document::new();
    let para = doc.paragraph_at_mut(0, 0).unwrap();
    para.runs.clear();
    para.runs.push(field_run("Name"));

    let mut row = BTreeMap::new();
    row.insert("Name".into(), "Ada".into());
    apply_mail_merge_row(&mut doc, &row);

    let text = match &doc.paragraph_at(0, 0).unwrap().runs[0].content {
        RunContent::Text(t) => t.as_str(),
        other => panic!("expected text after merge, got {other:?}"),
    };
    assert_eq!(text, "Ada");
}

#[test]
fn u_f26_s2_merge_field_unbound_placeholder() {
    let field = merge_field_data("Name");
    assert_eq!(field.field_type, FieldType::MergeField);
    assert_eq!(
        evaluate_field(&field, &FieldEvalContext::for_page(0, 1)),
        "«Name»"
    );
}

#[test]
fn u_f26_s2_guillemet_text_replace() {
    let mut doc = Document::new();
    let para = doc.paragraph_at_mut(0, 0).unwrap();
    para.runs.clear();
    para.runs
        .push(Run::new_text("Dear «Name», welcome to «City»."));

    let mut row = BTreeMap::new();
    row.insert("Name".into(), "Grace".into());
    row.insert("City".into(), "London".into());
    apply_mail_merge_row(&mut doc, &row);

    assert_eq!(
        doc.paragraph_at(0, 0).unwrap().runs[0].text(),
        "Dear Grace, welcome to London."
    );
}

#[test]
fn u_f26_s2_generate_documents_from_csv() {
    let mut template = Document::new();
    {
        let para = template.paragraph_at_mut(0, 0).unwrap();
        para.runs.clear();
        para.runs.push(field_run("Name"));
    }
    let csv = "Name,City\nAda,Paris\nGrace,London\n";
    let data = parse_mail_merge_csv(csv).unwrap();
    let docs = generate_mail_merge_documents(&template, &data);
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].paragraph_at(0, 0).unwrap().runs[0].text(), "Ada");
    assert_eq!(docs[1].paragraph_at(0, 0).unwrap().runs[0].text(), "Grace");
}
