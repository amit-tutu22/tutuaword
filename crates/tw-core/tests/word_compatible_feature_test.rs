//! Open/render/edit/find/round-trip probe for the Word-compatible feature fixture.

use tw_core::{document_plain_text, export_document, import_document_bundle, FormatContext, SyncSession};
use tw_edit::{Command, EditSession};
use tw_model::{
    Alignment, Block, HeaderFooterType, RunContent, ShapeKind, UnderlineStyle,
};
use tw_render::DisplayListBuilder;

const OMML_TOKEN: &str = "OMML_FEATURE_FIXTURE";

fn fixture_bytes() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../tw-docx/tests/corpus/word_compatible_feature_test.docx"
    );
    std::fs::read(path).expect("word_compatible_feature_test.docx fixture")
}

fn collect_plain_text(doc: &tw_model::Document) -> String {
    let mut out = String::new();
    for section in &doc.sections {
        for block in &section.blocks {
            match block {
                Block::Paragraph(p) => {
                    out.push_str(&p.full_text());
                    out.push('\n');
                }
                Block::Table(t) => {
                    for row in &t.rows {
                        for cell in &row.cells {
                            for b in &cell.blocks {
                                if let Block::Paragraph(p) = b {
                                    out.push_str(&p.full_text());
                                    out.push(' ');
                                }
                            }
                        }
                        out.push('\n');
                    }
                }
                _ => {}
            }
        }
        for hf in section.headers.values() {
            for b in &hf.blocks {
                if let Block::Paragraph(p) = b {
                    out.push_str(&p.full_text());
                    out.push('\n');
                }
            }
            if let Some(t) = &hf.plain_text {
                out.push_str(t);
                out.push('\n');
            }
        }
        for hf in section.footers.values() {
            for b in &hf.blocks {
                if let Block::Paragraph(p) = b {
                    out.push_str(&p.full_text());
                    out.push('\n');
                }
            }
            if let Some(t) = &hf.plain_text {
                out.push_str(t);
                out.push('\n');
            }
        }
    }
    out
}

fn count_blocks(doc: &tw_model::Document) -> (usize, usize, usize, usize) {
    let mut paragraphs = 0;
    let mut tables = 0;
    let mut images = 0;
    let mut shapes = 0;
    for section in &doc.sections {
        for block in &section.blocks {
            match block {
                Block::Paragraph(_) => paragraphs += 1,
                Block::Table(_) => tables += 1,
                Block::ImageBlock(_) => images += 1,
                Block::ShapeBlock(_) => shapes += 1,
                _ => {}
            }
        }
    }
    (paragraphs, tables, images, shapes)
}

fn shape_kinds(doc: &tw_model::Document) -> Vec<ShapeKind> {
    let mut kinds = Vec::new();
    for section in &doc.sections {
        for block in &section.blocks {
            if let Block::ShapeBlock(shape) = block {
                kinds.push(shape.shape.shape_type);
            }
        }
    }
    kinds
}

fn has_omml_token(doc: &tw_model::Document, token: &str) -> bool {
    for section in &doc.sections {
        for block in &section.blocks {
            let Block::Paragraph(p) = block else { continue };
            for run in &p.runs {
                if let RunContent::OfficeMath { xml } = &run.content {
                    if xml.contains(token) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn first_image_bytes(doc: &tw_model::Document) -> Option<Vec<u8>> {
    for section in &doc.sections {
        for block in &section.blocks {
            if let Block::ImageBlock(image) = block {
                return Some(image.data.bytes.clone());
            }
        }
    }
    None
}

fn has_style_named(doc: &tw_model::Document, ooxml_name: &str) -> bool {
    let Some(style_id) = doc.styles.ooxml_style_ids.get(ooxml_name).copied() else {
        return false;
    };
    for section in &doc.sections {
        for block in &section.blocks {
            if let Block::Paragraph(p) = block {
                if p.style_id == Some(style_id) {
                    return true;
                }
            }
        }
    }
    false
}

fn first_body_run(session: &SyncSession) -> tw_model::NodeId {
    for section in &session.edit.document.sections {
        for block in &section.blocks {
            if let Block::Paragraph(p) = block {
                if let Some(run) = p.runs.first() {
                    return run.id;
                }
            }
        }
    }
    panic!("document has no editable run");
}

fn has_bookmark_named(doc: &tw_model::Document, name: &str) -> bool {
    for section in &doc.sections {
        for block in &section.blocks {
            let Block::Paragraph(p) = block else { continue };
            for run in &p.runs {
                if let RunContent::Bookmark(b) = &run.content {
                    if b.name == name {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn has_hyperlink_example(doc: &tw_model::Document) -> bool {
    for section in &doc.sections {
        for block in &section.blocks {
            let Block::Paragraph(p) = block else { continue };
            for run in &p.runs {
                if let RunContent::Hyperlink { target, text } = &run.content {
                    if text.contains("example.com")
                        || target.url.contains("example.com")
                        || target.url.contains("rId16")
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn has_revision_run(doc: &tw_model::Document) -> bool {
    for section in &doc.sections {
        for block in &section.blocks {
            let Block::Paragraph(p) = block else { continue };
            for run in &p.runs {
                if run.revision.is_some() {
                    return true;
                }
            }
        }
    }
    false
}

fn has_citation_ref(doc: &tw_model::Document) -> bool {
    for section in &doc.sections {
        for block in &section.blocks {
            let Block::Paragraph(p) = block else { continue };
            for run in &p.runs {
                if matches!(run.content, RunContent::CitationRef(_)) {
                    return true;
                }
            }
        }
    }
    false
}

fn assert_f07_f09_f16_f19(bundle: &tw_core::ImportBundle, label: &str) {
    let doc = &bundle.document;
    assert!(
        doc.sections.len() >= 2,
        "{label}: expected multiple sections, got {}",
        doc.sections.len()
    );

    let landscape_idx = doc
        .sections
        .iter()
        .position(|s| s.format.is_landscape())
        .expect("{label}: landscape section missing");
    assert!(
        doc.sections[landscape_idx].format.is_landscape(),
        "{label}: section {landscape_idx} should be landscape"
    );

    let header_text = doc
        .resolved_header(landscape_idx, HeaderFooterType::Default)
        .and_then(|hf| {
            hf.blocks
                .first()
                .and_then(|b| b.paragraph().map(|p| p.full_text()))
                .or(hf.plain_text.clone())
        })
        .unwrap_or_default();
    assert!(
        header_text.contains("WORD-COMPATIBLE"),
        "{label}: header text missing WORD-COMPATIBLE, got {header_text:?}"
    );

    let footer_text = doc
        .resolved_footer(landscape_idx, HeaderFooterType::Default)
        .and_then(|hf| {
            hf.blocks
                .first()
                .and_then(|b| b.paragraph().map(|p| p.full_text()))
                .or(hf.plain_text.clone())
        })
        .unwrap_or_default();
    assert!(
        footer_text.contains("Feature Test Document") || footer_text.contains("Page"),
        "{label}: footer text missing expected content, got {footer_text:?}"
    );

    let plain = document_plain_text(doc);
    assert!(
        plain.contains("Item") && plain.contains("Quarter"),
        "{label}: document_plain_text missing table headers"
    );

    assert!(
        !doc.footnotes.is_empty(),
        "{label}: footnotes missing"
    );
    assert!(
        has_citation_ref(doc) || !doc.bibliography_sources.is_empty(),
        "{label}: citation or bibliography source missing"
    );
    assert!(
        has_bookmark_named(doc, "IMPORTANT_BOOKMARK"),
        "{label}: bookmark IMPORTANT_BOOKMARK missing"
    );
    assert!(has_revision_run(doc), "{label}: revision run missing");
    assert!(!doc.comments.is_empty(), "{label}: comments missing");
    assert!(has_hyperlink_example(doc), "{label}: hyperlink missing");
}

fn assert_f10_f14_objects(
    bundle: &tw_core::ImportBundle,
    label: &str,
) -> (usize, usize, usize) {
    let (paras, tables, images, shapes) = count_blocks(&bundle.document);
    assert!(images >= 1, "{label}: expected at least one image block");
    assert!(shapes >= 3, "{label}: expected shape/diagram/chart blocks");

    let image_bytes = first_image_bytes(&bundle.document).expect("{label}: image bytes");
    assert!(!image_bytes.is_empty(), "{label}: image bytes empty");

    let kinds = shape_kinds(&bundle.document);
    assert!(
        kinds.contains(&ShapeKind::TextBox),
        "{label}: missing TextBox shape, got {kinds:?}"
    );
    assert!(
        kinds.contains(&ShapeKind::Diagram),
        "{label}: missing Diagram shape, got {kinds:?}"
    );
    assert!(
        kinds.contains(&ShapeKind::Chart),
        "{label}: missing Chart shape, got {kinds:?}"
    );
    assert!(
        has_omml_token(&bundle.document, OMML_TOKEN),
        "{label}: OMML token missing"
    );

    let package = bundle.docx_package.as_ref().expect("{label}: docx package");
    for part in [
        "word/media/image1.png",
        "word/charts/chart1.xml",
        "word/diagrams/data1.xml",
        "word/diagrams/layout1.xml",
    ] {
        assert!(
            package.parts.contains_key(part),
            "{label}: missing package part {part}"
        );
    }

    (paras, tables, images)
}

#[test]
fn i_word_compat_open_render_find_edit_roundtrip() {
    let data = fixture_bytes();
    let bundle = import_document_bundle(&data, Some("word_compatible_feature_test.docx"))
        .expect("import fixture");

    let text = collect_plain_text(&bundle.document);
    for marker in [
        "Microsoft Word-Compatible Feature Test Document",
        "1. Document Management",
        "2. Text Editing",
        "3. Character Formatting",
        "4. Paragraph Formatting",
        "5. Lists",
        "6. Styles",
        "7. Page Layout",
        "8. Headers",
        "9. Tables",
        "10. Images",
        "11. Shapes",
        "12. Smart Objects",
        "13. Charts",
        "14. Equations",
        "PROJECT-ALPHA-2026",
        "project-alpha-2026",
        "CUSTOMER_NAME",
        "INV-0001",
        "IMPORTANT_BOOKMARK",
        "https://example.com",
        "The quick brown fox jumps over the lazy dog",
        "Left-aligned paragraph",
        "Centered paragraph",
        "Right-aligned paragraph",
        "Planning",
        "Requirement analysis",
        "Heading Style Test",
        "Custom Test Heading",
        "LANDSCAPE SECTION TEST",
        "FINAL TEST CHECKLIST",
        "Before ",
        "after (E = mc² preview)",
    ] {
        assert!(
            text.contains(marker),
            "imported text missing marker {marker:?}"
        );
    }

    let (paras, tables, _images) = assert_f10_f14_objects(&bundle, "import");
    assert!(paras >= 150, "expected rich paragraph body, got {paras}");
    assert_eq!(tables, 2, "fixture has two tables");
    assert_f07_f09_f16_f19(&bundle, "import");

    // Character formatting on the Bold/Italic/Underline sample run group.
    let mut saw_bold = false;
    let mut saw_italic = false;
    let mut saw_underline = false;
    let mut saw_all_caps = false;
    let mut saw_small_caps = false;
    let mut saw_center = false;
    let mut saw_right = false;
    let mut saw_justify = false;
    for section in &bundle.document.sections {
        for block in &section.blocks {
            let Block::Paragraph(p) = block else { continue };
            for run in &p.runs {
                if run.format.bold == Some(true) {
                    saw_bold = true;
                }
                if run.format.italic == Some(true) {
                    saw_italic = true;
                }
                if matches!(run.format.underline, Some(UnderlineStyle::Single)) {
                    saw_underline = true;
                }
                if run.format.all_caps == Some(true) {
                    saw_all_caps = true;
                }
                if run.format.small_caps == Some(true) {
                    saw_small_caps = true;
                }
            }
            match p.format.alignment {
                Some(Alignment::Center) => saw_center = true,
                Some(Alignment::Right) => saw_right = true,
                Some(Alignment::Justify) => saw_justify = true,
                _ => {}
            }
        }
    }
    assert!(saw_bold, "bold run missing");
    assert!(saw_italic, "italic run missing");
    assert!(saw_underline, "underline run missing");
    assert!(saw_all_caps, "all-caps run missing");
    assert!(saw_small_caps, "small-caps run missing");
    assert!(saw_center && saw_right && saw_justify, "para alignments missing");

    for style in [
        "Heading1",
        "Heading2",
        "ListBullet",
        "ListNumber",
        "Quote",
        "Title",
        "TestHeading",
        "TestSubheading",
    ] {
        assert!(
            has_style_named(&bundle.document, style),
            "style {style} not applied to any paragraph"
        );
    }

    // Header/footer imported from embedded section-break sectPr.
    let package = bundle.docx_package.as_ref().expect("docx package");
    assert!(
        package.parts.contains_key("word/header1.xml"),
        "header part missing from package"
    );
    assert!(
        package.parts.contains_key("word/footer1.xml"),
        "footer part missing from package"
    );
    for part in ["word/footnotes.xml", "word/comments.xml", "word/bibliography.xml"] {
        assert!(
            package.parts.contains_key(part),
            "package part {part} missing"
        );
    }

    // Layout + display list.
    let mut session = SyncSession::new();
    session.edit = EditSession::from_document(bundle.document.clone());
    session.relayout(None);
    assert!(session.page_count() >= 2, "page break / content should paginate");
    let session_text = document_plain_text(&session.edit.document);
    assert!(
        session_text.contains("Item") && session_text.contains("Quarter"),
        "session document_text missing table headers"
    );
    let dl = session.display_list_bytes();
    assert!(!dl.is_empty());
    let decoded = DisplayListBuilder::from_bytes(&dl).unwrap();
    assert!(decoded.page_width > 0.0 && decoded.page_height > 0.0);

    // Find / case sensitivity / wildcards.
    let case_sensitive = session.find_matches("PROJECT-ALPHA-2026", true, false, false, None);
    assert_eq!(case_sensitive.len(), 1, "exact case match count");
    let case_insensitive = session.find_matches("project-alpha-2026", false, false, false, None);
    assert!(
        case_insensitive.len() >= 2,
        "case-insensitive find expected ≥2, got {}",
        case_insensitive.len()
    );
    let wildcards = session.find_matches("INV-000*", false, false, true, None);
    assert!(
        wildcards.len() >= 3,
        "wildcard find expected ≥3 invoice ids, got {}",
        wildcards.len()
    );

    // Edit + DOCX package round-trip.
    let run_id = first_body_run(&session);
    session.apply(Command::InsertText {
        run_id,
        offset: 0,
        text: "[PROBE] ".into(),
    });
    assert!(collect_plain_text(&session.edit.document).contains("[PROBE]"));

    let ctx = FormatContext::from_bundle(bundle, Some("word_compatible_feature_test.docx".into()));
    let exported = export_document(&session.edit.document, &ctx).expect("export docx");
    let roundtrip =
        import_document_bundle(&exported, Some("word_compatible_feature_test.docx")).unwrap();
    let rt_text = collect_plain_text(&roundtrip.document);
    assert!(rt_text.contains("[PROBE]"));
    assert!(rt_text.contains("PROJECT-ALPHA-2026"));
    let (rt_paras, rt_tables, _) = assert_f10_f14_objects(&roundtrip, "roundtrip");
    assert_eq!(rt_tables, tables);
    assert!(rt_paras >= paras, "paragraph count should not shrink on round-trip");
    assert_f07_f09_f16_f19(&roundtrip, "roundtrip");
}
