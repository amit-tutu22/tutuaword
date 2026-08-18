//! F04.S3 — Explicit paragraph tab stops override the default tab grid.

use tw_layout::{layout_paragraph, ParagraphFrame};
use tw_model::{Paragraph, TabAlignment, TabStop};
use tw_shape::{GlyphAtlas, TextShaper};

fn b_glyph_x(para: &Paragraph, shaper: &mut TextShaper, atlas: &mut GlyphAtlas) -> f32 {
    let (lines, _) = layout_paragraph(
        shaper,
        atlas,
        para,
        ParagraphFrame::new(0.0, 0.0, 400.0)
            .with_tab_interval(36.0)
            .with_tab_stops(para.format.tab_stops.clone().unwrap_or_default()),
        0xFF000000,
    );
    assert_eq!(lines.len(), 1, "A\\tB should stay on one line");
    lines[0]
        .glyphs
        .iter()
        .find(|g| g.codepoint == 'B')
        .expect("B glyph")
        .x
}

/// U-F04-S3-custom-tab-stop: explicit stop at 200 pt lands `B` at that x.
#[test]
fn u_f04_s3_custom_tab_stop() {
    let mut shaper = TextShaper::new();
    let mut atlas = GlyphAtlas::new(512, 512);
    let mut para = Paragraph::with_text("A\tB");
    para.format.tab_stops = Some(vec![TabStop {
        position: 200.0,
        alignment: TabAlignment::Left,
        ..Default::default()
    }]);

    let x = b_glyph_x(&para, &mut shaper, &mut atlas);
    assert!(
        (x - 200.0).abs() <= 2.0,
        "tab should land at 200 pt ± 2, got {x}"
    );
}

#[test]
fn explicit_tab_stop_beats_default_grid() {
    let mut shaper = TextShaper::new();
    let mut atlas = GlyphAtlas::new(512, 512);

    let mut custom = Paragraph::with_text("A\tB");
    custom.format.tab_stops = Some(vec![TabStop {
        position: 144.0,
        alignment: TabAlignment::Left,
        ..Default::default()
    }]);
    let plain = Paragraph::with_text("A\tB");

    let custom_x = b_glyph_x(&custom, &mut shaper, &mut atlas);
    let plain_x = b_glyph_x(&plain, &mut shaper, &mut atlas);

    assert!(
        (custom_x - 144.0).abs() <= 2.0,
        "custom stop should land at 144 pt ± 2, got {custom_x}"
    );
    assert!(
        (plain_x - 36.0).abs() <= 2.0,
        "default grid first stop is 36 pt ± 2, got {plain_x}"
    );
    assert!(
        custom_x >= plain_x + 100.0,
        "custom stop ({custom_x}) must sit well past default ({plain_x})"
    );
}
