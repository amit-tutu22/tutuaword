use tw_layout::{LayoutBox, LayoutEngine, TextLine};
use tw_model::{
    Block, Document, ListLevel, ListMarkerFormat, ListSuffix, NumberingDefinition, NumberingRef,
    Paragraph, ParagraphStyle, StyleId,
};

/// Mirrors what Word writes for a numbered list: the level carries the real
/// indents, and a "List Paragraph" style with its own `w:ind` is attached to
/// some items but not others.
fn document_with_mixed_list_styling() -> Document {
    let mut doc = Document::new();

    doc.settings.numbering.definitions.insert(
        30,
        NumberingDefinition {
            id: 30,
            name: "Seminars".into(),
            levels: vec![ListLevel {
                level: 0,
                format: ListMarkerFormat::Decimal,
                indent: 13.5,
                hanging: 18.0,
                suffix: ListSuffix::Tab,
            }],
        },
    );

    let list_paragraph = StyleId::new();
    doc.styles.paragraph_styles.insert(
        list_paragraph,
        ParagraphStyle {
            id: list_paragraph,
            name: "List Paragraph".into(),
            based_on: None,
            char_format: Default::default(),
            para_format: tw_model::ParaFormat {
                indent_left: Some(36.0),
                ..Default::default()
            },
            next_style: None,
        },
    );

    doc.sections[0].blocks = (0..4)
        .map(|i| {
            let mut para = Paragraph::with_text(format!("Item number {i}"));
            para.format.numbering = Some(NumberingRef {
                numbering_id: 30,
                level: 0,
            });
            // Only alternating items carry the style, as in the source document.
            if i % 2 == 0 {
                para.style_id = Some(list_paragraph);
            }
            Block::Paragraph(para)
        })
        .collect();

    doc
}

fn list_lines(doc: &Document) -> Vec<TextLine> {
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(doc);
    layout
        .pages
        .iter()
        .flat_map(|p| p.boxes.iter())
        .filter_map(|b| match b {
            LayoutBox::TextLine(l) if l.list_marker.is_some() => Some(l.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn list_items_align_regardless_of_paragraph_style() {
    let lines = list_lines(&document_with_mixed_list_styling());
    assert_eq!(lines.len(), 4);

    let first = lines[0].x;
    for line in &lines {
        assert!(
            (line.x - first).abs() < 0.01,
            "list item text edges disagree: {} vs {}",
            line.x,
            first
        );
    }
}

#[test]
fn numbering_level_indent_wins_over_the_style_indent() {
    let doc = document_with_mixed_list_styling();
    let margin = doc.sections[0].format.margin_left;
    let lines = list_lines(&doc);

    // The level asks for 13.5pt, the style for 36pt.
    assert!(
        (lines[0].x - (margin + 13.5)).abs() < 0.01,
        "expected the numbering indent at {}, got {}",
        margin + 13.5,
        lines[0].x
    );
}

#[test]
fn direct_indent_overrides_the_numbering_level() {
    let mut doc = document_with_mixed_list_styling();
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.format.indent_left = Some(72.0);
    }
    let margin = doc.sections[0].format.margin_left;
    let lines = list_lines(&doc);

    assert!(
        (lines[0].x - (margin + 72.0)).abs() < 0.01,
        "direct formatting should win, got {}",
        lines[0].x
    );
    assert!(
        (lines[1].x - (margin + 13.5)).abs() < 0.01,
        "other items should be unaffected, got {}",
        lines[1].x
    );
}

#[test]
fn the_marker_hangs_left_of_the_text() {
    let doc = document_with_mixed_list_styling();
    let lines = list_lines(&doc);

    for line in &lines {
        let leftmost = line.glyphs.iter().map(|g| g.x).fold(f32::INFINITY, f32::min);
        let hang = line.x - leftmost;
        assert!(
            hang > 12.0 && hang < 20.0,
            "marker should hang ~18pt left of the text, hung {hang}"
        );
    }
}
