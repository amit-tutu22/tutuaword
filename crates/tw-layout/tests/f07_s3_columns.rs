//! F07.S3 — multi-column section layout.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, ColumnLayout, Document, Paragraph, SectionFormat};

#[test]
fn u_f07_s3_two_columns_balanced_flow() {
    let words: String = std::iter::repeat("word ").take(600).collect();
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(words.trim()))];
    doc.sections[0].format.page_height = 400.0;
    doc.sections[0].format.columns = ColumnLayout {
        count: 2,
        gap: 12.0,
    };

    let layout = LayoutEngine::new().layout_document(&doc);
    let margin_left = doc.sections[0].format.margin_left;
    let content = doc.sections[0].format.page_width
        - doc.sections[0].format.margin_left
        - doc.sections[0].format.margin_right;
    let col_width = (content - 12.0) / 2.0;
    let col2_x = margin_left + col_width + 12.0;

    let xs: Vec<f32> = layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.x),
            _ => None,
        })
        .collect();

    assert!(
        xs.iter().any(|x| (*x - margin_left).abs() < 1.0),
        "expected lines in column 1, got {xs:?}"
    );
    assert!(
        xs.iter().any(|x| (*x - col2_x).abs() < 1.0),
        "expected lines in column 2, got {xs:?}"
    );
}

#[test]
fn u_f07_s3_three_columns_narrower_width() {
    let mut format = SectionFormat::default();
    format.columns = ColumnLayout {
        count: 3,
        gap: 12.0,
    };
    let content = format.page_width - format.margin_left - format.margin_right;
    let col_width = (content - 24.0) / 3.0;

    let mut doc = Document::new();
    doc.sections[0].format = format;
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
        "Three column layout test paragraph with enough text to wrap.",
    ))];

    let layout = LayoutEngine::new().layout_document(&doc);
    let max_line_width = layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.width),
            _ => None,
        })
        .fold(0.0f32, f32::max);

    assert!(
        max_line_width <= col_width + 2.0,
        "line width {max_line_width} should fit column width {col_width}"
    );
}

#[test]
fn u_f07_s3_huge_column_gap_clamps_to_usable_columns() {
    let words: String = std::iter::repeat("word ").take(400).collect();
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(words.trim()))];
    doc.sections[0].format.page_height = 792.0;
    doc.sections[0].format.columns = ColumnLayout {
        count: 2,
        // python-docx EMU-sized gap (~18k pt if interpreted as twips).
        gap: 360_000.0,
    };

    let layout = LayoutEngine::new().layout_document(&doc);
    assert!(
        layout.pages.len() < 8,
        "clamped gap should not explode pagination, got {} pages",
        layout.pages.len()
    );

    let margin_left = doc.sections[0].format.margin_left;
    let content = doc.sections[0].format.page_width
        - doc.sections[0].format.margin_left
        - doc.sections[0].format.margin_right;
    let mut gap: f32 = 360_000.0 / 12700.0;
    let max_gap = ((content - 2.0) / 1.0).max(0.0);
    gap = gap.min(max_gap);
    let col_width = (content - gap) / 2.0;
    let col2_x = margin_left + col_width + gap;
    let xs: Vec<f32> = layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.x),
            _ => None,
        })
        .collect();
    assert!(
        xs.iter().any(|x| (*x - col2_x).abs() < 2.0),
        "expected usable second column after gap clamp, got {xs:?}"
    );
}
