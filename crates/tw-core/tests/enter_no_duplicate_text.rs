use tw_core::SyncSession;
use tw_edit::Command;
use tw_model::NodeId;
use tw_render::DisplayListBuilder;

fn first_run(session: &SyncSession) -> NodeId {
    session.edit.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn end_enter_repeated_keeps_single_text_copy() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "fffjfgkgjjk".into(),
    });

    let mut caret_run = run_id;
    let mut caret_off = 11usize;
    for i in 0..12 {
        session.apply(Command::SplitParagraphAt {
            run_id: caret_run,
            offset: caret_off,
        });
        let blocks = &session.edit.document.sections[0].blocks;
        let last = match blocks.last().unwrap() {
            tw_model::Block::Paragraph(p) => p,
            _ => panic!("expected paragraph after enter {i}"),
        };
        assert_eq!(
            last.full_text(),
            "",
            "new para after enter {i} must be empty"
        );
        caret_run = last.runs[0].id;
        caret_off = 0;
        session.relayout(None);
    }

    let texts: Vec<String> = session.edit.document.sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph().map(|p| p.full_text()))
        .collect();
    assert_eq!(
        texts.iter().filter(|t| t.as_str() == "fffjfgkgjjk").count(),
        1,
        "model={texts:?}"
    );

    // Painted glyphs for the phrase must not multiply with each Enter.
    let bytes = session.display_list_bytes();
    let list = DisplayListBuilder::from_bytes(&bytes).expect("display list");
    let glyph_count = list.atlas_batch.transforms.len() / 2;
    assert!(
        glyph_count < 30,
        "Enter must not stack stale glyphs; got {glyph_count} glyphs for one short line"
    );
}

#[test]
fn split_at_zero_does_not_leave_duplicate_model_text() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "fffjfgkgjjk".into(),
    });
    let mut caret_run = run_id;
    for i in 0..5 {
        session.apply(Command::SplitParagraphAt {
            run_id: caret_run,
            offset: 0,
        });
        let blocks = &session.edit.document.sections[0].blocks;
        let last = match blocks.last().unwrap() {
            tw_model::Block::Paragraph(p) => p,
            _ => panic!("para"),
        };
        caret_run = last.runs[0].id;
        session.relayout(None);
        let copies = session
            .edit
            .document
            .sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph().map(|p| p.full_text()))
            .filter(|t| t == "fffjfgkgjjk")
            .count();
        assert_eq!(copies, 1, "after enter-at-0 #{i} still one model copy");
    }
}
