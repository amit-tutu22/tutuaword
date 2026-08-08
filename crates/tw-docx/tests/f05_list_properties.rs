//! F05 list paragraph property round-trip (numRestart, outlineLvl).

use tw_docx::{export, import, DocxPackage};
use tw_model::{Block, Document, NumberingRef, Paragraph};

#[test]
fn num_restart_and_outline_level_round_trip() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("Section");
    para.format.numbering = Some(NumberingRef {
        numbering_id: 2,
        level: 1,
    });
    para.format.num_restart = Some(true);
    para.format.outline_level = Some(1);
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let imported = import(&export(&doc, &DocxPackage::minimal()).unwrap()).unwrap();
    let document_xml = String::from_utf8(
        imported.package.parts["word/document.xml"].clone(),
    )
    .unwrap();
    assert!(
        document_xml.contains("numRestart"),
        "expected numRestart in export: {document_xml}"
    );
    assert!(
        document_xml.contains("outlineLvl"),
        "expected outlineLvl in export: {document_xml}"
    );

    let para = imported.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    assert_eq!(para.format.num_restart, Some(true));
    assert_eq!(para.format.outline_level, Some(1));
    assert_eq!(
        para.format.numbering,
        Some(NumberingRef {
            numbering_id: 2,
            level: 1,
        })
    );
}
