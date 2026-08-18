//! F10.S4 — image transform, caption link, and JPEG re-encode.

use tw_edit::{Command, EditSession};
use tw_model::{Block, ImageBlock, ImageData, ImageTransform, TextWrap};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0,
    0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
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
            wrap_polygon: None,
    }
}

fn insert_image(session: &mut EditSession, image: ImageBlock) -> tw_model::NodeId {
    let id = image.id;
    session.document.sections[0]
        .blocks
        .push(Block::ImageBlock(image));
    id
}

#[test]
fn u_f10_s4_set_image_transform() {
    let mut session = EditSession::new();
    let image_id = insert_image(&mut session, inline_image());

    session
        .apply(Command::SetImageTransform {
            image_id,
            transform: ImageTransform {
                rotation_deg: 90.0,
                crop_left: 0.1,
                opacity: 0.5,
                ..Default::default()
            },
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert!((image.transform.rotation_deg - 90.0).abs() < 0.01);
    assert!((image.transform.crop_left - 0.1).abs() < 0.01);
    assert!((image.transform.opacity - 0.5).abs() < 0.01);
    let (w, _) = image.effective_display_size();
    assert!((w - 54.0).abs() < 0.01);

    session.undo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.transform.rotation_deg, 0.0);
}

#[test]
fn u_f10_s4_insert_image_caption() {
    let mut session = EditSession::new();
    let image_id = insert_image(&mut session, inline_image());

    session
        .apply(Command::InsertImageCaption { image_id })
        .unwrap();

    let blocks = &session.document.sections[0].blocks;
    let image = blocks[1].image().unwrap();
    let caption_id = image.caption_paragraph_id.expect("caption linked");
    let caption = blocks[2].paragraph().expect("caption paragraph");
    assert_eq!(caption.id, caption_id);
    assert_eq!(
        session
            .document
            .styles
            .find_style_by_name("Caption")
            .map(|s| s.id),
        caption.style_id
    );

    session.undo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert!(image.caption_paragraph_id.is_none());
    assert_eq!(session.document.sections[0].blocks.len(), 2);
}

#[test]
fn u_f10_s4_compress_image() {
    let mut session = EditSession::new();
    let image_id = insert_image(&mut session, inline_image());

    session
        .apply(Command::CompressImage {
            image_id,
            quality: 80,
        })
        .unwrap();

    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.data.mime_type, "image/jpeg");
    assert!(!image.data.bytes.is_empty());

    session.undo().unwrap();
    let image = session.document.sections[0].blocks[1].image().unwrap();
    assert_eq!(image.data.mime_type, "image/png");
    assert_eq!(image.data.bytes, PNG_1X1);
}
