//! Stage 2 integration: format commands flow through SyncSession → layout →
//! display list, and empty-document typing remains editable.

use tw_core::SyncSession;
use tw_edit::{Command, DocPosition, DocRange};
use tw_model::{Alignment, CharFormat, ParaFormat};
use tw_render::DisplayListBuilder;

fn first_run(session: &SyncSession) -> tw_model::NodeId {
    session.edit.document.paragraph_at(0, 0).unwrap().runs[0].id
}

#[test]
fn empty_document_layouts_and_exports_display_list() {
    let mut session = SyncSession::new();
    let bytes = session.display_list_bytes();
    assert!(!bytes.is_empty());
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert!(decoded.page_width > 0.0);
    assert!(decoded.page_height > 0.0);
}

#[test]
fn typing_into_empty_run_produces_glyphs() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);

    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "Hello".into(),
    });

    assert_eq!(
        session.edit.document.paragraph_at(0, 0).unwrap().full_text(),
        "Hello"
    );

    let bytes = session.display_list_bytes();
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert!(
        !decoded.atlas_batch.transforms.is_empty(),
        "typed text should produce glyph transforms"
    );
}

#[test]
fn bold_format_then_layout_keeps_text() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);

    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "Bold me".into(),
    });
    session.apply(Command::SetCharFormat {
        run_id,
        start: 0,
        end: 4,
        format: CharFormat {
            bold: Some(true),
            ..Default::default()
        },
        merge: true,
    });

    let para = session.edit.document.paragraph_at(0, 0).unwrap();
    let bold_runs: Vec<_> = para
        .runs
        .iter()
        .filter(|r| r.format.bold == Some(true))
        .collect();
    assert!(!bold_runs.is_empty());
    assert_eq!(bold_runs[0].text(), "Bold");

    let bytes = session.display_list_bytes();
    assert!(!bytes.is_empty());
}

#[test]
fn para_alignment_survives_relayout() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);

    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "Centered".into(),
    });
    session.apply(Command::SetParaFormatRange {
        range: DocRange {
            start: DocPosition {
                run_id,
                char_offset: 0,
            },
            end: DocPosition {
                run_id,
                char_offset: 8,
            },
        },
        format: ParaFormat {
            alignment: Some(Alignment::Center),
            ..Default::default()
        },
        merge: true,
    });

    session.relayout(None);
    assert_eq!(
        session.edit.document.paragraph_at(0, 0).unwrap().format.alignment,
        Some(Alignment::Center)
    );
    assert!(session.page_count() >= 1);
}

#[test]
fn delete_range_updates_plain_text_and_display_list() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);

    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "abcdef".into(),
    });
    session.apply(Command::DeleteRange {
        run_id,
        start: 1,
        end: 4,
    });

    assert_eq!(
        session.edit.document.paragraph_at(0, 0).unwrap().full_text(),
        "aef"
    );

    let before = session.display_list_bytes();
    assert!(!before.is_empty());
}

#[test]
fn format_range_across_runs_then_export_bytes() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);

    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "abcdefgh".into(),
    });
    // Split into two runs.
    session.apply(Command::SetCharFormat {
        run_id,
        start: 4,
        end: 8,
        format: CharFormat {
            italic: Some(true),
            ..Default::default()
        },
        merge: true,
    });

    let runs = &session.edit.document.paragraph_at(0, 0).unwrap().runs;
    assert_eq!(runs.len(), 2);
    let start = DocPosition {
        run_id: runs[0].id,
        char_offset: 2,
    };
    let end = DocPosition {
        run_id: runs[1].id,
        char_offset: 2,
    };

    session.apply(Command::SetCharFormatRange {
        range: DocRange { start, end },
        format: CharFormat {
            bold: Some(true),
            ..Default::default()
        },
        merge: true,
    });

    let bold: String = session
        .edit
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs
        .iter()
        .filter(|r| r.format.bold == Some(true))
        .map(|r| r.text().to_string())
        .collect();
    assert_eq!(bold, "cdef");

    let bytes = session.display_list_bytes();
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert!(!decoded.atlas_batch.transforms.is_empty());
}

#[test]
fn trailing_space_after_typed_character_advances_caret() {
    let mut session = SyncSession::new();
    let run_id = first_run(&session);

    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "A".into(),
    });
    let map_a = session.layout.line_map(0).unwrap();
    let (_, x_end_a, _, _) = map_a.lines[0].run_map[0];
    let caret_after_a = map_a.caret_at(run_id, 1).unwrap().0;

    session.apply(Command::InsertText {
        run_id,
        offset: 1,
        text: " ".into(),
    });
    assert_eq!(
        session.edit.document.paragraph_at(0, 0).unwrap().full_text(),
        "A "
    );

    let map_as = session.layout.line_map(0).unwrap();
    let (_, x_end_as, _, _) = map_as.lines[0].run_map[0];
    let caret_after_space = map_as.caret_at(run_id, 2).unwrap().0;

    assert!(
        x_end_as > x_end_a,
        "trailing space should widen layout before the next character"
    );
    assert!(
        caret_after_space > caret_after_a,
        "caret should advance immediately after inserting a trailing space"
    );
}
