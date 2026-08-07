//! R2.6 exit gate: undo transactions, coalescing, and total inverse.

use web_time::{Duration, Instant};
use tw_edit::{
    apply, Command, DocPosition, DocRange, EditError, EditSession, TransactionGuard,
};
use tw_model::{CharFormat, NodeId};

fn default_run_id(session: &EditSession) -> NodeId {
    session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id
}

fn paragraph_text(session: &EditSession) -> String {
    session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .full_text()
}

#[test]
fn hundred_char_word_types_as_one_undo_step() {
    let mut session = EditSession::new();
    let run_id = default_run_id(&session);

    for _ in 0..100 {
        let offset = paragraph_text(&session).chars().count();
        session
            .apply(Command::InsertText {
                run_id,
                offset,
                text: "a".into(),
            })
            .unwrap();
    }

    assert_eq!(session.undo_stack_len(), 1);
    assert_eq!(paragraph_text(&session), "a".repeat(100));

    session.undo().unwrap();
    assert_eq!(paragraph_text(&session), "");
    assert!(!session.can_undo());
}

#[test]
fn whitespace_breaks_coalescing() {
    let mut session = EditSession::new();
    let run_id = default_run_id(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "hello".into(),
        })
        .unwrap();
    assert_eq!(session.undo_stack_len(), 1);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 5,
            text: " ".into(),
        })
        .unwrap();
    assert_eq!(session.undo_stack_len(), 2);

    session.undo().unwrap();
    assert_eq!(paragraph_text(&session), "hello");
}

#[test]
fn coalesce_window_expiry_starts_new_undo_entry() {
    let mut session = EditSession::with_coalesce_window(Duration::from_secs(1));
    let run_id = default_run_id(&session);
    let t0 = Instant::now();
    session.set_test_clock(t0);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "ab".into(),
        })
        .unwrap();
    assert_eq!(session.undo_stack_len(), 1);

    session.set_test_clock(t0 + Duration::from_secs(2));
    session
        .apply(Command::InsertText {
            run_id,
            offset: 2,
            text: "cd".into(),
        })
        .unwrap();
    assert_eq!(session.undo_stack_len(), 2);
}

#[test]
fn transaction_commit_groups_commands() {
    let mut session = EditSession::new();
    let run_id = default_run_id(&session);
    let selection = DocRange {
        start: DocPosition {
            run_id,
            char_offset: 0,
        },
        end: DocPosition {
            run_id,
            char_offset: 0,
        },
    };

    let mut tx = session.begin_transaction(Some(selection.clone()));
    tx.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "a".into(),
    })
    .unwrap();
    tx.apply(Command::InsertText {
        run_id,
        offset: 1,
        text: "b".into(),
    })
    .unwrap();
    tx.apply(Command::InsertText {
        run_id,
        offset: 2,
        text: "c".into(),
    })
    .unwrap();
    tx.commit();

    assert_eq!(session.undo_stack_len(), 1);
    assert_eq!(session.selection_for_undo(0), Some(&selection));

    session.undo().unwrap();
    assert_eq!(paragraph_text(&session), "");
}

#[test]
fn transaction_abort_leaves_document_and_stack_unchanged() {
    let mut session = EditSession::new();
    let run_id = default_run_id(&session);

    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "seed".into(),
        })
        .unwrap();
    assert_eq!(session.undo_stack_len(), 1);

    let mut tx = session.begin_transaction(None);
    tx.apply(Command::InsertText {
        run_id,
        offset: 4,
        text: "X".into(),
    })
    .unwrap();
    tx.apply(Command::InsertText {
        run_id,
        offset: 5,
        text: "Y".into(),
    })
    .unwrap();
    tx.abort().unwrap();

    assert_eq!(paragraph_text(&session), "seed");
    assert_eq!(session.undo_stack_len(), 1);
}

#[test]
fn transaction_abort_on_failing_command_rolls_back_atomically() {
    let mut session = EditSession::new();
    let run_id = default_run_id(&session);
    let missing = NodeId::new();

    let mut tx = session.begin_transaction(None);
    tx.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "ok".into(),
    })
    .unwrap();
    let err = tx
        .apply(Command::InsertText {
            run_id: missing,
            offset: 0,
            text: "bad".into(),
        })
        .unwrap_err();
    assert!(matches!(err, EditError::RunNotFound(_)));
    tx.abort().unwrap();

    assert_eq!(paragraph_text(&session), "");
    assert_eq!(session.undo_stack_len(), 0);
}

#[test]
fn inverse_returns_err_for_non_invertible_restore_helpers() {
    let mut doc = tw_model::Document::new();
    let run_id = doc.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    let cmd = Command::RestoreRunFormats {
        formats: vec![(run_id, CharFormat::default())],
    };
    let result = apply(&mut doc, cmd.clone()).unwrap();
    let err = cmd.inverse(&result).unwrap_err();
    assert!(matches!(
        err,
        EditError::InverseNotSupported {
            command: "RestoreRunFormats"
        }
    ));
}

#[test]
fn batch_style_transaction_single_undo() {
    let mut session = EditSession::new();
    let run_id = default_run_id(&session);

    let mut tx: TransactionGuard<'_> = session.begin_transaction(None);
    let mut offset = 0usize;
    for ch in ["f", "o", "o"] {
        tx.apply(Command::InsertText {
            run_id,
            offset,
            text: ch.into(),
        })
        .unwrap();
        offset += 1;
    }
    tx.commit();

    assert_eq!(session.undo_stack_len(), 1);
    assert_eq!(paragraph_text(&session), "foo");
    session.undo().unwrap();
    assert_eq!(paragraph_text(&session), "");
}
