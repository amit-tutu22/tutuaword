//! Alt+Shift+Up / Down — move the caret's paragraph among its siblings.

use tw_edit::{move_block_command_for_caret, Command, EditSession};
use tw_model::{Block, Paragraph};

fn session_with(texts: &[&str]) -> EditSession {
    let mut session = EditSession::new();
    session.document.sections[0].blocks = texts
        .iter()
        .map(|text| Block::Paragraph(Paragraph::with_text(*text)))
        .collect();
    session
}

fn order(session: &EditSession) -> Vec<String> {
    session.document.sections[0]
        .blocks
        .iter()
        .filter_map(|block| block.paragraph())
        .map(|para| para.full_text())
        .collect()
}

fn run_id(session: &EditSession, block_index: usize) -> tw_model::NodeId {
    session.document.sections[0].blocks[block_index]
        .paragraph()
        .unwrap()
        .runs[0]
        .id
}

#[test]
fn u_f05_move_block_down_swaps_with_the_next_paragraph() {
    let mut session = session_with(&["alpha", "beta", "gamma"]);
    let caret = run_id(&session, 0);

    let command = move_block_command_for_caret(&session.document, Some(caret), 1).unwrap();
    session.apply(command).unwrap();

    assert_eq!(order(&session), ["beta", "alpha", "gamma"]);
}

#[test]
fn u_f05_move_block_up_swaps_with_the_previous_paragraph() {
    let mut session = session_with(&["alpha", "beta", "gamma"]);
    let caret = run_id(&session, 2);

    let command = move_block_command_for_caret(&session.document, Some(caret), -1).unwrap();
    session.apply(command).unwrap();

    assert_eq!(order(&session), ["alpha", "gamma", "beta"]);
}

#[test]
fn u_f05_move_block_undo_restores_the_original_order() {
    let mut session = session_with(&["alpha", "beta", "gamma"]);
    let caret = run_id(&session, 1);

    let command = move_block_command_for_caret(&session.document, Some(caret), -1).unwrap();
    session.apply(command).unwrap();
    assert_eq!(order(&session), ["beta", "alpha", "gamma"]);

    session.undo().unwrap();
    assert_eq!(order(&session), ["alpha", "beta", "gamma"]);

    session.redo().unwrap();
    assert_eq!(order(&session), ["beta", "alpha", "gamma"]);
}

#[test]
fn u_f05_move_block_at_the_document_edge_is_a_no_op() {
    let session = session_with(&["alpha", "beta"]);
    let first = run_id(&session, 0);
    let last = run_id(&session, 1);

    assert!(move_block_command_for_caret(&session.document, Some(first), -1).is_none());
    assert!(move_block_command_for_caret(&session.document, Some(last), 1).is_none());
}

#[test]
fn u_f05_move_block_keeps_paragraph_formatting_with_the_text() {
    let mut session = session_with(&["alpha", "beta"]);
    session.document.sections[0].blocks[1]
        .paragraph_mut()
        .unwrap()
        .format
        .indent_left = Some(72.0);
    let caret = run_id(&session, 1);

    let command = move_block_command_for_caret(&session.document, Some(caret), -1).unwrap();
    session.apply(command).unwrap();

    let moved = session.document.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(moved.full_text(), "beta");
    assert_eq!(moved.format.indent_left, Some(72.0));
}

#[test]
fn u_f05_move_block_delta_zero_builds_no_command() {
    let session = session_with(&["alpha", "beta"]);
    let caret = run_id(&session, 0);
    assert!(move_block_command_for_caret(&session.document, Some(caret), 0).is_none());
    assert!(matches!(
        Command::MoveBlock {
            id: session.document.sections[0].blocks[0].paragraph().unwrap().id,
            delta: 1,
        },
        Command::MoveBlock { delta: 1, .. }
    ));
}
