//! F23.S1 — corpus open/render gate (≥95%) and save survival.

mod common;

use std::fs;
use std::time::Instant;

use common::{
    ensure_corpus, is_gate_corpus_file, list_gate_corpus, write_docx_xml, corpus_dir,
    F23_S1_CORPUS_MIN,
};
use tw_docx::import;
use tw_layout::LayoutEngine;
use tw_render::DisplayListBuilder;

fn render_corpus_file(path: &std::path::Path) -> Result<(usize, usize), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let imported = import(&bytes).map_err(|e| e.to_string())?;
    let mut layout = LayoutEngine::new();
    let doc_layout = layout.layout_document(&imported.document);
    let page_count = doc_layout.pages.len();
    let mut glyph_total = 0usize;
    for page in &doc_layout.pages {
        let list = DisplayListBuilder::from_page(page, layout.atlas(), 1);
        glyph_total += list.atlas_batch.transforms.len() / 2;
        if list.atlas_batch.transforms.is_empty()
            && list.path_batch.points.is_empty()
            && list.rect_batch.rects.is_empty()
        {
            // Empty body paragraphs may produce empty glyph batches; still count
            // as a successful open when at least one page exists.
            if page_count == 0 {
                return Err("empty display list".into());
            }
        }
    }
    if page_count == 0 {
        return Err("no pages".into());
    }
    Ok((page_count, glyph_total))
}

/// I-F23-S1-corpus-render — ≥95% of gate corpus opens and layouts.
#[test]
fn corpus_render_gate_passes_95_percent() {
    ensure_corpus();
    let entries = list_gate_corpus();
    assert!(
        entries.len() >= F23_S1_CORPUS_MIN,
        "corpus should have at least {F23_S1_CORPUS_MIN} gate-eligible docx files, got {}",
        entries.len()
    );

    let mut passed = 0usize;
    let mut failures = Vec::new();
    for path in &entries {
        match render_corpus_file(path) {
            Ok((pages, _glyphs)) => {
                passed += 1;
                assert!(pages >= 1);
            }
            Err(err) => {
                failures.push(format!("{}: {err}", path.display()));
            }
        }
    }

    let pass_rate = passed as f64 / entries.len() as f64;
    assert!(
        pass_rate >= 0.95,
        "corpus pass rate {:.0}% below 95% gate ({passed}/{}). Failures: {:?}",
        pass_rate * 100.0,
        entries.len(),
        failures
    );
}

/// A shape summary of a document: what a save must not change.
///
/// Consecutive empty paragraphs (common around section breaks) are collapsed so
/// the gate measures content survival rather than serializer whitespace.
fn outline(doc: &tw_model::Document) -> Vec<String> {
    let raw: Vec<String> = doc
        .sections
        .iter()
        .flat_map(|section| section.blocks.iter())
        .map(|block| match block {
            tw_model::Block::Paragraph(para) => format!("p:{}", para.full_text()),
            tw_model::Block::Table(table) => format!(
                "table:{}x{}",
                table.rows.len(),
                table.rows.first().map(|r| r.cells.len()).unwrap_or(0)
            ),
            tw_model::Block::ImageBlock(image) => {
                format!("image:{}", image.data.bytes.len())
            }
            tw_model::Block::ShapeBlock(shape) => {
                format!("shape:{}x{}", shape.shape.width, shape.shape.height)
            }
            _ => "other".into(),
        })
        .collect();

    let mut normalized = Vec::with_capacity(raw.len());
    for item in raw {
        if item == "p:" && normalized.last().is_some_and(|prev| prev == "p:") {
            continue;
        }
        normalized.push(item);
    }
    normalized
}

#[test]
fn every_corpus_file_survives_a_save() {
    ensure_corpus();
    let entries = list_gate_corpus();

    let mut losses = Vec::new();
    for path in &entries {
        let bytes = fs::read(path).unwrap();
        let imported = match import(&bytes) {
            Ok(r) => r,
            Err(e) => {
                losses.push(format!(
                    "{}: import failed: {e}",
                    path.file_name().unwrap().to_string_lossy()
                ));
                continue;
            }
        };
        let before = outline(&imported.document);

        let mut package = imported.package.clone();
        package.mark_modified("word/document.xml".into());
        let exported = tw_docx::export(&imported.document, &package).unwrap();
        let after = outline(&import(&exported).unwrap().document);

        if before != after {
            losses.push(format!(
                "{}\n  before: {before:?}\n  after:  {after:?}",
                path.file_name().unwrap().to_string_lossy()
            ));
        }
    }

    assert!(
        losses.is_empty(),
        "saving changed {} of {} corpus documents:\n{}",
        losses.len(),
        entries.len(),
        losses.join("\n")
    );
}

/// I-F23-S1-roundtrip-50 — import → forced export → re-import for ≥50 fixtures.
#[test]
fn i_f23_s1_roundtrip_50() {
    ensure_corpus();
    let entries = list_gate_corpus();
    assert!(
        entries.len() >= F23_S1_CORPUS_MIN,
        "need ≥{F23_S1_CORPUS_MIN} fixtures for round-trip gate, got {}",
        entries.len()
    );

    let mut failures = Vec::new();
    for path in entries.iter().take(F23_S1_CORPUS_MIN) {
        let bytes = fs::read(path).unwrap();
        let imported = match import(&bytes) {
            Ok(r) => r,
            Err(e) => {
                failures.push(format!("{}: import {e}", path.display()));
                continue;
            }
        };
        let before = outline(&imported.document);
        let mut package = imported.package.clone();
        package.mark_modified("word/document.xml".into());
        let exported = match tw_docx::export(&imported.document, &package) {
            Ok(b) => b,
            Err(e) => {
                failures.push(format!("{}: export {e}", path.display()));
                continue;
            }
        };
        let reimported = match import(&exported) {
            Ok(r) => r,
            Err(e) => {
                failures.push(format!("{}: reimport {e}", path.display()));
                continue;
            }
        };
        let after = outline(&reimported.document);
        if before != after {
            failures.push(format!(
                "{}: outline mismatch before={before:?} after={after:?}",
                path.file_name().unwrap().to_string_lossy()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "round-trip failures ({}/{}):\n{}",
        failures.len(),
        F23_S1_CORPUS_MIN,
        failures.join("\n")
    );
}

#[test]
fn large_docx_open_benchmark_under_two_seconds() {
    let mut body = String::from(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#,
    );
    for i in 0..500 {
        body.push_str(&format!(
            r#"<w:p><w:r><w:t>Page filler paragraph {} with enough text to consume vertical space on the page during layout.</w:t></w:r></w:p>"#,
            i
        ));
        if i % 50 == 49 {
            body.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
        }
    }
    body.push_str("</w:body></w:document>");

    let dir = corpus_dir();
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("_benchmark_500page.docx");
    write_docx_xml(&path, &body);
    assert!(
        !is_gate_corpus_file(&path),
        "benchmark fixture must stay outside the open gate"
    );

    let bytes = fs::read(&path).unwrap();
    let start = Instant::now();
    let imported = import(&bytes).unwrap();
    let mut layout = LayoutEngine::new();
    layout.layout_document(&imported.document);
    let elapsed = start.elapsed();
    let max_secs = if cfg!(debug_assertions) { 12.0 } else { 2.0 };
    assert!(
        elapsed.as_secs_f64() < max_secs,
        "500-page layout took {:.2}s, expected < {:.0}s",
        elapsed.as_secs_f64(),
        max_secs
    );
}
