//! Unit tests for revision list JSON (F17.S2).

use tw_model::{revision_entries, Block, Document, Paragraph, Revision, RevisionType, Run, RunContent};

#[test]
fn u_f17_s2_revision_entries_json() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("");
    let mut run = Run::new_text("Added");
    run.revision = Some(Revision {
        id: run.id,
        revision_type: RevisionType::Insert,
        author: "Author".into(),
        timestamp: chrono::Utc::now(),
    });
    para.runs = vec![run.clone()];
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let entries = revision_entries(&doc);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].revision_type, "insert");
    assert_eq!(entries[0].author, "Author");
    assert!(entries[0].preview.contains("Added"));
    assert_eq!(entries[0].run_id, run.id.to_string());
}
