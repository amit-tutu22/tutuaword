//! F24.S2 — starter template → DocumentTheme gallery binding.

use tw_model::DocumentTheme;

/// Mirrors `DocumentTemplateSpec.themeName` in the Flutter catalog.
const TEMPLATE_THEMES: &[(&str, &str)] = &[
    ("resume", "Facet"),
    ("letter", "Office"),
    ("invoice", "Ion"),
    ("brochure", "Facet"),
    ("newsletter", "Ion"),
    ("business_proposal", "Office"),
    ("research_paper", "Facet"),
];

#[test]
fn u_f24_s2_template_theme_map() {
    assert_eq!(TEMPLATE_THEMES.len(), 7);
    for (template_id, theme_name) in TEMPLATE_THEMES {
        let theme = DocumentTheme::by_name(theme_name)
            .unwrap_or_else(|| panic!("{template_id} binds unknown theme {theme_name}"));
        assert_eq!(theme.name, *theme_name);
        assert!(
            DocumentTheme::gallery_themes().contains(theme_name),
            "{theme_name} must be a Design gallery theme"
        );
    }
}
