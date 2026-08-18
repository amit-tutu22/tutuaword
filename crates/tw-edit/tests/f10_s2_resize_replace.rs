//! F10.S2 — resize image display size and replace bytes while preserving wrap.

use tw_edit::{Command, EditSession};
use tw_model::{AnchorOrigin, Block, ImageAnchor, ImageBlock, ImageData, ImageTransform, TextWrap};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

const PNG_2X2: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x08, 0x06, 0x00, 0x00, 0x00, 0x72,
    0xB6, 0x0D, 0x24, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x60,
    0x60, 0x60, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0x5C, 0xCD, 0xFF, 0x69, 0x00, 0x00, 0x00,
    0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn floating_image() -> ImageBlock {
    ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 60.0,
        display_height: 40.0,
        wrap: TextWrap::Square,
        anchor: Some(ImageAnchor {
            x: 12.0,
            y: 18.0,
            origin_x: AnchorOrigin::Column,
            origin_y: AnchorOrigin::Page,
        }),
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: None,
            wrap_polygon: None,
    }
}

fn insert_floating(session: &mut EditSession) -> tw_model::NodeId {
    let image = floating_image();
    let id = image.id;
    session.document.sections[0]
        .blocks
        .push(Block::ImageBlock(image));
    id
}

#[test]
fn u_f10_s2_set_image_size() {
    let mut session = EditSession::new();
    let image_id = insert_floating(&mut session);

    session
        .apply(Command::SetImageSize {
            image_id,
            width: 120.0,
            height: 90.0,
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.display_width, 120.0);
    assert_eq!(image.display_height, 90.0);

    session.undo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.display_width, 60.0);
    assert_eq!(image.display_height, 40.0);
}

#[test]
fn u_f10_s2_replace_bytes_keeps_wrap() {
    let mut session = EditSession::new();
    let image_id = insert_floating(&mut session);
    let before = session.document.sections[0].blocks[1]
        .image()
        .unwrap()
        .clone();

    let replacement = ImageData::from_bytes(PNG_2X2.to_vec(), Some("image/png".into()));
    session
        .apply(Command::ReplaceImageBytes {
            image_id,
            data: replacement,
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.data.bytes, PNG_2X2);
    assert_eq!(image.data.width_px, 2);
    assert_eq!(image.wrap, before.wrap);
    assert!(image.anchor.is_some());
    assert!(before.anchor.is_some());
    let after_anchor = image.anchor.unwrap();
    let before_anchor = before.anchor.unwrap();
    assert!((after_anchor.x - before_anchor.x).abs() < 0.01);
    assert!((after_anchor.y - before_anchor.y).abs() < 0.01);
    assert_eq!(after_anchor.origin_x, before_anchor.origin_x);
    assert_eq!(after_anchor.origin_y, before_anchor.origin_y);
    assert_eq!(image.display_width, before.display_width);
    assert_eq!(image.display_height, before.display_height);

    session.undo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.data.bytes, PNG_1X1);
}
