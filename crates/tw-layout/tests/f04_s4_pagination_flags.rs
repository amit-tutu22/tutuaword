//! F04.S4 — keep-together / widow-orphan pagination and paragraph decoration.

mod f04_layout_helpers;

use f04_layout_helpers::{
    count_para_lines, layout_doc, para_line_count_on_page, CONTENT_HEIGHT, EXACT_LINE_PT,
};
use tw_layout::LayoutBox;
use tw_model::{Block, BorderSet, BorderSpec, Color, Document, LineSpacing, ParaFormat, Paragraph};

fn single_line_para(text: &str) -> Paragraph {
    let mut para = Paragraph::with_text(text);
    para.format.line_spacing = Some(LineSpacing::Exactly(EXACT_LINE_PT));
    para
}

fn multi_line_para(text: &str, format: ParaFormat) -> Paragraph {
    let mut para = Paragraph::with_text(text);
    para.format = format;
    para
}

fn doc_with_fillers(count: usize, tail: Paragraph) -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..count {
        doc.sections[0]
            .blocks
            .push(Block::Paragraph(single_line_para(&format!("f{i}"))));
    }
    doc.sections[0].blocks.push(Block::Paragraph(tail));
    doc
}

const WRAP_TEXT: &str = "word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word word ";

/// U-F04-S4-keep-together-no-split
#[test]
fn u_f04_s4_keep_together_no_split() {
    let wrap = WRAP_TEXT.repeat(2);
    let kept = multi_line_para(
        &wrap,
        ParaFormat {
            line_spacing: Some(LineSpacing::Exactly(EXACT_LINE_PT)),
            keep_together: Some(true),
            ..Default::default()
        },
    );
    let kept_id = kept.id;
    let layout = layout_doc(&doc_with_fillers(25, kept));

    assert!(layout.pages.len() >= 2);
    assert_eq!(para_line_count_on_page(&layout, 0, kept_id), 0);
    assert!(para_line_count_on_page(&layout, 1, kept_id) >= 3);
    assert_eq!(count_para_lines(&layout, kept_id), para_line_count_on_page(&layout, 1, kept_id));

    // Negative control: without keep_together the paragraph may split across pages.
    let split = multi_line_para(
        &wrap,
        ParaFormat {
            line_spacing: Some(LineSpacing::Exactly(EXACT_LINE_PT)),
            ..Default::default()
        },
    );
    let split_id = split.id;
    let split_layout = layout_doc(&doc_with_fillers(25, split));
    assert!(
        para_line_count_on_page(&split_layout, 0, split_id) >= 1
            && count_para_lines(&split_layout, split_id) > para_line_count_on_page(&split_layout, 0, split_id),
        "without keep_together a long paragraph should split across pages"
    );
}

/// U-F04-S4-widow-orphan
#[test]
fn u_f04_s4_widow_orphan() {
    let wrap = WRAP_TEXT.repeat(2);
    let target = multi_line_para(
        &wrap,
        ParaFormat {
            line_spacing: Some(LineSpacing::Exactly(EXACT_LINE_PT)),
            widow_orphan_control: Some(true),
            ..Default::default()
        },
    );
    let target_id = target.id;
    let layout = layout_doc(&doc_with_fillers(26, target));

    assert_eq!(
        para_line_count_on_page(&layout, 0, target_id),
        0,
        "widow/orphan control must not leave an orphan line on page 0"
    );
    assert!(
        count_para_lines(&layout, target_id) >= 2,
        "target paragraph should still layout to multiple lines overall"
    );

    let loose = multi_line_para(
        &wrap,
        ParaFormat {
            line_spacing: Some(LineSpacing::Exactly(EXACT_LINE_PT)),
            widow_orphan_control: Some(false),
            ..Default::default()
        },
    );
    let loose_id = loose.id;
    let loose_layout = layout_doc(&doc_with_fillers(26, loose));
    assert_eq!(
        para_line_count_on_page(&loose_layout, 0, loose_id),
        1,
        "with widow control off exactly one line may sit alone at the page bottom"
    );
    assert!(
        count_para_lines(&loose_layout, loose_id) >= 2,
        "loose paragraph should still wrap to multiple lines total"
    );
}

#[test]
fn page_fill_leaves_expected_remainder() {
    // Sanity: 26 × 24 pt lines consume 624 pt of the 648 pt content area → 24 pt left.
    let layout = layout_doc(&doc_with_fillers(
        26,
        single_line_para("tail"),
    ));
    let used: f32 = layout.pages[0]
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.line_height),
            _ => None,
        })
        .take(26)
        .sum();
    assert!(
        (used - 26.0 * EXACT_LINE_PT).abs() < 1.0,
        "filler lines should consume 26 × {EXACT_LINE_PT} pt"
    );
    assert!(
        (CONTENT_HEIGHT - used - EXACT_LINE_PT).abs() < 2.0,
        "one exact line should remain on page 0 before the tail paragraph"
    );
}

#[test]
fn paragraph_shading_rect_precedes_text_line() {
    let mut doc = Document::new();
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.format.shading = Some(Color {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }

    let layout = layout_doc(&doc);
    let yellow = Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    }
    .to_argb();

    let rect_idx = layout.pages[0].boxes.iter().position(|b| {
        matches!(b, LayoutBox::Rect { color, width, height, .. }
            if *color == yellow && *width > 0.0 && *height > 0.0)
    });
    let line_idx = layout.pages[0]
        .boxes
        .iter()
        .position(|b| matches!(b, LayoutBox::TextLine(_)));

    assert!(rect_idx.is_some(), "shading should emit a non-empty Rect");
    assert!(line_idx.is_some());
    assert!(
        rect_idx.unwrap() < line_idx.unwrap(),
        "shading rect must paint before text lines"
    );
}

#[test]
fn paragraph_borders_emit_four_strokes() {
    let spec = BorderSpec {
        width: 1.0,
        color: Color::BLACK,
    };
    let mut doc = Document::new();
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.format.borders = Some(BorderSet {
            top: Some(spec),
            left: Some(spec),
            bottom: Some(spec),
            right: Some(spec),
        });
    }

    let layout = layout_doc(&doc);
    let black = Color::BLACK.to_argb();
    let border_rects: Vec<_> = layout.pages[0]
        .boxes
        .iter()
        .filter(|b| matches!(b, LayoutBox::Rect { color, .. } if *color == black))
        .collect();
    assert_eq!(
        border_rects.len(),
        4,
        "uniform borders should emit top/left/bottom/right rects"
    );
}
