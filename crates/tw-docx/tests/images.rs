//! Inline and floating image import: real media bytes and anchor placement.

use std::io::Write;
use tw_docx::import;
use tw_model::{AnchorOrigin, Block, TextWrap};
use zip::write::SimpleFileOptions;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

fn build_docx(document_body: &str, with_media: bool) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();

        zip.start_file("word/document.xml", opts).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0"?><w:document xmlns:w="w"><w:body>{document_body}</w:body></w:document>"#
        )
        .unwrap();

        zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
        write!(
            zip,
            r#"<?xml version="1.0"?><Relationships><Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/logo.png"/></Relationships>"#
        )
        .unwrap();

        if with_media {
            zip.start_file("word/media/logo.png", opts).unwrap();
            zip.write_all(PNG_1X1).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

fn image_blocks(doc: &tw_model::Document) -> Vec<&tw_model::ImageBlock> {
    doc.sections
        .iter()
        .flat_map(|s| &s.blocks)
        .filter_map(|b| match b {
            Block::ImageBlock(img) => Some(img),
            _ => None,
        })
        .collect()
}

const INLINE_DRAWING: &str = r#"<w:p><w:r><w:drawing><wp:inline><wp:extent cx="914400" cy="457200"/><a:graphic><a:graphicData><pic:pic><pic:blipFill><a:blip r:embed="rId7"/></pic:blipFill></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

#[test]
fn an_inline_image_carries_its_media_bytes() {
    let docx = build_docx(INLINE_DRAWING, true);
    let result = import(&docx).unwrap();
    let images = image_blocks(&result.document);

    assert_eq!(images.len(), 1, "expected one image block");
    let image = images[0];
    assert_eq!(image.data.bytes, PNG_1X1, "media bytes should be resolved");
    assert_eq!(image.data.mime_type, "image/png");
    assert_eq!(image.data.asset_id, "word/media/logo.png");
}

#[test]
fn extent_is_converted_from_emu_to_points() {
    let docx = build_docx(INLINE_DRAWING, true);
    let result = import(&docx).unwrap();
    let image = image_blocks(&result.document)[0];

    // 914400 EMU is one inch, which is 72 points.
    assert!((image.display_width - 72.0).abs() < 0.01, "{}", image.display_width);
    assert!((image.display_height - 36.0).abs() < 0.01, "{}", image.display_height);
}

#[test]
fn a_missing_media_part_leaves_a_placeholder() {
    let docx = build_docx(INLINE_DRAWING, false);
    let result = import(&docx).unwrap();
    let image = image_blocks(&result.document)[0];

    assert!(image.data.bytes.is_empty());
}

#[test]
fn an_inline_image_has_no_anchor() {
    let docx = build_docx(INLINE_DRAWING, true);
    let result = import(&docx).unwrap();

    assert!(image_blocks(&result.document)[0].anchor.is_none());
}

#[test]
fn a_vml_rule_is_not_treated_as_an_image() {
    // HTML-to-DOCX converters emit horizontal separators as a bare VML rect
    // inside w:pict. There is no picture here, so there is nothing to draw.
    let body = r##"<w:p><w:r><w:pict><v:rect alt="" style="width:475.5pt;height:.05pt" o:hr="f"><v:stroke filltype="solid" color="#000000" opacity="0"/></v:rect></w:pict></w:r></w:p>"##;
    let docx = build_docx(body, true);
    let result = import(&docx).unwrap();

    assert!(image_blocks(&result.document).is_empty());
}

#[test]
fn a_drawing_without_a_picture_is_not_an_image() {
    let body = r#"<w:p><w:r><w:drawing><wp:inline><wp:extent cx="914400" cy="914400"/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;
    let docx = build_docx(body, true);
    let result = import(&docx).unwrap();

    assert!(image_blocks(&result.document).is_empty());
}

#[test]
fn a_legacy_vml_picture_still_resolves_its_media() {
    let body = r#"<w:p><w:r><w:pict><v:shape><v:imagedata r:id="rId7" o:title="logo"/></v:shape></w:pict></w:r></w:p>"#;
    let docx = build_docx(body, true);
    let result = import(&docx).unwrap();
    let images = image_blocks(&result.document);

    assert_eq!(images.len(), 1);
    assert_eq!(images[0].data.bytes, PNG_1X1);
}

#[test]
fn an_anchored_image_records_its_offsets_and_origins() {
    let body = r#"<w:p><w:r><w:drawing><wp:anchor behindDoc="1">
        <wp:positionH relativeFrom="column"><wp:posOffset>-457200</wp:posOffset></wp:positionH>
        <wp:positionV relativeFrom="page"><wp:posOffset>228600</wp:posOffset></wp:positionV>
        <wp:extent cx="914400" cy="914400"/>
        <a:blip r:embed="rId7"/>
        </wp:anchor></w:drawing></w:r></w:p>"#;
    let docx = build_docx(body, true);
    let result = import(&docx).unwrap();
    let image = image_blocks(&result.document)[0];
    let anchor = image.anchor.expect("anchored image should have an anchor");

    assert!((anchor.x - -36.0).abs() < 0.01, "{}", anchor.x);
    assert!((anchor.y - 18.0).abs() < 0.01, "{}", anchor.y);
    assert_eq!(anchor.origin_x, AnchorOrigin::Column);
    assert_eq!(anchor.origin_y, AnchorOrigin::Page);
    assert_eq!(image.wrap, TextWrap::Behind);
}

#[test]
fn square_wrap_and_paragraph_origin_are_preserved() {
    let body = r#"<w:p><w:r><w:drawing><wp:anchor behindDoc="0">
        <wp:positionH relativeFrom="margin"><wp:posOffset>0</wp:posOffset></wp:positionH>
        <wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV>
        <wp:extent cx="914400" cy="914400"/>
        <wp:wrapSquare wrapText="bothSides"/>
        <a:blip r:embed="rId7"/>
        </wp:anchor></w:drawing></w:r></w:p>"#;
    let docx = build_docx(body, true);
    let result = import(&docx).unwrap();
    let image = image_blocks(&result.document)[0];
    let anchor = image.anchor.expect("anchored image should have an anchor");

    assert_eq!(image.wrap, TextWrap::Square);
    assert_eq!(anchor.origin_x, AnchorOrigin::Margin);
    assert_eq!(anchor.origin_y, AnchorOrigin::Paragraph);
}

#[test]
fn wrap_none_in_front_is_not_behind() {
    let body = r#"<w:p><w:r><w:drawing><wp:anchor behindDoc="0">
        <wp:positionH relativeFrom="margin"><wp:posOffset>0</wp:posOffset></wp:positionH>
        <wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV>
        <wp:extent cx="914400" cy="914400"/>
        <wp:wrapNone/>
        <a:blip r:embed="rId7"/>
        </wp:anchor></w:drawing></w:r></w:p>"#;
    let docx = build_docx(body, true);
    let result = import(&docx).unwrap();
    assert_eq!(image_blocks(&result.document)[0].wrap, TextWrap::InFront);
}

#[test]
fn tight_wrap_polygon_is_imported() {
    let body = r#"<w:p><w:r><w:drawing><wp:anchor behindDoc="0">
        <wp:positionH relativeFrom="margin"><wp:posOffset>0</wp:posOffset></wp:positionH>
        <wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV>
        <wp:extent cx="914400" cy="914400"/>
        <wp:wrapTight wrapText="bothSides"><wp:wrapPolygon edited="0">
            <wp:start x="0" y="0"/><wp:lineTo x="762000" y="0"/>
            <wp:lineTo x="762000" y="914400"/><wp:lineTo x="0" y="914400"/>
        </wp:wrapPolygon></wp:wrapTight>
        <a:blip r:embed="rId7"/>
        </wp:anchor></w:drawing></w:r></w:p>"#;
    let docx = build_docx(body, true);
    let result = import(&docx).unwrap();
    let image = image_blocks(&result.document)[0];

    assert_eq!(image.wrap, TextWrap::Tight);
    let polygon = image.wrap_polygon.as_ref().expect("wrap polygon");
    assert_eq!(polygon.len(), 4);
    assert!((polygon[1].0 - 60.0).abs() < 0.01, "got {:?}", polygon[1]);
}

/// Exporting tight/through without the contour silently downgrades the wrap to
/// the image's bounding box on the next open.
#[test]
fn tight_wrap_polygon_survives_a_round_trip() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ImageBlock(tw_model::ImageBlock {
        id: tw_model::NodeId::new(),
        data: tw_model::ImageData::from_bytes(PNG_1X1.to_vec(), Some("image/png".into())),
        display_width: 72.0,
        display_height: 72.0,
        wrap: TextWrap::Tight,
        anchor: Some(tw_model::ImageAnchor {
            x: 0.0,
            y: 0.0,
            origin_x: AnchorOrigin::Column,
            origin_y: AnchorOrigin::Paragraph,
        }),
        transform: tw_model::ImageTransform::default(),
        caption_paragraph_id: None,
        alt_text: None,
        wrap_polygon: Some(vec![(0.0, 0.0), (60.0, 0.0), (60.0, 72.0), (0.0, 72.0)]),
    })];

    let bytes = tw_docx::export(&doc, &tw_docx::DocxPackage::minimal()).expect("export");
    let reopened = import(&bytes).unwrap();
    let image = image_blocks(&reopened.document)[0];

    assert_eq!(image.wrap, TextWrap::Tight);
    let polygon = image.wrap_polygon.as_ref().expect("exported wrap polygon");
    assert!(
        polygon.len() >= 4,
        "expected the contour back, got {polygon:?}"
    );
    assert!((polygon[1].0 - 60.0).abs() < 0.01, "got {:?}", polygon[1]);
    assert!((polygon[2].1 - 72.0).abs() < 0.01, "got {:?}", polygon[2]);
}

#[test]
fn top_and_bottom_wrap_is_preserved() {
    let body = r#"<w:p><w:r><w:drawing><wp:anchor behindDoc="0">
        <wp:positionH relativeFrom="margin"><wp:posOffset>0</wp:posOffset></wp:positionH>
        <wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV>
        <wp:extent cx="914400" cy="914400"/>
        <wp:wrapTopAndBottom/>
        <a:blip r:embed="rId7"/>
        </wp:anchor></w:drawing></w:r></w:p>"#;
    let docx = build_docx(body, true);
    let result = import(&docx).unwrap();
    assert_eq!(image_blocks(&result.document)[0].wrap, TextWrap::TopBottom);
}
