//! F16.S2 — TOC entry paragraphs layout with tab + page number.

use tw_layout::{layout_paragraph, ParagraphFrame};
use tw_model::{
    build_toc_blocks, document_outline, Block, Document, OutlineEntry, Paragraph, RunContent,
    TabAlignment, TOC_TAB_POSITION,
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
