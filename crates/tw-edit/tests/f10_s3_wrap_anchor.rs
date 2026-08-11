//! F10.S3 — image wrap modes and anchor positioning.

use tw_edit::{Command, EditSession};
use tw_model::{AnchorOrigin, Block, ImageAnchor, ImageBlock, ImageData, ImageTransform, TextWrap};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn inline_image() -> ImageBlock {
    ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 60.0,
        display_height: 40.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: None,
    }
}

fn insert_image(session: &mut EditSession, image: ImageBlock) -> tw_model::NodeId {
    let id = image.id;
    session.document.sections[0].blocks.push(Block::ImageBlock(image));
    id
}

#[test]
fn u_f10_s3_set_image_wrap_square_adds_anchor() {
    let mut session = EditSession::new();
    let image_id = insert_image(&mut session, inline_image());

    session
        .apply(Command::SetImageWrap {
            image_id,
            wrap: TextWrap::Square,
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.wrap, TextWrap::Square);
    assert!(image.anchor.is_some());

    session.undo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.wrap, TextWrap::Inline);
    assert!(image.anchor.is_none());
}

#[test]
fn u_f10_s3_set_image_wrap_inline_clears_anchor() {
    let mut session = EditSession::new();
    let mut image = inline_image();
    image.wrap = TextWrap::Square;
    image.anchor = Some(ImageAnchor {
        x: 12.0,
        y: 18.0,
        origin_x: AnchorOrigin::Column,
        origin_y: AnchorOrigin::Column,
    });
    let image_id = insert_image(&mut session, image);

    session
        .apply(Command::SetImageWrap {
            image_id,
            wrap: TextWrap::Inline,
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.wrap, TextWrap::Inline);
    assert!(image.anchor.is_none());
}

#[test]
fn u_f10_s3_set_image_anchor() {
    let mut session = EditSession::new();
    let image_id = insert_image(&mut session, inline_image());

    session
        .apply(Command::SetImageAnchor {
            image_id,
            anchor: ImageAnchor {
                x: 24.0,
                y: 36.0,
                origin_x: AnchorOrigin::Column,
                origin_y: AnchorOrigin::Column,
            },
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.wrap, TextWrap::Square);
    let anchor = image.anchor.expect("anchor");
    assert!((anchor.x - 24.0).abs() < 0.01);
    assert!((anchor.y - 36.0).abs() < 0.01);

    session.undo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.wrap, TextWrap::Inline);
    assert!(image.anchor.is_none());
}
