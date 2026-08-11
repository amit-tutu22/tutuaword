//! F23.S4 — RTF list/table import + TXT export.

use tw_core::{
    export_document, import_document_bundle, DetectedFormat, FormatContext,
};
use tw_model::Block;

#[test]
fn i_f23_s4_rtf_detect_import() {
    let rtf = br#"{\rtf1\ansi Hello RTF\par}"#;
    let bundle = import_document_bundle(rtf, Some("memo.rtf")).unwrap();
    assert_eq!(bundle.source_format, DetectedFormat::Rtf);
    let text = bundle.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .full_text();
    assert!(text.contains("Hello RTF"));
}

#[test]
fn u_f23_s4_txt_export_roundtrip() {
    let src = "Line one\nLine two\n";
    let bundle = import_document_bundle(src.as_bytes(), Some("notes.txt")).unwrap();
    assert_eq!(bundle.source_format, DetectedFormat::PlainText);

    let mut ctx = FormatContext::from_bundle(bundle.clone(), Some("notes.txt".into()));
    ctx.save_format = DetectedFormat::PlainText;
    let exported = export_document(&bundle.document, &ctx).unwrap();

    // Must be UTF-8 lines, not a TWDOC ZIP.
    assert!(!exported.starts_with(b"PK"), "txt export must not be ZIP/twdoc");
    let text = String::from_utf8(exported).unwrap();
    assert!(text.contains("Line one"));
    assert!(text.contains("Line two"));

    let again = import_document_bundle(text.as_bytes(), Some("notes.txt")).unwrap();
    let lines: Vec<_> = again
        .document
        .sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph().map(|p| p.full_text()))
        .collect();
    assert!(lines.iter().any(|l| l.contains("Line one")));
    assert!(lines.iter().any(|l| l.contains("Line two")));
}

#[test]
fn u_f23_s4_rtf_table_via_core() {
    let rtf = br#"{\rtf1\ansi
\trowd\cellx2000\cellx4000
\intbl A1\cell B1\cell\row
\trowd\cellx2000\cellx4000
\intbl A2\cell B2\cell\row
\par
}"#;
    let bundle = import_document_bundle(rtf, Some("grid.rtf")).unwrap();
    assert!(bundle
        .document
        .sections[0]
        .blocks
        .iter()
        .any(|b| matches!(b, Block::Table(_))));
}
