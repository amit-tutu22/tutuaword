//! F08.S3 — export `w:titlePg` and `w:evenAndOddHeaders`.

#[path = "f08_docx_helpers/mod.rs"]
mod helpers;

use helpers::{document_xml, settings_xml};
use tw_docx::{export_docx, import_docx, DocxPackage};
use tw_model::Document;

#[test]
fn u_f08_s3_export_title_pg_and_even_and_odd_headers() {
    let mut doc = Document::with_paragraph("Body");
    doc.sections[0].format.different_first_page = true;
    doc.settings.even_and_odd_headers = true;

    let package = DocxPackage::minimal();
    let bytes = export_docx(&doc, &package).unwrap();

    let imported = import_docx(&bytes).unwrap().document;
    assert!(imported.sections[0].format.different_first_page);
    assert!(imported.settings.even_and_odd_headers);

    assert!(document_xml(&bytes).contains("<w:titlePg"));
    assert!(settings_xml(&bytes).contains("<w:evenAndOddHeaders"));
}
