//! F12.S3 hardening — inserted SmartArt exports valid diagram OPC parts.

use tw_docx::{export, import};
use tw_edit::{Command, EditSession};
use tw_model::ShapeKind;

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        tw_model::Block::Paragraph(p) => p.id,
        tw_model::Block::Table(t) => t.id,
        tw_model::Block::ImageBlock(i) => i.id,
        tw_model::Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
}

#[test]
fn u_f12_s3_inserted_diagram_export_round_trip() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    session
        .apply(Command::InsertDiagram {
            after_block_id: after,
            width: 432.0,
            height: 216.0,
        })
        .unwrap();

    let package = tw_docx::DocxPackage::default();
    let exported = export(&session.document, &package).unwrap();
    let imported = import(&exported).unwrap();

    let shape = imported.document.sections[0].blocks[1]
        .shape()
        .expect("inserted diagram");
    assert_eq!(shape.shape.shape_type, ShapeKind::Diagram);
    assert!(imported.package.parts.contains_key("word/diagrams/data1.xml"));
    assert!(imported.package.parts.contains_key("word/diagrams/layout1.xml"));

    let document_xml =
        String::from_utf8(imported.package.parts["word/document.xml"].clone()).unwrap();
    assert!(
        document_xml.contains("r:dm=") && document_xml.contains("r:lo="),
        "diagram relIds missing: {document_xml}"
    );

    let content_types =
        String::from_utf8(imported.package.parts["[Content_Types].xml"].clone()).unwrap();
    assert!(content_types.contains("/word/diagrams/data1.xml"));
    assert!(content_types.contains("/word/diagrams/layout1.xml"));
}
