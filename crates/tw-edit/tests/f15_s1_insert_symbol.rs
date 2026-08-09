//! F15.S1 — insert special / currency / math symbols via InsertText.

use tw_edit::{Command, EditSession};

fn first_run(session: &EditSession) -> tw_model::NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn u_f15_s1_insert_copyright() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "©".into(),
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "©"
    );
}

#[test]
fn u_f15_s1_insert_symbol_after_text() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Copyright ".into(),
        })
        .unwrap();

    let offset = session.document.paragraph_at(0, 0).unwrap().full_text().chars().count();
    session
        .apply(Command::InsertText {
            run_id,
            offset,
            text: "©".into(),
        })
        .unwrap();

    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().full_text(),
        "Copyright ©"
    );
}

#[test]
fn u_f15_s1_insert_currency_and_math_symbols() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);
    let symbols = ["€", "£", "±", "×", "∞", "→"];

    let mut offset = 0usize;
    for sym in symbols {
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: sym.to_string(),
            })
            .unwrap();
        offset += sym.chars().count();
    }

    let text = session.document.paragraph_at(0, 0).unwrap().full_text();
    for sym in symbols {
        assert!(text.contains(sym), "missing {sym} in {text:?}");
    }
}

#[test]
fn u_f15_s1_insert_symbol_undo() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "™".into(),
        })
        .unwrap();
    session.undo().unwrap();

    assert_eq!(session.document.paragraph_at(0, 0).unwrap().full_text(), "");
}

#[test]
fn u_f15_s1_insert_multiple_symbols_single_run() {
    let mut session = EditSession::new();
    let run_id = first_run(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "©®™".into(),
        })
        .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.full_text(), "©®™");
    assert_eq!(para.runs.len(), 1, "adjacent symbol inserts should stay in one run");
}
