//! F10.S1 — insert real PNG/JPEG/SVG bytes into the document model.

use tw_edit::{Command, EditSession};
use tw_model::Block;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

const MINI_SVG: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="blue"/></svg>"#;

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
}

#[test]
fn insert_image_bytes_stores_png_in_model() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    let data = tw_model::ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into()));

    session
        .apply(Command::InsertImage {
            after_block_id: after,
            width: 72.0,
            height: 72.0,
            data: Some(data),
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1]
        .image()
        .expect("image block");
    assert_eq!(image.data.bytes, PNG_1X1);
    assert_eq!(image.data.mime_type, "image/png");
    assert_eq!(image.data.width_px, 1);
    assert_eq!(image.data.height_px, 1);
}

#[test]
fn insert_image_bytes_accepts_svg() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    let data = tw_model::ImageData::from_bytes(MINI_SVG.to_vec(), Some("image/svg+xml".into()));

    session
        .apply(Command::InsertImage {
            after_block_id: after,
            width: 15.0,
            height: 7.5,
            data: Some(data),
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.data.mime_type, "image/svg+xml");
    assert_eq!(image.data.width_px, 20);
    assert_eq!(image.data.height_px, 10);
}

#[test]
fn inserted_image_block_is_inline() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    let data = tw_model::ImageData::from_bytes(PNG_1X1.to_vec(), None);

    session
        .apply(Command::InsertImage {
            after_block_id: after,
            width: 1.0,
            height: 1.0,
            data: Some(data),
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert!(matches!(image.wrap, tw_model::TextWrap::Inline));
    assert!(matches!(session.document.sections[0].blocks[1], Block::ImageBlock(_)));
}
