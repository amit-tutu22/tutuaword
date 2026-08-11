//! F06.S4 — style inspector summary (U-F06-S4-inspector-shows-source).

use tw_model::{style_inspector_at, Document};

#[test]
fn u_f06_s4_inspector_shows_style_and_direct_char() {
    let mut doc = Document::with_paragraph("Title");
    let run_id = doc.paragraph_at_mut(0, 0).unwrap().runs[0].id;

    doc.paragraph_at_mut(0, 0).unwrap().runs[0].format.bold = Some(true);
    let summary = style_inspector_at(&doc, run_id).unwrap();
    assert_eq!(summary.display(), "Normal + Bold direct");

    let h1_id = doc.styles.find_style_by_name("Heading 1").unwrap().id;
    {
        let para = doc.paragraph_at_mut(0, 0).unwrap();
        para.style_id = Some(h1_id);
        para.runs[0].format.italic = Some(true);
    }
    let summary = style_inspector_at(&doc, run_id).unwrap();
    assert_eq!(summary.paragraph_style.as_deref(), Some("Heading 1"));
    assert_eq!(summary.display(), "Heading 1 + Italic direct");
}

#[test]
fn u_f06_s4_style_only_bold_not_listed_as_direct() {
    let mut doc = Document::with_paragraph("Heading text");
    let run_id = doc.paragraph_at_mut(0, 0).unwrap().runs[0].id;
    let h1_id = doc.styles.find_style_by_name("Heading 1").unwrap().id;
    doc.paragraph_at_mut(0, 0).unwrap().style_id = Some(h1_id);

    let summary = style_inspector_at(&doc, run_id).unwrap();
    assert_eq!(summary.display(), "Heading 1");
    assert!(summary.direct_char_labels.is_empty());
}
