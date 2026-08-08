//! Shared helpers for F05 list layout tests.

#![allow(dead_code)]

use tw_layout::{LayoutBox, LayoutEngine, TextLine};
use tw_model::Document;

pub const HANGING_PT: f32 = 18.0;
pub const HANG_TOL: f32 = 2.0;
pub const POS_TOL: f32 = 0.01;

pub fn layout_doc(doc: &Document) -> tw_layout::DocumentLayout {
    LayoutEngine::new().layout_document(doc)
}

pub fn list_lines(doc: &Document) -> Vec<TextLine> {
    layout_doc(doc)
        .pages
        .iter()
        .flat_map(|p| p.boxes.iter())
        .filter_map(|b| match b {
            LayoutBox::TextLine(l) if l.list_marker.is_some() => Some(l.clone()),
            _ => None,
        })
        .collect()
}

pub fn list_markers(doc: &Document) -> Vec<String> {
    list_lines(doc)
        .into_iter()
        .map(|l| l.list_marker.unwrap())
        .collect()
}

pub fn assert_near(actual: f32, expected: f32, tol: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected} ± {tol}, got {actual}"
    );
}

pub fn assert_hanging(line: &TextLine, expected_hang: f32) {
    let leftmost = line
        .glyphs
        .iter()
        .map(|g| g.x)
        .fold(f32::INFINITY, f32::min);
    let hang = line.x - leftmost;
    assert_near(hang, expected_hang, HANG_TOL, "hanging indent");
}

pub fn assert_text_x(line: &TextLine, margin_left: f32, indent_pt: f32) {
    assert_near(
        line.x,
        margin_left + indent_pt,
        POS_TOL,
        "list text x",
    );
}
