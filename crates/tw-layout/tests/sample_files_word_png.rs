//! Word PNG baseline comparison for sample-files.com fixtures.
//!
//! Writes engine PNGs under `tests/golden/sample-files/`. When a Word baseline
//! PNG exists beside it (`*_word.png`), asserts pixel diff < 2%.

use std::fs;
use std::path::PathBuf;

use tw_core::import_document_bundle;
use tw_layout::{LayoutBox, LayoutEngine};
use tw_render::{pixel_diff_ratio, rasterize_page};

const FIXTURES: &[&str] = &[
    "sample-files.com-formatted-report.docx",
    "sample-files.com-lists.docx",
    "sample-files.com-table-document.docx",
];

fn sample_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SAMPLE_DOCX_DIR") {
        return PathBuf::from(dir);
    }
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Downloads"))
        .unwrap_or_else(|| PathBuf::from("/Users/amitkumar/Downloads"))
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/sample-files")
}

fn stem(file: &str) -> &str {
    file.strip_suffix(".docx").unwrap_or(file)
}

#[test]
fn sample_files_engine_png_and_word_diff() {
    let sample_dir = sample_dir();
    if !sample_dir.exists() {
        eprintln!("skip: sample dir missing ({})", sample_dir.display());
        return;
    }
    fs::create_dir_all(golden_dir()).expect("golden dir");

    for file in FIXTURES {
        let path = sample_dir.join(file);
        if !path.exists() {
            eprintln!("skip: missing {}", path.display());
            continue;
        }
        let bytes = fs::read(&path).expect("read fixture");
        let bundle = import_document_bundle(&bytes, Some(file)).expect("import");

        // Fonts the document embeds must be registered before layout, or the
        // baseline is rasterized with fallback metrics Word never used.
        let mut engine = LayoutEngine::new();
        for font in &bundle.embedded_fonts {
            let _ = engine.register_face(&font.spec, font.data.clone());
        }
        let layout = engine.layout_document(&bundle.document);
        let page = layout.pages.first().expect("page 0");
        let has_lines = page
            .boxes
            .iter()
            .any(|b| matches!(b, LayoutBox::TextLine(_) | LayoutBox::Table(_)));
        assert!(has_lines, "{file} should produce text lines on page 0");

        let png = rasterize_page(page, engine.atlas());
        let engine_path = golden_dir().join(format!("{}_engine.png", stem(file)));
        fs::write(&engine_path, &png).expect("write engine png");

        let word_path = golden_dir().join(format!("{}_word.png", stem(file)));
        if word_path.exists() {
            let word = fs::read(&word_path).expect("read word baseline");
            let diff = pixel_diff_ratio(&png, &word);
            assert!(
                diff < 0.02,
                "{} pixel diff {:.2}% exceeds 2%",
                file,
                diff * 100.0
            );
        } else {
            eprintln!(
                "skip word compare: no baseline at {}",
                word_path.display()
            );
        }
    }
}
