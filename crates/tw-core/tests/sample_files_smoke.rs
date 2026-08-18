//! Smoke-open sample DOCX files from disk and exercise core editor features.
//!
//! Default directory: `$HOME/Downloads` (or `$SAMPLE_DOCX_DIR`).
//! Skips cleanly when fixtures are missing so CI stays green.
//!
//! Operator docs: `docs/sample-files-smoke.md`.

use std::path::{Path, PathBuf};
use std::time::Instant;

use tw_core::{
    document_plain_text, export_document, import_document_bundle, DetectedFormat, FormatContext,
    SyncSession,
};
use tw_edit::{Command, EditSession};
use tw_model::{revision_run_ids, Block, RunContent};
use tw_render::DisplayListBuilder;

struct Case {
    file: &'static str,
    /// Soft expectations — failures are reported, not always hard-asserted.
    expect_tables: bool,
    expect_images: bool,
    expect_lists: bool,
    expect_revisions: bool,
    expect_multi_column: bool,
    min_pages: u32,
    max_pages: Option<u32>,
    min_text_chars: usize,
}

const CASES: &[Case] = &[
    Case {
        file: "sample-files.com-basic-text.docx",
        expect_tables: false,
        expect_images: false,
        expect_lists: false,
        expect_revisions: false,
        expect_multi_column: false,
        min_pages: 1,
        max_pages: None,
        min_text_chars: 20,
    },
    Case {
        file: "sample-files.com-formatted-report.docx",
        expect_tables: true,
        expect_images: false,
        expect_lists: false,
        expect_revisions: false,
        expect_multi_column: false,
        min_pages: 1,
        max_pages: None,
        min_text_chars: 20,
    },
    Case {
        file: "sample-files.com-image-document.docx",
        expect_tables: false,
        expect_images: true,
        expect_lists: false,
        expect_revisions: false,
        expect_multi_column: false,
        min_pages: 1,
        max_pages: None,
        min_text_chars: 0,
    },
    Case {
        file: "sample-files.com-table-document.docx",
        expect_tables: true,
        expect_images: false,
        expect_lists: false,
        expect_revisions: false,
        expect_multi_column: false,
        min_pages: 1,
        max_pages: None,
        min_text_chars: 1,
    },
    Case {
        file: "sample-files.com-template.docx",
        expect_tables: true,
        expect_images: false,
        expect_lists: false,
        expect_revisions: false,
        expect_multi_column: false,
        min_pages: 1,
        max_pages: None,
        min_text_chars: 0,
    },
    Case {
        file: "sample-files.com-lists.docx",
        expect_tables: false,
        expect_images: false,
        expect_lists: true,
        expect_revisions: false,
        expect_multi_column: false,
        min_pages: 1,
        max_pages: None,
        min_text_chars: 10,
    },
    Case {
        file: "sample-files.com-tracked-changes.docx",
        expect_tables: false,
        expect_images: false,
        expect_lists: false,
        expect_revisions: true,
        expect_multi_column: false,
        min_pages: 1,
        max_pages: None,
        min_text_chars: 1,
    },
    Case {
        file: "sample-files.com-multi-column.docx",
        expect_tables: false,
        expect_images: false,
        expect_lists: false,
        expect_revisions: false,
        expect_multi_column: true,
        min_pages: 1,
        max_pages: Some(8),
        min_text_chars: 10,
    },
    Case {
        file: "sample-files.com-large-document.docx",
        expect_tables: false,
        expect_images: false,
        expect_lists: false,
        expect_revisions: false,
        expect_multi_column: false,
        min_pages: 2,
        max_pages: None,
        min_text_chars: 200,
    },
];

fn sample_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SAMPLE_DOCX_DIR") {
        return PathBuf::from(dir);
    }
    dirs_next_home()
        .map(|h| h.join("Downloads"))
        .unwrap_or_else(|| PathBuf::from("/Users/amitkumar/Downloads"))
}

fn dirs_next_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn count_tables(doc: &tw_model::Document) -> usize {
    doc.sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter(|b| matches!(b, Block::Table(_)))
        .count()
}

fn count_images(doc: &tw_model::Document) -> usize {
    let mut n = 0;
    for section in &doc.sections {
        for block in &section.blocks {
            match block {
                Block::ImageBlock(_) => n += 1,
                Block::Paragraph(p) => {
                    n += p
                        .runs
                        .iter()
                        .filter(|r| matches!(r.content, RunContent::InlineImage(_)))
                        .count();
                }
                Block::Table(t) => {
                    for row in &t.rows {
                        for cell in &row.cells {
                            for b in &cell.blocks {
                                if matches!(b, Block::ImageBlock(_)) {
                                    n += 1;
                                }
                                if let Block::Paragraph(p) = b {
                                    n += p
                                        .runs
                                        .iter()
                                        .filter(|r| {
                                            matches!(r.content, RunContent::InlineImage(_))
                                        })
                                        .count();
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    n
}

fn count_list_paras(doc: &tw_model::Document) -> usize {
    doc.sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .filter(|p| p.format.numbering.is_some())
        .count()
}

/// Word often applies lists via `pStyle` (ListBullet / ListNumber) without a
/// direct `w:numPr` on the paragraph. Count those as list content too.
fn count_list_style_paras(doc: &tw_model::Document) -> usize {
    doc.sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .filter(|p| {
            let Some(style_id) = p.style_id else {
                return false;
            };
            let ooxml = doc.styles.ooxml_id_for(style_id).unwrap_or_default();
            let name = doc
                .styles
                .paragraph_styles
                .get(&style_id)
                .map(|s| s.name.as_str())
                .unwrap_or("");
            let blob = format!("{ooxml} {name}").to_ascii_lowercase();
            blob.contains("listbullet")
                || blob.contains("listnumber")
                || blob.contains("list bullet")
                || blob.contains("list number")
        })
        .count()
}

fn max_columns(doc: &tw_model::Document) -> u32 {
    doc.sections
        .iter()
        .map(|s| s.format.columns.count.max(1))
        .max()
        .unwrap_or(1)
}

struct Report {
    file: String,
    ok: bool,
    notes: Vec<String>,
    elapsed_ms: u128,
}

fn smoke_one(path: &Path, case: &Case) -> Report {
    let started = Instant::now();
    let mut notes = Vec::new();
    let mut ok = true;

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return Report {
                file: case.file.into(),
                ok: false,
                notes: vec![format!("read failed: {e}")],
                elapsed_ms: started.elapsed().as_millis(),
            };
        }
    };
    notes.push(format!("bytes={}", bytes.len()));

    let bundle = match import_document_bundle(&bytes, Some(case.file)) {
        Ok(b) => b,
        Err(e) => {
            return Report {
                file: case.file.into(),
                ok: false,
                notes: vec![format!("import failed: {e}")],
                elapsed_ms: started.elapsed().as_millis(),
            };
        }
    };
    notes.push("import=ok".into());

    let doc = &bundle.document;
    let tables = count_tables(doc);
    let images = count_images(doc);
    let lists = count_list_paras(doc);
    let list_styles = count_list_style_paras(doc);
    let numbering_defs = doc.settings.numbering.definitions.len();
    let revisions = revision_run_ids(doc).len();
    let columns = max_columns(doc);
    let text = document_plain_text(doc);
    notes.push(format!(
        "tables={tables} images={images} lists={lists} list_styles={list_styles} numbering_defs={numbering_defs} revisions={revisions} columns={columns} text_chars={}",
        text.chars().count()
    ));

    if case.expect_tables && tables == 0 {
        ok = false;
        notes.push("FAIL: expected tables".into());
    }
    if case.expect_images && images == 0 {
        ok = false;
        notes.push("FAIL: expected images".into());
    }
    if case.expect_lists && lists == 0 && list_styles == 0 {
        ok = false;
        notes.push("FAIL: expected list paragraphs or list styles".into());
    } else if case.expect_lists && lists == 0 && list_styles > 0 {
        notes.push(
            "WARN: lists are style-linked (ListBullet/ListNumber) without para numPr — open/render OK; promote/demote may need style→numbering resolve"
                .into(),
        );
    }
    if case.expect_revisions && revisions == 0 {
        ok = false;
        notes.push("FAIL: expected tracked-change revisions".into());
    }
    if case.expect_multi_column && columns < 2 {
        ok = false;
        notes.push(format!("FAIL: expected multi-column, got {columns}"));
    }
    if text.chars().count() < case.min_text_chars {
        ok = false;
        notes.push(format!(
            "FAIL: text too short ({} < {})",
            text.chars().count(),
            case.min_text_chars
        ));
    }

    let mut session = SyncSession::new();
    for font in &bundle.embedded_fonts {
        let _ = session.layout.register_face(&font.spec, font.data.clone());
    }
    session.edit = EditSession::from_document(doc.clone());
    session.relayout(None);

    let pages = session.page_count();
    notes.push(format!("pages={pages}"));
    if pages < case.min_pages {
        ok = false;
        notes.push(format!("FAIL: pages {pages} < {}", case.min_pages));
    }
    if let Some(max) = case.max_pages {
        if pages > max {
            ok = false;
            notes.push(format!("FAIL: pages {pages} > {max}"));
        }
    }

    let dl = session.display_list_bytes();
    if dl.is_empty() {
        ok = false;
        notes.push("FAIL: empty display list".into());
    } else if let Some(decoded) = DisplayListBuilder::from_bytes(&dl) {
        notes.push(format!(
            "display_list=ok page={}x{}",
            decoded.page_width, decoded.page_height
        ));
        if decoded.page_width <= 0.0 || decoded.page_height <= 0.0 {
            ok = false;
            notes.push("FAIL: invalid page size".into());
        }
    } else {
        ok = false;
        notes.push("FAIL: display list decode".into());
    }

    // Hit-test / caret placement near top-left content.
    let hit = session
        .layout
        .line_map(0)
        .and_then(|map| map.hit_test(90.0, 90.0));
    if let Some(h) = &hit {
        notes.push(format!("hit_test=ok run={} off={}", h.run_id, h.char_offset));
    } else if case.min_text_chars > 0 {
        ok = false;
        notes.push("FAIL: hit_test returned None".into());
    } else {
        notes.push("WARN: hit_test None (may be image-only)".into());
    }

    // Basic edit: insert a marker then verify plaintext contains it.
    let edit_target = hit
        .map(|h| (h.run_id, h.char_offset))
        .or_else(|| {
            session
                .edit
                .document
                .paragraph_at(0, 0)
                .and_then(|p| p.runs.first().map(|r| (r.id, 0usize)))
        });
    if let Some((run_id, offset)) = edit_target {
        session.apply(Command::InsertText {
            run_id,
            offset,
            text: "⟦SMOKE⟧".into(),
        });
        session.relayout(None);
        let after = document_plain_text(&session.edit.document);
        if after.contains("⟦SMOKE⟧") {
            notes.push("edit_insert=ok".into());
        } else {
            ok = false;
            notes.push("FAIL: insert text not visible in plaintext".into());
        }
    } else {
        notes.push("WARN: no editable run for insert smoke".into());
    }

    // DOCX export round-trip (structure only — reopen must succeed).
    let mut ctx = FormatContext::from_bundle(
        tw_core::ImportBundle {
            document: session.edit.document.clone(),
            source_format: bundle.source_format,
            docx_package: bundle.docx_package.clone(),
            odt_package: None,
            embedded_fonts: bundle.embedded_fonts.clone(),
        },
        Some(case.file.to_string()),
    );
    ctx.save_format = DetectedFormat::Docx;
    match export_document(&session.edit.document, &ctx) {
        Ok(exported) => match import_document_bundle(&exported, Some("roundtrip.docx")) {
            Ok(rt) => {
                notes.push(format!(
                    "roundtrip=ok text_chars={}",
                    document_plain_text(&rt.document).chars().count()
                ));
            }
            Err(e) => {
                ok = false;
                notes.push(format!("FAIL: roundtrip re-import: {e}"));
            }
        },
        Err(e) => {
            ok = false;
            notes.push(format!("FAIL: export docx: {e}"));
        }
    }

    Report {
        file: case.file.into(),
        ok,
        notes,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

#[test]
fn sample_files_open_and_feature_smoke() {
    let dir = sample_dir();
    let present: Vec<_> = CASES
        .iter()
        .filter(|c| dir.join(c.file).is_file())
        .collect();
    if present.is_empty() {
        eprintln!(
            "skip sample_files_smoke: no fixtures in {} (set SAMPLE_DOCX_DIR)",
            dir.display()
        );
        return;
    }

    let mut reports = Vec::new();
    for case in present {
        let path = dir.join(case.file);
        eprintln!("\n=== {} ===", case.file);
        let report = smoke_one(&path, case);
        for n in &report.notes {
            eprintln!("  {n}");
        }
        eprintln!(
            "  => {} ({} ms)",
            if report.ok { "PASS" } else { "FAIL" },
            report.elapsed_ms
        );
        reports.push(report);
    }

    let failed: Vec<_> = reports.iter().filter(|r| !r.ok).collect();
    assert!(
        failed.is_empty(),
        "{} sample file(s) failed:\n{}",
        failed.len(),
        failed
            .iter()
            .map(|r| format!("- {}: {}", r.file, r.notes.join("; ")))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
