//! Shared helpers for F07.S4 page-decoration layout tests.

#![allow(dead_code)]

use tw_layout::{LayoutBox, LayoutEngine, PageLayout};
use tw_model::{Block, Document, Paragraph, SectionFormat};

pub fn layout(format: SectionFormat, text: &str) -> tw_layout::DocumentLayout {
    let mut doc = Document::new();
    doc.sections[0].format = format;
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(text))];
    LayoutEngine::new().layout_document(&doc)
}

pub fn content_line_count(page: &PageLayout, margin_left: f32) -> usize {
    page.boxes
        .iter()
        .filter(|b| {
            matches!(
                b,
                LayoutBox::TextLine(l) if (l.x - margin_left).abs() < 2.0
            )
        })
        .count()
}

pub fn gutter_line_count(page: &PageLayout, gutter_x: f32) -> usize {
    page.boxes
        .iter()
        .filter(|b| {
            matches!(
                b,
                LayoutBox::TextLine(l) if (l.x - gutter_x).abs() < 2.0
            )
        })
        .count()
}

pub fn first_box_index(page: &PageLayout, pred: impl Fn(&LayoutBox) -> bool) -> Option<usize> {
    page.boxes.iter().position(pred)
}

pub fn full_page_rect_index(page: &PageLayout, argb: u32) -> Option<usize> {
    first_box_index(page, |b| {
        matches!(
            b,
            LayoutBox::Rect {
                x,
                y,
                width,
                height,
                color,
            } if *x == 0.0
                && *y == 0.0
                && (*width - page.width).abs() < 0.5
                && (*height - page.height).abs() < 0.5
                && *color == argb
        )
    })
}

pub fn first_content_line_index(page: &PageLayout, margin_left: f32) -> Option<usize> {
    first_box_index(page, |b| {
        matches!(
            b,
            LayoutBox::TextLine(l) if (l.x - margin_left).abs() < 2.0
        )
    })
}

pub fn watermark_band_rect_index(page: &PageLayout, format: &SectionFormat) -> Option<usize> {
    let (x, y, w, h) = format.watermark_band();
    first_box_index(page, |b| {
        matches!(
            b,
            LayoutBox::Rect {
                x: rx,
                y: ry,
                width,
                height,
                ..
            } if (*rx - x).abs() < 0.5
                && (*ry - y).abs() < 0.5
                && (*width - w).abs() < 0.5
                && (*height - h).abs() < 0.5
        )
    })
}
