//! F16.S3 — insert citation and bibliography commands.

use tw_edit::{Command, EditSession};
use tw_model::{BibliographySource, BIBLIOGRAPHY_TITLE, RunContent};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

fn smith_source() -> BibliographySource {
    BibliographySource::new("Smith2020", "Smith, John", "Example Research", "2020")
}

fn add_smith(session: &mut EditSession) {
    session
        .apply(Command::AddBibliographySource {
            source: smith_source(),
        })
        .unwrap();
}

#[test]
fn u_f16_s3_add_bibliography_source_registers_key() {
    let mut session = EditSession::new();
    add_smith(&mut session);
    assert_eq!(session.document.bibliography_sources.len(), 1);
    assert!(session.document.bibliography_source_by_key("Smith2020").is_some());
}

#[test]
fn u_f16_s3_insert_citation_creates_ref_run() {
    let mut session = EditSession::new();
    add_smith(&mut session);
    let run_id = first_run(&session);

    session
        .apply(Command::InsertCitation {
            run_id,
            offset: 0,
            source_key: "Smith2020".into(),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    let cite = para
        .runs
        .iter()
        .find(|run| matches!(run.content, RunContent::CitationRef(_)))
        .expect("citation ref");
    if let RunContent::CitationRef(cite) = &cite.content {
        assert_eq!(cite.source_key, "Smith2020");
        assert_eq!(cite.display_text.as_deref(), Some("(Smith, 2020)"));
    }
}

#[test]
fn u_f16_s3_insert_citation_requires_source() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    let err = session
        .apply(Command::InsertCitation {
            run_id,
            offset: 0,
            source_key: "Missing".into(),
        })
        .unwrap_err();
    assert!(matches!(err, tw_edit::EditError::InvalidRange));
}

#[test]
fn u_f16_s3_insert_bibliography_from_citations() {
    let mut session = EditSession::new();
    add_smith(&mut session);
    let run_id = first_run(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "See ".into(),
        })
        .unwrap();
    let offset = session.document.paragraph_at(0, 0).unwrap().full_text().len();
    session
        .apply(Command::InsertCitation {
            run_id,
            offset,
            source_key: "Smith2020".into(),
        })
        .unwrap();

    let after = session.document.paragraph_at(0, 0).unwrap().id;
    session
        .apply(Command::InsertBibliography {
            after_block_id: after,
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
    assert!(texts.iter().any(|t| t.contains(BIBLIOGRAPHY_TITLE)));
    assert!(texts.iter().any(|t| t.contains("Example Research")));
    assert!(texts.iter().any(|t| t.contains("2020")));
}
