//! Floating images are positioned absolutely; inline images occupy the flow.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{AnchorOrigin, Block, Document, ImageAnchor, ImageBlock, Paragraph};

fn image_boxes(layout: &tw_layout::DocumentLayout) -> Vec<&tw_layout::ImageLayout> {
    layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .filter_map(|b| match b {
            LayoutBox::Image(img) => Some(img),
            _ => None,
        })
        .collect()
}

fn first_line_y(layout: &tw_layout::DocumentLayout) -> f32 {
    layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .find_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.y),
            _ => None,
        })
        .expect("expected a text line")
}

fn document_with(image: ImageBlock) -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::ImageBlock(image),
        Block::Paragraph(Paragraph::with_text("Dear Parents,")),
    ];
    doc
}

#[test]
fn a_floating_image_does_not_push_text_down() {
    let mut floating = ImageBlock::placeholder(60.0, 80.0);
    floating.anchor = Some(ImageAnchor {
        x: -40.0,
        y: 25.0,
        origin_x: AnchorOrigin::Column,
        origin_y: AnchorOrigin::Page,
    });

    let mut engine = LayoutEngine::new();
    let floated = engine.layout_document(&document_with(floating));
    let inline = engine.layout_document(&document_with(ImageBlock::placeholder(60.0, 80.0)));

    assert!(
        first_line_y(&floated) < first_line_y(&inline),
        "floating {} should sit above inline {}",
        first_line_y(&floated),
        first_line_y(&inline)
    );
}

#[test]
fn a_column_relative_anchor_offsets_from_the_left_margin() {
    let mut image = ImageBlock::placeholder(60.0, 80.0);
    image.anchor = Some(ImageAnchor {
        x: 10.0,
        y: 0.0,
        origin_x: AnchorOrigin::Column,
        origin_y: AnchorOrigin::Page,
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&document_with(image));
    let placed = image_boxes(&layout)[0];
    assert!((placed.x - 82.0).abs() < 0.01, "{}", placed.x); // 72pt margin + 10pt
}

#[test]
fn a_page_relative_anchor_ignores_the_margin() {
    let mut image = ImageBlock::placeholder(60.0, 80.0);
    image.anchor = Some(ImageAnchor {
        x: 20.0,
        y: 25.0,
        origin_x: AnchorOrigin::Page,
        origin_y: AnchorOrigin::Page,
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&document_with(image));
    let placed = image_boxes(&layout)[0];

    assert!((placed.x - 20.0).abs() < 0.01, "{}", placed.x);
    assert!((placed.y - 25.0).abs() < 0.01, "{}", placed.y);
}

#[test]
fn image_bytes_reach_the_layout_box() {
    let mut image = ImageBlock::placeholder(60.0, 80.0);
    image.data.bytes = vec![1, 2, 3, 4];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&document_with(image));

    assert_eq!(image_boxes(&layout)[0].encoded.as_slice(), &[1, 2, 3, 4]);
}
