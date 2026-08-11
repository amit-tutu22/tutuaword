//! R3.1: a whole document laid out from host-supplied font bytes, with the
//! system font database never consulted.
//!
//! This is the test that makes the web font path verifiable on a native CI host:
//! `LayoutEngine::with_injected_fonts` performs no system scan, so if the
//! resulting layout has real glyphs with real advances, they can only have come
//! from the bytes the host handed over.

mod font_fixture;

use tw_layout::{FontFaceSpec, LayoutBox, LayoutEngine, TextLine};
use tw_model::{Block, CharFormat, Document, Paragraph, Run};

const FAMILY: &str = "Injected Web Sans";
const TEXT: &str = "Handgloves";

fn document_in(family: &str) -> Document {
    let mut run = Run::new_text(TEXT);
    run.format = CharFormat {
        font_family: Some(family.into()),
        font_size: Some(18.0),
        ..Default::default()
    };
    let mut para = Paragraph::new();
    para.runs = vec![run];
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(para)];
    doc
}

fn text_lines(layout: &tw_layout::DocumentLayout) -> Vec<&TextLine> {
    layout
        .pages
        .iter()
        .flat_map(|page| &page.boxes)
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line),
            _ => None,
        })
        .collect()
}

#[test]
fn a_document_lays_out_from_injected_font_bytes_alone() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("a_document_lays_out_from_injected_font_bytes_alone");
    };

    let mut engine = LayoutEngine::with_injected_fonts();
    assert_eq!(
        engine.shaper().fonts().face_count(),
        0,
        "construction must not scan the system for fonts"
    );

    let font = engine
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("host font bytes should register");

    assert_eq!(
        engine.shaper().fonts().face_count(),
        1,
        "the injected face is the only one the engine can see"
    );
    assert_eq!(engine.shaper().fonts().families(), vec![FAMILY.to_string()]);

    let layout = engine.layout_document(&document_in(FAMILY));
    let lines = text_lines(&layout);
    assert_eq!(lines.len(), 1, "one short paragraph is one line");
    let line = lines[0];

    assert_eq!(
        line.glyphs.len(),
        TEXT.chars().count(),
        "every character should have produced an inked glyph"
    );
    assert!(line.width > 0.0, "line should have measurable width");
    assert!(line.ascent > 0.0 && line.descent > 0.0);
    assert!(
        line.glyphs.iter().all(|g| g.glyph_id != 0),
        "glyph id 0 is .notdef — the injected face was never really shaped with"
    );
    assert!(
        line.glyphs.iter().all(|g| g.font_id == font.key()),
        "every glyph must come from the injected face"
    );
    for pair in line.glyphs.windows(2) {
        assert!(
            pair[1].x > pair[0].x,
            "glyphs must advance: {} then {}",
            pair[0].x,
            pair[1].x
        );
    }
    assert!(
        line.glyphs.iter().all(|g| g.width > 0.0 && g.height > 0.0),
        "each glyph should carry a rasterized atlas rect"
    );
    assert!(
        engine.atlas().generation > 0,
        "rasterizing injected glyphs should have filled the atlas"
    );
}

#[test]
fn a_family_the_host_never_registered_still_renders() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("a_family_the_host_never_registered_still_renders");
    };

    let mut engine = LayoutEngine::with_injected_fonts();
    let font = engine
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("register");

    // The document asks for a family the host never supplied, and the theme's
    // minor font is missing too. Falling through to the one available face is
    // the defined behaviour; a blank page or a panic is not.
    let layout = engine.layout_document(&document_in("Calibri"));
    let lines = text_lines(&layout);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].glyphs.len(), TEXT.chars().count());
    assert!(lines[0].glyphs.iter().all(|g| g.font_id == font.key()));
}

#[test]
fn layout_without_any_registered_font_is_blank_rather_than_a_panic() {
    let mut engine = LayoutEngine::with_injected_fonts();

    let layout = engine.layout_document(&document_in(FAMILY));

    assert!(!layout.pages.is_empty(), "the document should still paginate");
    let lines = text_lines(&layout);
    assert!(!lines.is_empty(), "the paragraph should still occupy a line");
    assert!(
        lines.iter().all(|line| line.glyphs.is_empty()),
        "there is no face to draw with, so there is nothing to draw"
    );
    assert!(lines.iter().all(|line| line.line_height > 0.0));
}

#[test]
fn registering_a_face_invalidates_layout_produced_without_it() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("registering_a_face_invalidates_layout_produced_without_it");
    };

    let mut engine = LayoutEngine::with_injected_fonts();
    let doc = document_in(FAMILY);

    let before = engine.layout_document(&doc);
    assert!(text_lines(&before)[0].glyphs.is_empty());

    engine
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("register");

    let after = engine.layout_document(&doc);
    assert_eq!(
        text_lines(&after)[0].glyphs.len(),
        TEXT.chars().count(),
        "cached glyph-free pages must not survive a font registration"
    );
}
