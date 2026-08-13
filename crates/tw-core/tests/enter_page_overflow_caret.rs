//! Enter at the bottom of a page must land the caret on the new paragraph
//! (possibly on page N+1), not snap back to the top of page N.

use tw_core::SyncSession;
use tw_edit::Command;
use tw_model::NodeId;

fn first_run(session: &SyncSession) -> NodeId {
    session
        .edit
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs[0]
        .id
}

#[test]
fn split_at_page_bottom_places_new_run_on_following_page() {
    let mut session = SyncSession::new();
    let mut run_id = first_run(&session);

    // Fill page 0 with empty paragraphs until layout grows past one page.
    for _ in 0..80 {
        session.apply(Command::SplitParagraphAt {
            run_id,
            offset: 0,
        });
        // Caret lands on the new paragraph's first run.
        let blocks = &session.edit.document.sections[0].blocks;
        let last = match blocks.last().unwrap() {
            tw_model::Block::Paragraph(p) => p.runs[0].id,
            _ => panic!("expected paragraph"),
        };
        run_id = last;
        session.relayout(None);
        if session.page_count() >= 2 {
            break;
        }
    }

    assert!(
        session.page_count() >= 2,
        "expected soft pagination after repeated Enter; got {} pages",
        session.page_count()
    );

    let new_run = run_id;
    // New run must not resolve on page 0 (that false-positive pinned the caret
    // to the top of the previous page in the Flutter shell).
    assert!(
        session.layout.line_map(0).unwrap().caret_at(new_run, 0).is_none(),
        "new paragraph after page overflow must not appear on page 0"
    );
    assert!(
        session
            .layout
            .line_map(1)
            .unwrap()
            .caret_at(new_run, 0)
            .is_some(),
        "new paragraph after page overflow must be caretable on page 1"
    );
}


#[test]
fn empty_enter_advances_caret_y_on_same_page() {
    let mut session = SyncSession::new();
    let mut run_id = first_run(&session);
    let mut prev_y = None;
    for i in 0..10 {
        session.apply(Command::SplitParagraphAt { run_id, offset: 0 });
        let blocks = &session.edit.document.sections[0].blocks;
        run_id = match blocks.last().unwrap() {
            tw_model::Block::Paragraph(p) => p.runs[0].id,
            _ => panic!("expected paragraph"),
        };
        session.relayout(None);
        let (x, y, h) = session
            .layout
            .line_map(0)
            .unwrap()
            .caret_at(run_id, 0)
            .expect("caret on page 0");
        let _ = (x, h);
        if let Some(py) = prev_y {
            assert!(
                y > py + 0.5,
                "enter {i}: caret y should advance; was {py}, now {y}"
            );
        }
        prev_y = Some(y);
        if session.page_count() > 1 {
            break;
        }
    }
}
