//! Shared helpers for F04 paragraph layout integration tests.

#![allow(dead_code)]

use tw_layout::{LayoutBox, LayoutEngine, PageIndex};
use tw_model::{Document, NodeId};

pub const CONTENT_HEIGHT: f32 = 648.0; // 792 - 72 - 72
pub const EXACT_LINE_PT: f32 = 24.0;

pub fn layout_doc(doc: &Document) -> tw_layout::DocumentLayout {
    LayoutEngine::new().layout_document(doc)
}

pub fn second_text_line_y(doc: &Document) -> f32 {
    layout_doc(doc)
        .pages[0]
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.y),
            _ => None,
        })
        .nth(1)
        .expect("second text line")
}

pub fn para_line_count_on_page(
    layout: &tw_layout::DocumentLayout,
    page: PageIndex,
    para_id: NodeId,
) -> usize {
    layout.pages[page as usize]
        .boxes
        .iter()
        .filter(|b| {
            matches!(
                b,
                LayoutBox::TextLine(l) if l.paragraph_id == para_id
            )
        })
        .count()
}

pub fn count_para_lines(layout: &tw_layout::DocumentLayout, para_id: NodeId) -> usize {
    layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .filter(|b| {
            matches!(
                b,
                LayoutBox::TextLine(l) if l.paragraph_id == para_id
            )
        })
        .count()
}

pub fn assert_near(actual: f32, expected: f32, tol: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected} ± {tol}, got {actual}"
    );
}
