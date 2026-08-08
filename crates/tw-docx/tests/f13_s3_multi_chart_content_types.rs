//! F13.S3 hardening — multiple inserted charts each get content-type overrides.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::Block;

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
}

fn last_block_id(session: &EditSession) -> tw_model::NodeId {
    let blocks = &session.document.sections[0].blocks;
    match blocks.last().expect("blocks") {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
}

#[test]
fn u_f13_s3_multi_chart_content_types() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    session
        .apply(Command::InsertChart {
            after_block_id: after,
            width: 432.0,
            height: 216.0,
        })
        .unwrap();
    let after = last_block_id(&session);
    session
        .apply(Command::InsertChart {
            after_block_id: after,
            width: 360.0,
            height: 180.0,
        })
        .unwrap();

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();

    assert!(imported.package.parts.contains_key("word/charts/chart1.xml"));
    assert!(imported.package.parts.contains_key("word/charts/chart2.xml"));

    let content_types =
        String::from_utf8(imported.package.parts["[Content_Types].xml"].clone()).unwrap();
    assert!(content_types.contains("/word/charts/chart1.xml"));
    assert!(content_types.contains("/word/charts/chart2.xml"));
}
