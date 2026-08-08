//! Stress: ten inserted charts export with matching OPC metadata.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::Block;

fn last_block_id(session: &EditSession) -> tw_model::NodeId {
    let blocks = &session.document.sections[0].blocks;
    match blocks.last().expect("blocks") {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block"),
    }
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_multi_chart_export_ten() {
    let mut session = EditSession::new();
    let mut after = last_block_id(&session);
    for _ in 0..10 {
        session
            .apply(Command::InsertChart {
                after_block_id: after,
                width: 300.0,
                height: 150.0,
            })
            .unwrap();
        after = last_block_id(&session);
    }

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();

    let content_types =
        String::from_utf8(imported.package.parts["[Content_Types].xml"].clone()).unwrap();
    for i in 1..=10 {
        let part = format!("word/charts/chart{i}.xml");
        assert!(imported.package.parts.contains_key(&part), "missing {part}");
        assert!(
            content_types.contains(&format!("/{part}")),
            "missing override for {part}"
        );
    }
}
