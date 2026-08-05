//! Characters the run font lacks are shaped with a fallback face rather than
//! coming out as `.notdef`.

use tw_model::CharFormat;
use tw_shape::TextShaper;

/// Not every machine ships the same fonts, so skip rather than fail when no
/// installed face covers the character under test.
fn fallback_available(shaper: &mut TextShaper, ch: char) -> bool {
    shaper.fonts_mut().fallback_for(ch).is_some()
}

#[test]
fn the_rupee_sign_is_not_rendered_as_notdef() {
    let mut shaper = TextShaper::new();
    let font = shaper.default_font().expect("a default font");
    if !fallback_available(&mut shaper, '₹') {
        return;
    }

    let shaped = shaper.shape("₹", &CharFormat::default(), font);

    assert_eq!(shaped.glyphs.len(), 1);
    assert_ne!(shaped.glyphs[0].glyph_id, 0, "should not be .notdef");
    assert!(shaped.glyphs[0].x_advance > 0.0);
}

#[test]
fn only_the_uncovered_character_switches_face() {
    let mut shaper = TextShaper::new();
    let font = shaper.default_font().expect("a default font");
    if shaper.fonts_mut().covers(font, '₹') || !fallback_available(&mut shaper, '₹') {
        return; // The primary font has it, so there is nothing to fall back to.
    }

    let shaped = shaper.shape("in ₹)", &CharFormat::default(), font);

    let rupee = shaped
        .glyphs
        .iter()
        .find(|g| g.cluster == 3)
        .expect("a glyph for the rupee sign");
    assert_ne!(rupee.font_key(), font.key(), "rupee uses a fallback face");

    for glyph in shaped.glyphs.iter().filter(|g| g.cluster != 3) {
        assert_eq!(glyph.font_key(), font.key(), "ASCII stays on the run font");
    }
}

#[test]
fn text_the_font_covers_shapes_with_a_single_face() {
    let mut shaper = TextShaper::new();
    let font = shaper.default_font().expect("a default font");

    let shaped = shaper.shape("Closing Price", &CharFormat::default(), font);

    assert!(!shaped.glyphs.is_empty());
    assert!(shaped.glyphs.iter().all(|g| g.font_key() == font.key()));
    assert!(shaped.glyphs.iter().all(|g| g.glyph_id != 0));
}

#[test]
fn clusters_are_character_indices_not_byte_offsets() {
    let mut shaper = TextShaper::new();
    let font = shaper.default_font().expect("a default font");

    // The rupee sign is three bytes, so byte offsets would overshoot from here.
    let shaped = shaper.shape("a₹bc", &CharFormat::default(), font);

    let max_cluster = shaped.glyphs.iter().map(|g| g.cluster).max().unwrap();
    assert_eq!(max_cluster, 3, "last of four characters");
    assert_eq!(shaped.cluster_to_glyph.len(), 4);
}

#[test]
fn a_space_does_not_split_a_run_onto_a_fallback_face() {
    let mut shaper = TextShaper::new();
    let font = shaper.default_font().expect("a default font");

    let shaped = shaper.shape("a b", &CharFormat::default(), font);

    assert!(shaped.glyphs.iter().all(|g| g.font_key() == font.key()));
}

#[test]
fn resolving_a_fallback_twice_returns_the_same_face() {
    let mut shaper = TextShaper::new();
    let first = shaper.fonts_mut().fallback_for('₹');
    let second = shaper.fonts_mut().fallback_for('₹');

    assert_eq!(first, second);
}
