use tw_html::{export, import};
use tw_model::{Block, Document, Paragraph};

#[test]
fn html_round_trip_preserves_heading_and_bold() {
    let source = br#"<!DOCTYPE html><html><body>
        <h1>Title</h1>
        <p>Hello <strong>world</strong></p>
    </body></html>"#;
    let doc = import(source).unwrap();
    let first = doc.sections[0].blocks[0].paragraph().unwrap();
    let h1 = doc.styles.find_style_by_name("Heading 1").unwrap().id;
    assert_eq!(first.style_id, Some(h1));

    let html = String::from_utf8(export(&doc).unwrap()).unwrap();
    assert!(
        html.contains("<h1>Title</h1>"),
        "expected plain Heading 1, got:\n{html}"
    );
    assert!(html.contains("<strong>world</strong>") || html.contains("<strong>world"));
    assert!(
        !html.contains("<h1><strong>Title</strong></h1>"),
        "<body> must not be treated as <b>"
    );
}

#[test]
fn u_f23_s3_html_export_headings() {
    let mut doc = Document::new();
    let h1 = doc.styles.find_style_by_name("Heading 1").unwrap().id;
    let quote = doc.styles.find_style_by_name("Quote").unwrap().id;

    let mut title = Paragraph::with_text("Title");
    title.style_id = Some(h1);
    let mut quoted = Paragraph::with_text("Quoted");
    quoted.style_id = Some(quote);
    let body = Paragraph::with_text("Body");

    doc.sections[0].blocks = vec![
        Block::Paragraph(title),
        Block::Paragraph(quoted),
        Block::Paragraph(body),
    ];

    let html = String::from_utf8(export(&doc).unwrap()).unwrap();
    assert!(
        html.contains("<h1>Title</h1>"),
        "Heading 1 must export as <h1>, got:\n{html}"
    );
    assert!(
        html.contains("<p>Quoted</p>"),
        "Quote must stay <p>, got:\n{html}"
    );
    assert!(
        html.contains("<p>Body</p>"),
        "Normal body must stay <p>, got:\n{html}"
    );
    assert_eq!(
        html.matches("<h1>").count(),
        1,
        "only Heading 1 should become <h1>"
    );
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
