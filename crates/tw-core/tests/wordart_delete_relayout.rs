//! WordArt delete must update layout glyphs.
use tw_edit::{Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::Block;

#[test]
fn wordart_delete_updates_layout_glyphs() {
    let mut session = EditSession::new();
    let after = match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        _ => panic!(),
    };
    session
        .apply(Command::InsertWordArt {
            after_block_id: after,
            text: "lhWordArt".into(),
            width: 220.0,
            height: 72.0,
        })
        .unwrap();
    let run_id = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs[0]
        .id;

    let mut engine = LayoutEngine::new();
    engine.layout_document(&session.document);
    let glyph_text = |engine: &LayoutEngine| -> String {
        engine
            .line_map(0)
            .unwrap()
            .lines
            .iter()
            .filter(|l| !l.decorative)
            .flat_map(|l| l.glyphs.iter().map(|g| g.codepoint))
            .collect()
    };
    let before = glyph_text(&engine);
    assert!(before.contains("lhWordArt"), "got {before}");

    session
        .apply(Command::DeleteRange {
            run_id,
            start: 1,
            end: 2,
        })
        .unwrap();
    engine.invalidate_nodes(&session.document, &[run_id]);
    engine.layout_document(&session.document);
    let after_text = glyph_text(&engine);
    assert_eq!(after_text.replace('¶', ""), "lWordArt");
}
