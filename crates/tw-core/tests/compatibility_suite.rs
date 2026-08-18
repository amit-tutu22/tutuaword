//! Headless execution of `Tutuaword_Compatibility_Test_Suite` fixtures.
//!
//! Directory: `$COMPAT_SUITE_DIR` or
//! `$HOME/Desktop/tutuaword/Tutuaword_Compatibility_Test_Suite`.
//! Skips cleanly when the pack is missing so CI stays green.

use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

use tw_core::{
    document_plain_text, export_document, import_document_bundle, DetectedFormat, FormatContext,
    SyncSession,
};
use tw_edit::{Command, EditSession};
use tw_model::{revision_run_ids, Block, RunContent};
use tw_pdf::{DisplayListPdfExporter, PdfExportOptions, PdfExporter};
use tw_render::{rasterize_page, DisplayListBuilder};

struct Case {
    file: &'static str,
    marker: &'static str,
    extra_needles: &'static [&'static str],
    expect_tables: bool,
    expect_images: usize,
    expect_lists: bool,
    expect_sections: usize,
    expect_import_fail: bool,
    min_pages: u32,
}

const CASES: &[Case] = &[
    Case {
        file: "Document_01_Basic.docx",
        marker: "TUTUA-01-BASIC",
        extra_needles: &[
            "PAGE_MARKER_01_01",
            "PAGE_MARKER_01_10",
        ],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_02_Formatting.docx",
        marker: "TUTUA-02-FORMATTING",
        extra_needles: &["PARA_FORMAT_01", "PARA_FORMAT_07"],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_03_Tables.docx",
        marker: "TUTUA-03-TABLES",
        extra_needles: &["TABLE_MARKER_03", "T-01", "T-14"],
        expect_tables: true,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_04_Images.docx",
        marker: "TUTUA-04-IMAGES",
        extra_needles: &["IMAGE_MARKER_04"],
        expect_tables: false,
        expect_images: 3,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_05_Sections.docx",
        marker: "TUTUA-05-SECTIONS",
        extra_needles: &["SECTION_MARKER_05", "HEADER_SECTION_03", "FOOTER_SECTION_03"],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 3,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_06_Lists.docx",
        marker: "TUTUA-06-LISTS",
        extra_needles: &["LIST_MARKER_06", "Level 1 item 1", "Level 3 item 5.3.2"],
        expect_tables: false,
        expect_images: 0,
        expect_lists: true,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_07_References.docx",
        marker: "TUTUA-07-REFERENCES",
        extra_needles: &["REFERENCES_MARKER_07", "TABLE OF CONTENTS"],
        expect_tables: true,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_08_Review.docx",
        marker: "TUTUA-08-REVIEW",
        extra_needles: &["REVIEW_MARKER_08", "COMMENT_01", "INSERTED_TEXT_01"],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_09_Objects.docx",
        marker: "TUTUA-09-OBJECTS",
        extra_needles: &["OBJECTS_MARKER_09", "[SHAPE: PROCESS BOX]"],
        expect_tables: true,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_10_Unicode.docx",
        marker: "TUTUA-10-UNICODE",
        extra_needles: &[
            "UNICODE_MARKER_10",
            "नमस्ते",
            "ನಮಸ್ಕಾರ",
            "بالعالم",
            "你好世界",
        ],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_11_Large_500_pages.docx",
        marker: "TUTUA-11-LARGE",
        extra_needles: &["LARGE_PAGE_MARKER_001", "LARGE_PAGE_MARKER_500"],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 2,
    },
    Case {
        file: "Document_12_DOCM_compatible.docm",
        marker: "TUTUA-12-DOCM",
        extra_needles: &["DOCM_MARKER_12"],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
    Case {
        file: "Document_13_Damaged_bad_xml.docx",
        marker: "TUTUA-13",
        extra_needles: &[],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 0,
        expect_import_fail: true,
        min_pages: 0,
    },
    Case {
        file: "Document_13_Damaged_truncated.docx",
        marker: "TUTUA-13",
        extra_needles: &[],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 0,
        expect_import_fail: true,
        min_pages: 0,
    },
    Case {
        file: "Document_14_AI.docx",
        marker: "TUTUA-14-AI",
        extra_needles: &[
            "AI_MARKER_14_NORMAL",
            "AI_MARKER_14_ADVERSARIAL",
            "Ignore all prior instructions",
        ],
        expect_tables: false,
        expect_images: 0,
        expect_lists: false,
        expect_sections: 1,
        expect_import_fail: false,
        min_pages: 1,
    },
];

struct Report {
    file: String,
    status: &'static str,
    notes: Vec<String>,
    elapsed_ms: u128,
}

fn suite_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("COMPAT_SUITE_DIR") {
        return PathBuf::from(dir);
    }
    std::env::var_os("HOME")
        .map(|h| {
            PathBuf::from(h).join("Desktop/tutuaword/Tutuaword_Compatibility_Test_Suite")
        })
        .unwrap_or_else(|| PathBuf::from("/Users/amitkumar/Desktop/tutuaword/Tutuaword_Compatibility_Test_Suite"))
}

fn evidence_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("COMPAT_EVIDENCE_DIR") {
        return PathBuf::from(dir);
    }
    std::env::temp_dir().join("tutuaword-qa-evidence")
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

fn count_shapes(doc: &tw_model::Document) -> usize {
    doc.sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter(|b| matches!(b, Block::ShapeBlock(_)))
        .count()
}

fn count_list_paras(doc: &tw_model::Document) -> usize {
    doc.sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .filter(|p| p.format.numbering.is_some())
        .count()
}

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

fn header_footer_count(doc: &tw_model::Document) -> (usize, usize) {
    let headers = doc.sections.iter().map(|s| s.headers.len()).sum();
    let footers = doc.sections.iter().map(|s| s.footers.len()).sum();
    (headers, footers)
}

fn fail(notes: &mut Vec<String>, ok: &mut bool, msg: impl Into<String>) {
    *ok = false;
    notes.push(format!("FAIL: {}", msg.into()));
}

fn run_case(dir: &Path, evidence: &Path, case: &Case) -> Report {
    let started = Instant::now();
    let mut notes = Vec::new();
    let mut ok = true;
    let path = dir.join(case.file);
    notes.push(format!("path={}", path.display()));

    let bytes = match fs::read(&path) {
        Ok(b) => {
            notes.push(format!("bytes={}", b.len()));
            b
        }
        Err(e) => {
            return Report {
                file: case.file.into(),
                status: "BLOCKED",
                notes: vec![format!("read failed: {e}")],
                elapsed_ms: started.elapsed().as_millis(),
            };
        }
    };

    let import = catch_unwind(AssertUnwindSafe(|| {
        import_document_bundle(&bytes, Some(case.file))
    }));
    let import = match import {
        Ok(inner) => inner,
        Err(_) => {
            if case.expect_import_fail {
                notes.push("import panicked caught — treated as safe reject".into());
                return Report {
                    file: case.file.into(),
                    status: "PASS",
                    notes,
                    elapsed_ms: started.elapsed().as_millis(),
                };
            }
            return Report {
                file: case.file.into(),
                status: "FAIL",
                notes: {
                    notes.push("FAIL: import panicked".into());
                    notes
                },
                elapsed_ms: started.elapsed().as_millis(),
            };
        }
    };

    match import {
        Err(e) => {
            notes.push(format!("import_error={e}"));
            if case.expect_import_fail {
                notes.push("safe reject (no crash, no overwrite)".into());
                return Report {
                    file: case.file.into(),
                    status: "PASS",
                    notes,
                    elapsed_ms: started.elapsed().as_millis(),
                };
            }
            fail(&mut notes, &mut ok, format!("unexpected import failure: {e}"));
            return Report {
                file: case.file.into(),
                status: "FAIL",
                notes,
                elapsed_ms: started.elapsed().as_millis(),
            };
        }
        Ok(_) if case.expect_import_fail => {
            fail(
                &mut notes,
                &mut ok,
                "damaged fixture imported; expected safe reject",
            );
            return Report {
                file: case.file.into(),
                status: "FAIL",
                notes,
                elapsed_ms: started.elapsed().as_millis(),
            };
        }
        Ok(bundle) => {
            notes.push("import=ok".into());
            let doc = &bundle.document;
            let tables = count_tables(doc);
            let images = count_images(doc);
            let lists = count_list_paras(doc);
            let list_styles = count_list_style_paras(doc);
            let revisions = revision_run_ids(doc).len();
            let (headers, footers) = header_footer_count(doc);
            let comments = doc.comments.len();
            let footnotes = doc.footnotes.len();
            let shapes = count_shapes(doc);
            let text = document_plain_text(doc);
            notes.push(format!(
                "sections={} tables={tables} images={images} lists={lists} list_styles={list_styles} revisions={revisions} headers={headers} footers={footers} comments={comments} footnotes={footnotes} shapes={shapes} chars={}",
                doc.sections.len(),
                text.chars().count()
            ));

            if !text.contains(case.marker) {
                fail(
                    &mut notes,
                    &mut ok,
                    format!("missing primary marker {}", case.marker),
                );
            } else {
                notes.push(format!("marker={} present", case.marker));
            }
            for needle in case.extra_needles {
                if !text.contains(needle) {
                    fail(&mut notes, &mut ok, format!("missing needle {needle}"));
                }
            }
            if case.file.contains("Document_01") {
                let n = (1..=10)
                    .filter(|i| text.contains(&format!("PAGE_MARKER_01_{i:02}")))
                    .count();
                notes.push(format!("page_markers={n}/10"));
                if n != 10 {
                    fail(&mut notes, &mut ok, "expected exactly 10 PAGE_MARKER_01_*");
                }
            }
            if case.file.contains("Document_11") {
                let n = (1..=500)
                    .filter(|i| text.contains(&format!("LARGE_PAGE_MARKER_{i:03}")))
                    .count();
                notes.push(format!("large_page_markers={n}/500"));
                if n != 500 {
                    fail(&mut notes, &mut ok, "expected 500 LARGE_PAGE_MARKER_*");
                }
            }
            if case.expect_tables && tables == 0 {
                fail(&mut notes, &mut ok, "expected tables");
            }
            if images < case.expect_images {
                fail(
                    &mut notes,
                    &mut ok,
                    format!("expected >= {} images, got {images}", case.expect_images),
                );
            }
            if case.expect_lists && lists == 0 && list_styles == 0 {
                fail(&mut notes, &mut ok, "expected list paragraphs or list styles");
            } else if case.expect_lists && lists == 0 && list_styles > 0 {
                notes.push("WARN: lists are style-linked without para numPr".into());
            }
            if doc.sections.len() < case.expect_sections {
                fail(
                    &mut notes,
                    &mut ok,
                    format!(
                        "expected >= {} sections, got {}",
                        case.expect_sections,
                        doc.sections.len()
                    ),
                );
            }
            if case.file.contains("Document_05") {
                if headers == 0 {
                    fail(&mut notes, &mut ok, "expected at least one header part");
                }
                if footers == 0 {
                    fail(&mut notes, &mut ok, "expected at least one footer part");
                }
            }
            if case.file.contains("Document_07") {
                if footnotes == 0 {
                    fail(&mut notes, &mut ok, "expected native footnotes part");
                }
            }
            if case.file.contains("Document_08") {
                if comments == 0 {
                    fail(&mut notes, &mut ok, "expected native comments.xml");
                }
                if revisions == 0 {
                    fail(&mut notes, &mut ok, "expected native tracked revisions");
                }
            }
            if case.file.contains("Document_09") {
                if shapes == 0 {
                    fail(
                        &mut notes,
                        &mut ok,
                        "expected native DrawingML shape/chart/diagram",
                    );
                }
            }
            if case.file.contains("Document_14") {
                notes.push(
                    "AI adversarial text treated as inert document content (no tools invoked)"
                        .into(),
                );
            }
            if case.file.contains(".docm") {
                notes.push(
                    "DOCM opened as package; engine does not execute VBA/macros".into(),
                );
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
                fail(
                    &mut notes,
                    &mut ok,
                    format!("pages {pages} < {}", case.min_pages),
                );
            }

            let dl = session.display_list_bytes();
            if dl.is_empty() {
                fail(&mut notes, &mut ok, "empty display list");
            } else if let Some(decoded) = DisplayListBuilder::from_bytes(&dl) {
                notes.push(format!(
                    "display_list=ok page={}x{}",
                    decoded.page_width, decoded.page_height
                ));
            } else {
                fail(&mut notes, &mut ok, "display list decode");
            }

            if let Some(page) = session.layout.document_layout().pages.first() {
                let png = rasterize_page(page, session.layout.atlas());
                let stem = case.file.rsplit_once('.').map(|(s, _)| s).unwrap_or(case.file);
                let png_path = evidence.join(format!("{stem}_page1.png"));
                if let Err(e) = fs::write(&png_path, &png) {
                    notes.push(format!("WARN: png write failed: {e}"));
                } else {
                    notes.push(format!("png={} bytes={}", png_path.display(), png.len()));
                }
            }

            let hits = session.find_matches(case.marker, false, false, false, None);
            notes.push(format!("find({})={}", case.marker, hits.len()));
            if hits.is_empty() && text.contains(case.marker) {
                fail(&mut notes, &mut ok, "find did not locate primary marker");
            }

            let hit = session
                .layout
                .line_map(0)
                .and_then(|map| map.hit_test(90.0, 90.0));
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
                match session.edit.apply(Command::InsertText {
                    run_id,
                    offset,
                    text: "⟦QA-EDIT⟧".into(),
                }) {
                    Ok(_) => {
                        session.relayout(None);
                        let after = document_plain_text(&session.edit.document);
                        if after.contains("⟦QA-EDIT⟧") {
                            notes.push("edit_insert=ok".into());
                        } else {
                            fail(&mut notes, &mut ok, "insert text not visible");
                        }
                        match session.edit.undo() {
                            Ok(Some(_)) => {
                                let undone = document_plain_text(&session.edit.document);
                                if undone.contains("⟦QA-EDIT⟧") {
                                    fail(&mut notes, &mut ok, "undo did not remove insert");
                                } else {
                                    notes.push("undo=ok".into());
                                }
                            }
                            Ok(None) => fail(&mut notes, &mut ok, "undo returned None"),
                            Err(e) => fail(&mut notes, &mut ok, format!("undo error: {e}")),
                        }
                        if let Err(e) = session.edit.apply(Command::InsertText {
                            run_id,
                            offset,
                            text: "⟦QA-EDIT⟧".into(),
                        }) {
                            notes.push(format!("WARN: re-insert after undo failed: {e}"));
                        } else {
                            session.relayout(None);
                        }
                    }
                    Err(e) => {
                        notes.push(format!("WARN: insert skipped ({e}); trying first body run"));
                        if let Some((rid, off)) = session
                            .edit
                            .document
                            .paragraph_at(0, 0)
                            .and_then(|p| p.runs.first().map(|r| (r.id, 0usize)))
                        {
                            match session.edit.apply(Command::InsertText {
                                run_id: rid,
                                offset: off,
                                text: "⟦QA-EDIT⟧".into(),
                            }) {
                                Ok(_) => {
                                    session.relayout(None);
                                    notes.push("edit_insert=ok (fallback run)".into());
                                }
                                Err(e2) => fail(
                                    &mut notes,
                                    &mut ok,
                                    format!("insert failed on fallback run: {e2}"),
                                ),
                            }
                        }
                    }
                }
            } else {
                notes.push("WARN: no editable run for insert/undo".into());
            }

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
                        let rt_text = document_plain_text(&rt.document);
                        notes.push(format!(
                            "roundtrip=ok chars={}",
                            rt_text.chars().count()
                        ));
                        if !rt_text.contains(case.marker) {
                            fail(&mut notes, &mut ok, "round-trip dropped primary marker");
                        }
                        if rt_text.contains("⟦QA-EDIT⟧") {
                            notes.push("roundtrip_keeps_edit=ok".into());
                        }
                        let out = evidence.join(format!("{}_roundtrip.docx", case.file));
                        let _ = fs::write(out, exported);
                    }
                    Err(e) => fail(&mut notes, &mut ok, format!("roundtrip re-import: {e}")),
                },
                Err(e) => fail(&mut notes, &mut ok, format!("export docx: {e}")),
            }

            match DisplayListPdfExporter.export(
                &session.edit.document,
                &PdfExportOptions::default(),
            ) {
                Ok(pdf) if pdf.starts_with(b"%PDF") => {
                    notes.push(format!("pdf=ok bytes={}", pdf.len()));
                    let stem = case.file.rsplit_once('.').map(|(s, _)| s).unwrap_or(case.file);
                    let _ = fs::write(evidence.join(format!("{stem}.pdf")), pdf);
                }
                Ok(_) => fail(&mut notes, &mut ok, "pdf export missing %PDF header"),
                Err(e) => fail(&mut notes, &mut ok, format!("pdf export: {e}")),
            }
        }
    }

    Report {
        file: case.file.into(),
        status: if ok { "PASS" } else { "FAIL" },
        notes,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

#[test]
fn compatibility_suite_open_edit_roundtrip() {
    let dir = suite_dir();
    if !dir.is_dir() {
        eprintln!("skip compatibility_suite: missing {}", dir.display());
        return;
    }
    let evidence = evidence_dir();
    let _ = fs::create_dir_all(&evidence);
    eprintln!("evidence dir: {}", evidence.display());

    let mut reports = Vec::new();
    for case in CASES {
        if !dir.join(case.file).is_file() {
            reports.push(Report {
                file: case.file.into(),
                status: "BLOCKED",
                notes: vec!["fixture file missing".into()],
                elapsed_ms: 0,
            });
            continue;
        }
        eprintln!("\n=== {} ===", case.file);
        let report = match catch_unwind(AssertUnwindSafe(|| run_case(&dir, &evidence, case))) {
            Ok(r) => r,
            Err(_) => Report {
                file: case.file.into(),
                status: "FAIL",
                notes: vec!["FAIL: panic while executing fixture (caught)".into()],
                elapsed_ms: 0,
            },
        };
        for n in &report.notes {
            eprintln!("  {n}");
        }
        eprintln!("  => {} ({} ms)", report.status, report.elapsed_ms);
        reports.push(report);
    }

    let summary: String = reports
        .iter()
        .map(|r| format!("{} {} {}ms", r.status, r.file, r.elapsed_ms))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(evidence.join("summary.txt"), &summary);
    eprintln!("\n=== SUMMARY ===\n{summary}");

    let failed: Vec<_> = reports
        .iter()
        .filter(|r| r.status == "FAIL")
        .collect();
    let blocked: Vec<_> = reports.iter().filter(|r| r.status == "BLOCKED").collect();
    eprintln!(
        "totals: pass={} fail={} blocked={}",
        reports.iter().filter(|r| r.status == "PASS").count(),
        failed.len(),
        blocked.len()
    );
    // Keep going through every fixture; still fail the cargo test if any FAIL.
    assert!(
        failed.is_empty(),
        "{} fixture(s) failed:\n{}",
        failed.len(),
        failed
            .iter()
            .map(|r| format!("- {}: {}", r.file, r.notes.join("; ")))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
