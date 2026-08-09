//! Stress: repeated symbol InsertText and DOCX export round-trip.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};

const SYMBOLS: &[&str] = &[
    "©", "®", "™", "§", "¶", "€", "£", "¥", "±", "×", "÷", "≠", "∞", "√", "π", "→", "←",
];

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_symbol_insert_churn() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    let mut expected_len = 0usize;

    for i in 0..500 {
        let sym = SYMBOLS[i % SYMBOLS.len()];
        let offset = session.document.paragraph_at(0, 0).unwrap().full_text().chars().count();
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: sym.to_string(),
            })
            .unwrap();
        expected_len += sym.chars().count();
    }

    assert_eq!(
        session
            .document
            .paragraph_at(0, 0)
            .unwrap()
            .full_text()
            .chars()
            .count(),
        expected_len
    );

    session.undo().unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "",
        "coalesced InsertText undo clears the symbol burst"
    );
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_symbol_docx_round_trip() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    let mut text = String::from("Symbols: ");
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: text.clone(),
        })
        .unwrap();

    for sym in SYMBOLS {
        let offset = session.document.paragraph_at(0, 0).unwrap().full_text().chars().count();
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: sym.to_string(),
            })
            .unwrap();
        text.push_str(sym);
    }

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();

    assert_eq!(
        imported.document.paragraph_at(0, 0).unwrap().full_text(),
        text
    );
}
