//! Floating images are positioned absolutely; inline images occupy the flow.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{
    AnchorOrigin, Block, Document, ImageAnchor, ImageBlock, Paragraph, TextWrap,
};

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

fn document_with_wrap(image: ImageBlock, text: &str) -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::ImageBlock(image),
        Block::Paragraph(Paragraph::with_text(text)),
    ];
    doc
}

fn first_text_line(layout: &tw_layout::DocumentLayout) -> &tw_layout::TextLine {
    layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .find_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line),
            _ => None,
        })
        .expect("expected a text line")
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

#[test]
fn u_f10_s3_square_wrap_reflow() {
    let mut square = ImageBlock::placeholder(100.0, 80.0);
    square.wrap = TextWrap::Square;
    square.anchor = Some(ImageAnchor {
        x: 0.0,
        y: 0.0,
        origin_x: AnchorOrigin::Column,
        origin_y: AnchorOrigin::Column,
    });

    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(4);
    let mut engine = LayoutEngine::new();
    let square_layout = engine.layout_document(&document_with_wrap(square, &text));
    let inline_layout =
        engine.layout_document(&document_with_wrap(ImageBlock::placeholder(100.0, 80.0), &text));

    assert!(
        first_line_y(&square_layout) < first_line_y(&inline_layout),
        "square wrap should not push text below the inline image band",
    );

    let first_line = first_text_line(&square_layout);
    let content_width = square_layout.pages[0].content_width;
    assert!(
        first_line.x > 72.0 + 100.0,
        "text should start beside the image, got x={}",
        first_line.x,
    );
    assert!(
        first_line.width < content_width - 50.0,
        "line beside image should be narrower than the full column",
    );
}

#[test]
fn paragraph_relative_float_stays_with_its_host_flow() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(Paragraph::with_text("Title line")),
        Block::Paragraph(Paragraph::with_text("Intro paragraph one.")),
        {
            let mut image = ImageBlock::placeholder(100.0, 100.0);
            image.wrap = TextWrap::Square;
            image.anchor = Some(ImageAnchor {
                x: 0.0,
                y: 0.0,
                origin_x: AnchorOrigin::Margin,
                origin_y: AnchorOrigin::Paragraph,
            });
            Block::ImageBlock(image)
        },
        Block::Paragraph(Paragraph::with_text(
            "Text beside the square-wrapped image should not sit under it.",
        )),
    ];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let image = image_boxes(&layout)[0];
    let title_y = first_line_y(&layout);

    assert!(
        image.y > title_y + 10.0,
        "paragraph-relative float should sit below the title, image.y={} title_y={}",
        image.y,
        title_y,
    );

    let beside = layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) if line.y >= image.y && line.y < image.y + image.height => {
                Some(line)
            }
            _ => None,
        })
        .next()
        .expect("expected a text line beside the image");
    assert!(
        beside.x > image.x + image.width,
        "text should wrap to the right of the square image, x={}",
        beside.x,
    );
}

#[test]
fn top_and_bottom_wrap_skips_the_image_band() {
    let mut image = ImageBlock::placeholder(200.0, 80.0);
    image.wrap = TextWrap::TopBottom;
    image.anchor = Some(ImageAnchor {
        x: 0.0,
        y: 0.0,
        origin_x: AnchorOrigin::Column,
        origin_y: AnchorOrigin::Paragraph,
    });

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&document_with_wrap(image, "After the image band."));
    let placed = image_boxes(&layout)[0];
    let text_y = first_line_y(&layout);

    assert!(
        text_y >= placed.y + placed.height,
        "top-and-bottom wrap should push text below the image, text_y={} image_bottom={}",
        text_y,
        placed.y + placed.height,
    );
}
