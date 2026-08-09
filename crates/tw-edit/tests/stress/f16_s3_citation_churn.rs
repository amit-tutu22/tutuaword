//! Stress: repeated citation insert and bibliography DOCX round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::{BibliographySource, BIBLIOGRAPHY_TITLE, RunContent};

fn tail_insert_pos(session: &EditSession) -> (tw_model::NodeId, usize) {
    let para = session.document.paragraph_at(0, 0).unwrap();
    let run_id = para.runs.last().unwrap().id;
    let offset = tw_edit::run_char_len_by_id(&session.document, run_id);
    (run_id, offset)
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f16_s3_citation_insert_churn() {
    let mut session = EditSession::new();
    for i in 0..20 {
        session
            .apply(Command::AddBibliographySource {
                source: BibliographySource::new(
                    format!("Ref{i}"),
                    format!("Author{i}, A."),
                    format!("Paper {i}"),
                    "2024",
                ),
            })
            .unwrap();
    }

    session
        .apply(Command::InsertText {
            run_id: session.document.paragraph_at(0, 0).unwrap().runs[0].id,
            offset: 0,
            text: "Survey: ".into(),
        })
        .unwrap();

    for i in 0..20 {
        let (run_id, offset) = tail_insert_pos(&session);
        session
            .apply(Command::InsertCitation {
                run_id,
                offset,
                source_key: format!("Ref{i}"),
            })
            .unwrap();
    }

    let cite_count = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter(|run| matches!(run.content, RunContent::CitationRef(_)))
        .count();
    assert_eq!(cite_count, 20);
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f16_s3_bibliography_docx_roundtrip() {
    let mut session = EditSession::new();
    session
        .apply(Command::AddBibliographySource {
            source: BibliographySource::new(
                "Jones2019",
                "Jones, Mary",
                "Survey Methods",
                "2019",
            ),
        })
        .unwrap();
    let run_id = session.document.paragraph_at(0, 0).unwrap().runs[0].id;
    session
        .apply(Command::InsertCitation {
            run_id,
            offset: 0,
            source_key: "Jones2019".into(),
        })
        .unwrap();
    let after = session.document.paragraph_at(0, 0).unwrap().id;
    session
        .apply(Command::InsertBibliography {
            after_block_id: after,
        })
        .unwrap();

    for _ in 0..3 {
        let package = tw_docx::DocxPackage::default();
        let bytes = export(&session.document, &package).expect("export");
        let imported = import(&bytes).expect("import");
        assert_eq!(imported.document.bibliography_sources.len(), 1);
        let titles = imported
            .document
            .sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph())
            .filter(|p| p.full_text().contains(BIBLIOGRAPHY_TITLE))
            .count();
        assert_eq!(titles, 1);
        session.document = imported.document;
    }
}
