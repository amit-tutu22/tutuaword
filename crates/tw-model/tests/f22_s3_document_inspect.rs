//! F22.S3 — Document Inspector findings.

use tw_model::{
    inspect_document, Block, CharFormat, CommentThread, Document, InspectCategory, Paragraph, Run,
    RunContent,
};

#[test]
fn u_f22_s3_inspect_comments_metadata_hidden() {
    let mut doc = Document::with_paragraph("Visible ");
    doc.properties.title = Some("Report".into());
    doc.properties.author = Some("Ada".into());

    let mut hidden = Run::new_text("secret");
    hidden.format = CharFormat {
        hidden: Some(true),
        ..Default::default()
    };
    doc.sections[0].blocks[0]
        .paragraph_mut()
        .unwrap()
        .runs
        .push(hidden);

    let run_id = doc.paragraph_at(0, 0).unwrap().runs[0].id;
    doc.comments.push(CommentThread::new(0, run_id, 0, "Ada", "note"));
    let mut comment_run = Run::new_text("");
    comment_run.content = RunContent::CommentRef(tw_model::CommentRef {
        comment_id: 0,
        display_number: Some(1),
    });
    doc.sections[0].blocks[0]
        .paragraph_mut()
        .unwrap()
        .runs
        .push(comment_run);

    let findings = inspect_document(&doc);
    assert!(findings
        .iter()
        .any(|f| f.category == InspectCategory::Comments && f.count >= 1));
    assert!(findings
        .iter()
        .any(|f| f.category == InspectCategory::Metadata && f.count == 2));
    assert!(findings
        .iter()
        .any(|f| f.category == InspectCategory::HiddenText && f.count == 1));
}

#[test]
fn u_f22_s3_inspect_clean_document_empty() {
    let doc = Document::with_paragraph("Hello");
    assert!(inspect_document(&doc).is_empty());
}

#[test]
fn u_f22_s3_inspect_table_hidden() {
    let mut doc = Document::new();
    let mut table = tw_model::Table::new(1, 1);
    let mut para = Paragraph::with_text("cell");
    let mut hidden = Run::new_text("hid");
    hidden.format.hidden = Some(true);
    para.runs.push(hidden);
    table.rows[0].cells[0].blocks = vec![Block::Paragraph(para)];
    doc.sections[0].blocks = vec![Block::Table(table)];

    let findings = inspect_document(&doc);
    assert!(findings
        .iter()
        .any(|f| f.category == InspectCategory::HiddenText && f.count == 1));
}
