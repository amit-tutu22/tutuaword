use tw_shape::{AtlasKey, GlyphAtlas, RasterizedGlyph};

fn solid_glyph(width: u32, height: u32) -> RasterizedGlyph {
    RasterizedGlyph {
        width,
        height,
        bearing_x: 0.0,
        bearing_y: height as f32,
        rgba: vec![255; (width * height * 4) as usize],
        is_color: false,
    }
}

#[test]
fn atlas_grows_instead_of_silently_dropping_glyphs() {
    let mut atlas = GlyphAtlas::new(64, 64);
    let gen0 = atlas.generation;
    // Fill past the first row/height so grow() must expand.
    for i in 0..20 {
        let key = AtlasKey::new(1, i, 12.0);
        let entry = atlas.insert(key, &solid_glyph(20, 20));
        assert!(entry.width > 0 && entry.height > 0);
    }
    assert!(atlas.height > 64 || atlas.generation > gen0);
}
