use tw_model::CharFormat;
use tw_shape::TextShaper;

#[test]
fn space_has_horizontal_advance() {
    let mut shaper = TextShaper::new();
    let font = shaper.default_font().expect("font");
    let shaped_a = shaper.shape("A", &CharFormat::default(), font);
    let shaped_as = shaper.shape("A ", &CharFormat::default(), font);
    let advance_a: f32 = shaped_a.glyphs.iter().map(|g| g.x_advance).sum();
    let advance_as: f32 = shaped_as.glyphs.iter().map(|g| g.x_advance).sum();
    eprintln!("advance A={advance_a} A-space={advance_as} glyphs_a={} glyphs_as={}", shaped_a.glyphs.len(), shaped_as.glyphs.len());
    assert!(advance_as > advance_a, "space should add advance");
}
