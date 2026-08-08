//! U-F10-S1-import-png-bytes — DOCX image round-trip preserves PNG bytes.

use tw_docx::{export, import, DocxPackage};
use tw_model::{Block, Document, ImageBlock, ImageData, ImageTransform, TextWrap};

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
    0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
    0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn round_trip(doc: &Document) -> Document {
    let bytes = export(doc, &DocxPackage::minimal()).unwrap();
    import(&bytes).unwrap().document
}

#[test]
fn u_f10_s1_import_png_bytes() {
    let mut doc = Document::new();
    let image = ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 72.0,
        display_height: 72.0,
        wrap: TextWrap::Inline,
        anchor: None,
        transform: ImageTransform::default(),
        caption_paragraph_id: None,
    };
    doc.sections[0].blocks = vec![Block::ImageBlock(image)];

    let result = round_trip(&doc);
    let image = result.sections[0].blocks[0].image().unwrap();

    assert_eq!(image.data.bytes, PNG_1X1);
    assert_eq!(image.data.mime_type, "image/png");
}
