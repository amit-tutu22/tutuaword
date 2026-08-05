//! Stage 3 layout unit tests: font metrics, decorations, justify, page splits.

use tw_layout::{DecorationKind, LayoutBox, LayoutEngine};
use tw_model::{
    Alignment, Block, CharFormat, Color, Document, Paragraph, ParaFormat, UnderlineStyle,
};

#[test]
fn underline_and_highlight_produce_rect_decorations() {
    let mut doc = Document::new();
    let mut run = tw_model::Run::new_text("Decorated");
    run.format = CharFormat {
        underline: Some(UnderlineStyle::Single),
        highlight: Some(Color {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        }),
        ..Default::default()
    };
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![run],
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let line = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l),
            _ => None,
        })
        .expect("text line");

    assert!(
        line.decorations
            .iter()
            .any(|d| d.kind == DecorationKind::Underline),
        "underline should emit a decoration rect"
    );
    assert!(
        line.decorations
            .iter()
            .any(|d| d.kind == DecorationKind::Highlight),
        "highlight should emit a background rect"
    );
}

#[test]
fn real_font_metrics_are_proportional_to_em_size() {
    let doc = Document::with_paragraph("Metrics");
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let line = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l),
            _ => None,
        })
        .expect("text line");

    assert!(
        line.ascent > 8.0 && line.ascent < 14.0,
        "ascent should come from font tables, got {}",
        line.ascent
    );
    assert!(line.descent > 2.0 && line.descent < 6.0);
}

#[test]
fn long_paragraph_splits_across_pages() {
    let mut doc = Document::new();
    let text: String = (0..80)
        .map(|i| format!("Sentence number {i} with enough words to wrap. "))
        .collect();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(text))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    assert!(
        layout.pages.len() > 1,
        "expected paragraph to continue on a second page, got {} pages",
        layout.pages.len()
    );

    let line_count: usize = layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .filter(|b| matches!(b, LayoutBox::TextLine(_)))
        .count();
    assert!(line_count > 20, "expected many text lines across pages");
}

#[test]
fn justified_paragraph_records_space_stops() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph({
        let mut p = Paragraph::with_text(
            "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron",
        );
        p.format = ParaFormat {
            alignment: Some(Alignment::Justify),
            ..Default::default()
        };
        p
    })];
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let lines: Vec<_> = layout.pages[0]
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l),
            _ => None,
        })
        .collect();
    assert!(
        lines.iter().any(|l| !l.justify_stops.is_empty()),
        "justified lines should record space positions"
    );
}
