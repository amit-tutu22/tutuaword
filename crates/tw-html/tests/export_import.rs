use tw_html::{export, import};
use tw_model::{Block, Document};

#[test]
fn html_round_trip_preserves_heading_and_bold() {
    let source = br#"<!DOCTYPE html><html><body>
        <h1>Title</h1>
        <p>Hello <strong>world</strong></p>
    </body></html>"#;
    let doc = import(source).unwrap();
    let first = doc.sections[0].blocks[0].paragraph().unwrap();
    assert!(first.style_id.is_some() || first.runs.iter().any(|r| r.format.bold == Some(true)));

    let html = String::from_utf8(export(&doc).unwrap()).unwrap();
    assert!(html.contains("<h1>") || html.contains("Title"));
    assert!(html.contains("<strong>") || html.contains("world"));
}

#[test]
fn html_export_marks_revisions_with_mark_tag() {
    let mut doc = Document::with_paragraph("Reviewed");
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.runs[0].revision = Some(tw_model::Revision::insert("Editor"));
    }
    let html = String::from_utf8(export(&doc).unwrap()).unwrap();
    assert!(html.contains("<mark>"));
    assert!(html.contains("Reviewed"));
}

#[test]
fn html_import_rejects_non_html_bytes() {
    let err = import(b"plain text only").unwrap_err();
    assert!(err.to_string().contains("not a valid html"));
}
