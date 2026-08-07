//! R3.1: faces supplied to the engine as bytes, with no system font scan and no
//! filesystem access, which is the only thing a web host can do.

mod font_fixture;

use tw_model::CharFormat;
use tw_shape::{FontDatabase, FontFaceSpec, FontRegistrationError, TextShaper};

const FAMILY: &str = "Injected Web Sans";

fn text_format(size: f32) -> CharFormat {
    CharFormat {
        font_family: Some(FAMILY.into()),
        font_size: Some(size),
        ..Default::default()
    }
}

#[test]
fn an_empty_database_holds_no_faces_and_resolves_nothing() {
    let mut fonts = FontDatabase::empty();

    assert!(fonts.is_empty());
    assert_eq!(fonts.face_count(), 0);
    assert_eq!(fonts.families(), Vec::<String>::new());
    assert!(fonts.resolve(None).is_none());
    assert!(fonts.resolve_styled(Some("Calibri"), true, true).is_none());
    assert!(fonts.fallback_for('A').is_none());
}

#[test]
fn a_registered_family_resolves_with_no_system_database_behind_it() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("a_registered_family_resolves_with_no_system_database_behind_it");
    };

    let mut fonts = FontDatabase::empty();
    let font = fonts
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("host font bytes should register");

    assert_eq!(
        fonts.face_count(),
        1,
        "the injected face must be the only one; a system scan would add hundreds"
    );
    assert_eq!(fonts.families(), vec![FAMILY.to_string()]);
    assert_eq!(fonts.resolve(Some(FAMILY)), Some(font));
    assert_eq!(
        fonts.resolve(Some("injected web sans")),
        Some(font),
        "the host chose the label, so it matches case-insensitively"
    );
}

#[test]
fn a_family_the_host_never_registered_falls_back_to_one_it_did() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("a_family_the_host_never_registered_falls_back_to_one_it_did");
    };

    let mut fonts = FontDatabase::empty();
    let font = fonts
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("register");

    // Neither the requested family nor the built-in default family exists here.
    assert_eq!(fonts.resolve(Some("Wingdings 17")), Some(font));
    assert_eq!(fonts.resolve(None), Some(font));
    assert_eq!(
        fonts.resolve_styled(Some("Wingdings 17"), true, true),
        Some(font),
        "a wrong-style match beats rendering nothing"
    );
}

#[test]
fn styled_lookup_picks_the_face_registered_for_that_style() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("styled_lookup_picks_the_face_registered_for_that_style");
    };

    let mut fonts = FontDatabase::empty();
    let regular = fonts
        .register_face(&FontFaceSpec::new(FAMILY), bytes.clone())
        .expect("regular");
    let bold = fonts
        .register_face(&FontFaceSpec::new(FAMILY).bold(), bytes.clone())
        .expect("bold");
    let italic = fonts
        .register_face(&FontFaceSpec::new(FAMILY).italic(), bytes.clone())
        .expect("italic");
    let bold_italic = fonts
        .register_face(&FontFaceSpec::new(FAMILY).bold().italic(), bytes)
        .expect("bold italic");

    assert_eq!(fonts.face_count(), 4);
    assert_eq!(fonts.resolve_styled(Some(FAMILY), false, false), Some(regular));
    assert_eq!(fonts.resolve_styled(Some(FAMILY), true, false), Some(bold));
    assert_eq!(fonts.resolve_styled(Some(FAMILY), false, true), Some(italic));
    assert_eq!(
        fonts.resolve_styled(Some(FAMILY), true, true),
        Some(bold_italic)
    );
}

#[test]
fn a_registered_face_carries_its_own_metrics_and_coverage() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("a_registered_face_carries_its_own_metrics_and_coverage");
    };

    let mut fonts = FontDatabase::empty();
    let font = fonts
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("register");

    let size = 24.0;
    let (ascent, descent, _gap) = fonts.vertical_metrics(font, size);
    assert!(ascent > 0.0 && descent > 0.0);
    assert_ne!(
        (ascent, descent),
        (size, size * 0.25),
        "metrics should come from the injected face, not the no-face defaults"
    );
    assert!(fonts.covers(font, 'H'));
    assert!(fonts.face_data(font).is_some_and(|data| !data.is_empty()));
}

#[test]
fn registration_rejects_bytes_that_are_not_a_font() {
    let mut fonts = FontDatabase::empty();

    let error = fonts
        .register_face(&FontFaceSpec::new("Bogus"), vec![0u8; 128])
        .expect_err("garbage must not register");

    assert_eq!(error, FontRegistrationError::UnreadableFontData);
    assert_eq!(fonts.face_count(), 0, "a rejected face must leave no trace");
}

#[test]
fn registration_rejects_a_face_index_past_the_end_of_the_file() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("registration_rejects_a_face_index_past_the_end_of_the_file");
    };

    let mut fonts = FontDatabase::empty();
    let error = fonts
        .register_face(&FontFaceSpec::new(FAMILY).index(9), bytes)
        .expect_err("index 9 is past the end of a single-face file");

    assert!(matches!(
        error,
        FontRegistrationError::FaceIndexOutOfRange { index: 9, .. }
    ));
    assert_eq!(fonts.face_count(), 0);
}

#[test]
fn register_font_data_keeps_the_names_inside_the_file() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("register_font_data_keeps_the_names_inside_the_file");
    };

    let mut fonts = FontDatabase::empty();
    let registered = fonts.register_font_data(bytes).expect("register");

    assert!(!registered.is_empty());
    let families = fonts.families();
    assert!(!families.is_empty(), "the file's own family name is used");
    assert_eq!(
        fonts.resolve(Some(&families[0])),
        Some(registered[0]),
        "the family the file declares resolves to the face it declared it for"
    );
}

#[test]
fn shaping_an_injected_face_yields_advancing_real_glyphs() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("shaping_an_injected_face_yields_advancing_real_glyphs");
    };

    let mut shaper = TextShaper::with_injected_fonts();
    let font = shaper
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("register");

    let shaped = shaper.shape("Handgloves", &text_format(18.0), font);

    assert_eq!(shaped.glyphs.len(), "Handgloves".chars().count());
    assert!(
        shaped.glyphs.iter().all(|g| g.x_advance > 0.0),
        "every glyph should advance the pen"
    );
    assert!(
        shaped.glyphs.iter().all(|g| g.glyph_id != 0),
        "glyph id 0 is .notdef — the face was not really consulted"
    );
    assert!(shaped.glyphs.iter().all(|g| g.font_key() == font.key()));
}

#[test]
fn an_injected_face_rasterizes_to_a_bitmap_with_ink() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("an_injected_face_rasterizes_to_a_bitmap_with_ink");
    };

    let mut shaper = TextShaper::with_injected_fonts();
    let font = shaper
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("register");
    let size = 48.0;
    let glyph_id = shaper.shape("H", &text_format(size), font).glyphs[0].glyph_id;

    let raster = shaper.rasterize_glyph(font, glyph_id, size);

    assert!(raster.width > 0 && raster.height > 0, "expected a bitmap");
    assert!(
        raster.rgba.chunks(4).any(|px| px[3] > 0),
        "expected non-empty glyph coverage"
    );
}

#[test]
fn theme_configuration_resolves_against_injected_faces() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("theme_configuration_resolves_against_injected_faces");
    };

    let mut shaper = TextShaper::with_injected_fonts();
    let body = shaper
        .register_face(&FontFaceSpec::new("Host Body"), bytes.clone())
        .expect("body");
    let heading = shaper
        .register_face(&FontFaceSpec::new("Host Heading"), bytes)
        .expect("heading");
    shaper.configure_from_theme("Host Body", "Host Heading");

    assert_eq!(shaper.default_font(), Some(body));
    assert_eq!(shaper.fonts_mut().resolve(Some("Host Heading")), Some(heading));
    assert_eq!(
        shaper.fonts_mut().resolve(Some("Calibri")),
        Some(body),
        "an unregistered family lands on the theme's minor font"
    );
}

#[test]
fn coverage_fallback_searches_only_injected_faces() {
    let Some(bytes) = font_fixture::any_font_bytes() else {
        return font_fixture::skipped("coverage_fallback_searches_only_injected_faces");
    };

    let mut fonts = FontDatabase::empty();
    let font = fonts
        .register_face(&FontFaceSpec::new(FAMILY), bytes)
        .expect("register");

    assert_eq!(fonts.fallback_for('A'), Some(font));
    // A face the host did not supply coverage for resolves to nothing rather
    // than reaching for an installed font.
    assert_eq!(fonts.fallback_for('\u{10FFFD}'), None);
}
