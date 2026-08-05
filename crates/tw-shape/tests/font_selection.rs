//! Runs are shaped with the family, weight, and slant they ask for.

use tw_shape::FontDatabase;

#[test]
fn an_unknown_family_falls_back_to_the_default() {
    let mut fonts = FontDatabase::new();

    let missing = fonts.resolve_styled(Some("Definitely Not Installed 9000"), false, false);
    let default = fonts.resolve_styled(None, false, false);

    assert_eq!(missing, default, "should not pick an arbitrary face");
    assert!(default.is_some());
}

#[test]
fn an_empty_family_is_treated_as_unset() {
    let mut fonts = FontDatabase::new();

    assert_eq!(
        fonts.resolve_styled(Some("   "), false, false),
        fonts.resolve_styled(None, false, false)
    );
}

#[test]
fn bold_resolves_to_a_different_face_than_regular() {
    let mut fonts = FontDatabase::new();
    let regular = fonts.resolve_styled(Some("Arial"), false, false);
    let bold = fonts.resolve_styled(Some("Arial"), true, false);

    if regular.is_none() || bold.is_none() {
        return; // Arial is not installed on this machine.
    }
    assert_ne!(regular, bold, "bold should select the bold face");
}

#[test]
fn italic_resolves_to_a_different_face_than_regular() {
    let mut fonts = FontDatabase::new();
    let regular = fonts.resolve_styled(Some("Arial"), false, false);
    let italic = fonts.resolve_styled(Some("Arial"), false, true);

    if regular.is_none() || italic.is_none() {
        return;
    }
    assert_ne!(regular, italic);
}

#[test]
fn distinct_families_resolve_to_distinct_faces() {
    let mut fonts = FontDatabase::new();
    let sans = fonts.resolve_styled(Some("Arial"), false, false);
    let serif = fonts.resolve_styled(Some("Times New Roman"), false, false);

    if sans.is_none() || serif.is_none() {
        return;
    }
    assert_ne!(sans, serif);
}

#[test]
fn resolving_the_same_request_twice_is_stable() {
    let mut fonts = FontDatabase::new();

    assert_eq!(
        fonts.resolve_styled(Some("Georgia"), true, false),
        fonts.resolve_styled(Some("Georgia"), true, false)
    );
    assert_eq!(
        fonts.resolve_styled(Some("Nope Not Real"), false, false),
        fonts.resolve_styled(Some("Nope Not Real"), false, false)
    );
}
