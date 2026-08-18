//! F16.S2 — TOC entry paragraphs layout with tab + page number.

use tw_layout::{layout_paragraph, LayoutBox, LayoutEngine, ParagraphFrame};
use tw_model::{
    bookmark_run, build_toc_blocks, document_outline, hyperlink_run, Block, Document, OutlineEntry,
    Paragraph, RunContent, TabAlignment, TOC_TAB_POSITION,
};
use tw_shape::{GlyphAtlas, TextShaper};

fn layout_toc_entry(doc: &Document) -> Paragraph {
    let blocks = build_toc_blocks(&[(OutlineEntry {
        paragraph_id: doc.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .id,
        level: 1,
        text: "Methods".into(),
        run_id: doc.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .id,
    }, 2)]);

    match &blocks[1] {
        Block::Paragraph(p) => p.clone(),
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn u_f16_s2_toc_entry_tab_and_page_layout() {
    let doc = Document::with_paragraph("Heading");
    let para = layout_toc_entry(&doc);

    assert!(
        para.runs.iter().any(|r| matches!(r.content, RunContent::Tab)),
        "TOC entry must contain a tab run"
    );
    assert_eq!(
        para.format
            .tab_stops
            .as_ref()
            .and_then(|stops| stops.first())
            .map(|s| s.alignment),
        Some(TabAlignment::Right)
    );

    let mut shaper = TextShaper::new();
    let mut atlas = GlyphAtlas::new(512, 512);
    let (lines, _) = layout_paragraph(
        &mut shaper,
        &mut atlas,
        &para,
        ParagraphFrame::new(0.0, 0.0, 500.0)
            .with_tab_interval(36.0)
            .with_tab_stops(para.format.tab_stops.clone().unwrap_or_default()),
        0xFF000000,
    );
    assert_eq!(lines.len(), 1);
    let page_glyph = lines[0]
        .glyphs
        .iter()
        .find(|g| g.codepoint == '2')
        .expect("page number glyph");
    assert!(
        page_glyph.x >= TOC_TAB_POSITION - 24.0,
        "page number should sit near right tab stop, got x={}",
        page_glyph.x
    );
    assert!(
        lines[0].glyphs.iter().any(|g| g.codepoint == '.' && g.x < page_glyph.x),
        "dotted tab leader should fill the gap before the page number"
    );
}

#[test]
fn u_f16_s2_outline_excludes_toc_title() {
    let mut session_doc = Document::with_paragraph("Real Heading");
    let para_id = session_doc.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;
    if let Some(para) = session_doc.sections[0].blocks[0].paragraph_mut() {
        if let Some(style) = session_doc.styles.find_style_by_name("Heading 1") {
            para.style_id = Some(style.id);
            para.runs[0].content = tw_model::RunContent::Text("Real Heading".into());
        }
    }

    let toc_blocks = build_toc_blocks(&[(OutlineEntry {
        paragraph_id: para_id,
        level: 0,
        text: "Real Heading".into(),
        run_id: session_doc.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .id,
    }, 1)]);

    session_doc.sections[0]
        .blocks
        .extend(toc_blocks);

    let outline = document_outline(&session_doc);
    assert_eq!(outline.len(), 1);
    assert_eq!(outline[0].text, "Real Heading");
}

#[test]
fn u_f16_s2_imported_toc_hyperlink_gets_page_number_and_leader() {
    let mut doc = Document::new();
    let mut heading = Paragraph::with_text("Efficiency Indicators");
    heading.runs.insert(0, bookmark_run("_sk28j4tyc25k", 1));
    let mut toc = Paragraph::new();
    toc.runs = vec![hyperlink_run(
        "#_sk28j4tyc25k",
        "Efficiency Indicators",
        None,
    )];
    doc.sections[0].blocks = vec![Block::Paragraph(toc), Block::Paragraph(heading)];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];
    let content_right = page.content_left + page.content_width;

    let page_glyph = page
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(line) => line
                .glyphs
                .iter()
                .find(|g| g.codepoint == '1' && g.x > content_right - 48.0),
            _ => None,
        })
        .expect("imported TOC hyperlink should paint a page number");
    assert!(
        page_glyph.x > content_right - 48.0,
        "page number should sit near the right margin, got x={} right={}",
        page_glyph.x,
        content_right
    );

    let toc_line = page
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(line)
                if line.glyphs.iter().any(|g| g.codepoint == 'E')
                    && line.glyphs.iter().any(|g| g.codepoint == '1' && g.x > 400.0) =>
            {
                Some(line)
            }
            _ => None,
        })
        .expect("TOC entry line");
    let title_end = toc_line
        .glyphs
        .iter()
        .filter(|g| g.x < 400.0)
        .map(|g| g.x + g.width)
        .fold(0.0f32, f32::max);
    let leader_dots = toc_line
        .glyphs
        .iter()
        .filter(|g| g.codepoint == '.' && g.x > title_end && g.x < page_glyph.x)
        .count();
    assert!(
        leader_dots >= 4,
        "dotted leader should fill the gap, got {leader_dots} dots"
    );

    // The number is laid out on its own and then moved onto the entry, so it
    // must share the entry's baseline rather than float above the leaders.
    let title_baseline = toc_line
        .glyphs
        .iter()
        .find(|g| g.codepoint == 'E')
        .map(|g| g.y + g.height)
        .expect("title glyph");
    assert!(
        (page_glyph.y + page_glyph.height - title_baseline).abs() < 1.5,
        "page number should share the entry baseline, number_bottom={} title_bottom={title_baseline}",
        page_glyph.y + page_glyph.height
    );
}

/// Only a paragraph that is nothing but a link is a TOC entry; a cross-reference
/// inside a sentence must not grow dotted leaders and a page number.
#[test]
fn u_f16_s2_inline_cross_reference_is_not_decorated() {
    let mut doc = Document::new();
    let mut heading = Paragraph::with_text("Efficiency Indicators");
    heading.runs.insert(0, bookmark_run("_sk28j4tyc25k", 1));

    let mut body = Paragraph::new();
    body.runs = vec![
        tw_model::Run::new_text("As covered in "),
        hyperlink_run("#_sk28j4tyc25k", "Efficiency Indicators", None),
        tw_model::Run::new_text(", revenue grew."),
    ];
    doc.sections[0].blocks = vec![Block::Paragraph(body), Block::Paragraph(heading)];

    let layout = LayoutEngine::new().layout_document(&doc);
    let page = &layout.pages[0];
    let content_right = page.content_left + page.content_width;

    let decorated = page.boxes.iter().any(|b| match b {
        LayoutBox::TextLine(line) => line
            .glyphs
            .iter()
            .any(|g| g.codepoint == '1' && g.x > content_right - 48.0),
        _ => false,
    });
    assert!(
        !decorated,
        "an inline cross-reference should not get a TOC page number"
    );
}

