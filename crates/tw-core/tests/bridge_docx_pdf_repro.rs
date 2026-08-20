use std::path::PathBuf;

use tw_docx::import;
use tw_pdf::{DisplayListPdfExporter, PdfExportOptions, PdfExporter};

#[test]
fn bridge_docx_shaded_callout_text_visible() {
    let path = PathBuf::from(
        "/Users/amitkumar/Downloads/Flutter_Rust_Communication_Bridge_Architecture.docx",
    );
    if !path.exists() {
        eprintln!("skip: missing {}", path.display());
        return;
    }
    let bytes = std::fs::read(&path).unwrap();
    let imported = import(&bytes).expect("import");
    let pdf = DisplayListPdfExporter
        .export(&imported.document, &PdfExportOptions::default())
        .expect("pdf");
    let s = String::from_utf8_lossy(&pdf);
    assert!(
        s.contains("Architecture decision"),
        "shaded callout cell text must be visible in PDF"
    );
    assert!(
        s.contains("never exposes engine internals")
            || s.contains("never exposes engine inter"),
        "callout body text missing"
    );
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tmp-bridge-fixed.pdf");
    std::fs::write(&out, &pdf).unwrap();
}
