//! Empty header band must be hit-testable across the content width.

use tw_edit::{Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::{Document, HeaderFooterType, SectionFormat};

#[test]
fn empty_header_band_is_hittable_across_width() {
    let mut session = EditSession::from_document(Document::new());
    let seed = session
        .apply(Command::EnsureHeaderFooter {
            section_index: 0,
            is_header: true,
            hf_type: HeaderFooterType::Default,
        })
        .unwrap()
        .seed_run_id
        .unwrap();

    let mut engine = LayoutEngine::new();
    let _ = engine.layout_document(&session.document);
    let map = engine.line_map(0).expect("page");
    let format = SectionFormat::default();
    let header_y = format.margin_top * 0.25;
    let content_width = format.page_width - format.margin_left - format.margin_right;
    let mid_x = format.margin_left + content_width * 0.5;

    let hit = map
        .hit_test(mid_x, header_y)
        .expect("empty header mid-band must resolve a caret");
    assert_eq!(hit.run_id, seed);
    assert_eq!(hit.char_offset, 0);

    let caret = map
        .caret_at(seed, 0)
        .expect("header seed run must have caret geometry");
    assert!(
        caret.1 < format.margin_top,
        "caret baseline {} should sit in the header margin above {}",
        caret.1,
        format.margin_top
    );
}
