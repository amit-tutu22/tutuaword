//! Layout dump for sample-files.com-formatted-report.docx TOC and lists.

use std::path::PathBuf;

use tw_core::import_document_bundle;
use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::RunContent;

fn sample_path() -> Option<PathBuf> {
    let dir = std::env::var("SAMPLE_DOCX_DIR")
        .map(PathBuf::from)
        .ok()
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Downloads")))?;
    let path = dir.join("sample-files.com-formatted-report.docx");
    path.is_file().then_some(path)
}

fn line_text(line: &tw_layout::TextLine) -> String {
    line.glyphs.iter().map(|g| g.codepoint).collect()
}

#[test]
fn formatted_report_toc_and_lists_have_visible_text() {
    let Some(path) = sample_path() else {
        eprintln!("skip: fixture missing");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let bundle = import_document_bundle(&bytes, Some("formatted-report.docx")).unwrap();
    let doc = &bundle.document;

    let mut toc_paras = 0usize;
    let mut empty_hyperlinks = 0usize;
    for section in &doc.sections {
        for block in &section.blocks {
            let Some(para) = block.paragraph() else {
                continue;
            };
            let hyperlinks: Vec<_> = para
                .runs
                .iter()
                .filter_map(|r| match &r.content {
                    RunContent::Hyperlink { text, .. } => Some(text.clone()),
                    _ => None,
                })
                .collect();
            if hyperlinks.is_empty() {
                continue;
            }
            toc_paras += 1;
            if hyperlinks.iter().any(|t| t.is_empty()) {
                empty_hyperlinks += 1;
            }
        }
    }
    assert!(
        toc_paras >= 10,
        "expected imported TOC hyperlinks, got {toc_paras}"
    );
    assert_eq!(empty_hyperlinks, 0, "TOC hyperlink text was dropped on import");

    let mut engine = LayoutEngine::new();
    for font in &bundle.embedded_fonts {
        let _ = engine.register_face(&font.spec, font.data.clone());
    }
    let layout = engine.layout_document(doc);
    assert!(layout.pages.len() >= 3, "pages={}", layout.pages.len());

    let page0 = &layout.pages[0];
    let tiny_rects = page0
        .boxes
        .iter()
        .filter(|b| matches!(b, LayoutBox::Rect { width, height, .. } if *width <= 2.5 && *height <= 2.5))
        .count();
    assert!(
        tiny_rects < 20,
        "TOC leaders should be glyphs on the text line, not hundreds of rects (got {tiny_rects})"
    );

    let joined: String = layout
        .pages
        .iter()
        .flat_map(|p| p.boxes.iter())
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line_text(line)),
            _ => None,
        })
        .collect();
    assert!(
        joined.contains("ExecutiveSummary"),
        "TOC/heading text missing from layout glyphs"
    );
    assert!(
        joined.contains("Revenuegrowth"),
        "list body text missing from layout glyphs"
    );

    let toc_line = page0
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(line)
                if line_text(line).contains("ExecutiveSummary")
                    && line.glyphs.iter().any(|g| g.codepoint == '.') =>
            {
                Some(line)
            }
            _ => None,
        })
        .expect("TOC entry should include dotted leader glyphs");
    let content_right = page0.content_left + page0.content_width;
    assert!(
        toc_line
            .glyphs
            .iter()
            .any(|g| g.codepoint.is_ascii_digit() && g.x > content_right - 48.0),
        "TOC entry should include a right-aligned page number"
    );

    let list_line = layout
        .pages
        .iter()
        .flat_map(|p| p.boxes.iter())
        .find_map(|b| match b {
            LayoutBox::TextLine(line)
                if line.list_marker.as_deref() == Some("•")
                    && line_text(line).contains("Revenuegrowth") =>
            {
                Some(line)
            }
            _ => None,
        })
        .expect("Key Highlights list item");
    let marker_x = list_line.glyphs.first().map(|g| g.x).unwrap_or(0.0);
    let text_x = list_line
        .glyphs
        .iter()
        .find(|g| g.codepoint == 'R')
        .map(|g| g.x)
        .unwrap_or(0.0);
    assert!(
        marker_x < text_x,
        "list marker should hang left of the body text (marker={marker_x} text={text_x})"
    );
}
