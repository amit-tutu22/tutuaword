//! F21.S3 — set image alternative text with undo.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ImageBlock, ImageData, ImageTransform, TextWrap};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn insert_image(session: &mut EditSession) -> tw_model::NodeId {
    let image = ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 72.0,
        display_height: 48.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: None,
        wrap_polygon: None,
    };
    let id = image.id;
    session.document.sections[0]
        .blocks
        .push(Block::ImageBlock(image));
    id
}

#[test]
fn u_f21_s3_set_alt_text() {
    let mut session = EditSession::new();
    let image_id = insert_image(&mut session);

    session
        .apply(Command::SetImageAltText {
            image_id,
            alt_text: Some("  Company logo  ".into()),
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.alt_text.as_deref(), Some("Company logo"));

    session.undo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.alt_text, None);

    session.redo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.alt_text.as_deref(), Some("Company logo"));

    session
        .apply(Command::SetImageAltText {
            image_id,
            alt_text: None,
        })
        .unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.alt_text, None);
}
