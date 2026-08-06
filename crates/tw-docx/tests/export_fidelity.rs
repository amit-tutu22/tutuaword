//! Round-trip gates for DOCX export.
//!
//! Export used to replace every table and image with an empty paragraph, so
//! saving a document destroyed it. These tests assert that what goes in comes
//! back out: block structure, table geometry, image bytes, list membership,
//! and page setup.

use std::io::{Cursor, Write};

use tw_docx::{export, import, DocxPackage};
use tw_model::{
    Alignment, Block, BorderSpec, CellFormat, CharFormat, Color, Document, ImageBlock, ImageData,
    NumberingRef, Paragraph, ParaFormat, Run, Table, TableCell, TableRow, TextWrap, UnderlineStyle,
};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn minimal_docx(document_xml: &str, extra_parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<Types/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", options)
            .unwrap();
        zip.write_all(b"<Relationships/>").unwrap();
        for (name, data) in extra_parts {
            zip.start_file(*name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }
    buf
}

/// Exports a freshly built document and reads it straight back.
fn round_trip(doc: &Document) -> Document {
    let bytes = export(doc, &DocxPackage::minimal()).unwrap();
    import(&bytes).unwrap().document
}

fn blocks(doc: &Document) -> &[Block] {
    &doc.sections[0].blocks
}

fn png_bytes() -> Vec<u8> {
    // Not a decodable image, but export only needs opaque bytes to store.
    vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3, 4]
}

// -- structure ----------------------------------------------------------------

#[test]
fn a_table_survives_export() {
    let mut doc = Document::new();
    let table = Table::new(2, 3);
    doc.sections[0].blocks = vec![Block::Table(table)];

    let result = round_trip(&doc);

    let table = blocks(&result)[0]
        .table()
        .expect("the table came back as something else");
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0].cells.len(), 3);
}

#[test]
fn table_cell_text_survives_export() {
    let mut doc = Document::new();
    let mut table = Table::new(1, 2);
    table.rows[0].cells[0].blocks = vec![Block::Paragraph(Paragraph::with_text("Left"))];
    table.rows[0].cells[1].blocks = vec![Block::Paragraph(Paragraph::with_text("Right"))];
    doc.sections[0].blocks = vec![Block::Table(table)];

    let result = round_trip(&doc);

    let table = blocks(&result)[0].table().unwrap();
    assert_eq!(
        table.rows[0].cells[0].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Left"
    );
    assert_eq!(
        table.rows[0].cells[1].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Right"
    );
}

#[test]
fn column_widths_and_row_heights_survive_export() {
    let mut doc = Document::new();
    let mut table = Table::new(1, 2);
    table.format.column_widths = vec![144.0, 72.0];
    table.format.width = Some(216.0);
    table.rows[0].height = Some(24.0);
    doc.sections[0].blocks = vec![Block::Table(table)];

    let result = round_trip(&doc);

    let table = blocks(&result)[0].table().unwrap();
    assert_eq!(table.format.column_widths, vec![144.0, 72.0]);
    assert_eq!(table.rows[0].height, Some(24.0));
}

#[test]
fn a_merged_cell_keeps_its_span() {
    let mut doc = Document::new();
    let wide = TableCell {
        format: CellFormat {
            colspan: 3,
            ..Default::default()
        },
        ..TableCell::new()
    };
    let table = Table {
        id: tw_model::NodeId::new(),
        format: tw_model::TableFormat {
            width: Some(300.0),
            column_widths: vec![100.0, 100.0, 100.0],
            border: Some(BorderSpec::default()),
        },
        rows: vec![TableRow::with_cells(vec![wide])],
        style_id: None,
    };
    doc.sections[0].blocks = vec![Block::Table(table)];

    let result = round_trip(&doc);

    let table = blocks(&result)[0].table().unwrap();
    assert_eq!(table.rows[0].cells[0].format.colspan, 3);
}

#[test]
fn an_image_survives_export_with_its_bytes() {
    let mut doc = Document::new();
    let image = ImageBlock {
        id: tw_model::NodeId::new(),
        data: ImageData {
            asset_id: "word/media/image1.png".into(),
            mime_type: "image/png".into(),
            width_px: 2,
            height_px: 2,
            bytes: png_bytes(),
        },
        display_width: 120.0,
        display_height: 90.0,
        wrap: TextWrap::Inline,
        anchor: None,
    };
    doc.sections[0].blocks = vec![Block::ImageBlock(image)];

    let result = round_trip(&doc);

    let image = blocks(&result)[0]
        .image()
        .expect("the image came back as something else");
    assert_eq!(image.data.bytes, png_bytes());
    assert_eq!(image.display_width, 120.0);
    assert_eq!(image.display_height, 90.0);
}

#[test]
fn an_exported_image_lands_in_the_media_folder_with_a_relationship() {
    let mut doc = Document::new();
    let mut image = ImageBlock::placeholder(72.0, 72.0);
    image.data.bytes = png_bytes();
    image.data.asset_id = "word/media/image1.png".into();
    doc.sections[0].blocks = vec![Block::ImageBlock(image)];

    let bytes = export(&doc, &DocxPackage::minimal()).unwrap();
    let package = import(&bytes).unwrap().package;

    assert!(package.parts.contains_key("word/media/image1.png"));
    let rels = String::from_utf8(
        package.parts["word/_rels/document.xml.rels"].clone(),
    )
    .unwrap();
    assert!(rels.contains("media/image1.png"), "{rels}");
    let types = String::from_utf8(package.parts["[Content_Types].xml"].clone()).unwrap();
    assert!(types.contains(r#"Extension="png""#), "{types}");
}

#[test]
fn a_floating_image_keeps_its_anchor() {
    let mut doc = Document::new();
    let mut image = ImageBlock::placeholder(60.0, 40.0);
    image.data.bytes = png_bytes();
    image.data.asset_id = "word/media/image1.png".into();
    image.anchor = Some(tw_model::ImageAnchor {
        x: 36.0,
        y: 18.0,
        origin_x: tw_model::AnchorOrigin::Page,
        origin_y: tw_model::AnchorOrigin::Column,
    });
    doc.sections[0].blocks = vec![Block::ImageBlock(image)];

    let result = round_trip(&doc);

    let anchor = blocks(&result)[0]
        .image()
        .unwrap()
        .anchor
        .expect("the image lost its anchor and became inline");
    assert_eq!(anchor.x, 36.0);
    assert_eq!(anchor.y, 18.0);
    assert_eq!(anchor.origin_x, tw_model::AnchorOrigin::Page);
}

#[test]
fn a_mixed_document_keeps_its_block_order_and_count() {
    let mut doc = Document::new();
    let mut image = ImageBlock::placeholder(48.0, 48.0);
    image.data.bytes = png_bytes();
    image.data.asset_id = "word/media/image1.png".into();
    doc.sections[0].blocks = vec![
        Block::Paragraph(Paragraph::with_text("Intro")),
        Block::Table(Table::new(1, 2)),
        Block::ImageBlock(image),
        Block::Paragraph(Paragraph::with_text("Outro")),
    ];

    let result = round_trip(&doc);

    let blocks = blocks(&result);
    assert_eq!(blocks.len(), 4, "blocks were dropped or duplicated");
    assert!(matches!(blocks[0], Block::Paragraph(_)));
    assert!(matches!(blocks[1], Block::Table(_)));
    assert!(matches!(blocks[2], Block::ImageBlock(_)));
    assert_eq!(blocks[3].paragraph().unwrap().full_text(), "Outro");
}

#[test]
fn an_empty_paragraph_is_not_swallowed() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(Paragraph::with_text("Above")),
        Block::Paragraph(Paragraph::new()),
        Block::Paragraph(Paragraph::with_text("Below")),
    ];

    let result = round_trip(&doc);

    assert_eq!(blocks(&result).len(), 3, "the blank line was lost");
}

// -- formatting ---------------------------------------------------------------

#[test]
fn list_membership_survives_export() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("A bullet");
    para.format.numbering = Some(NumberingRef {
        numbering_id: 3,
        level: 1,
    });
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let result = round_trip(&doc);

    assert_eq!(
        blocks(&result)[0].paragraph().unwrap().format.numbering,
        Some(NumberingRef {
            numbering_id: 3,
            level: 1
        })
    );
}

#[test]
fn paragraph_formatting_survives_export() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("Formatted");
    para.format = ParaFormat {
        alignment: Some(Alignment::Center),
        space_before: Some(12.0),
        space_after: Some(6.0),
        indent_left: Some(36.0),
        ..Default::default()
    };
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let result = round_trip(&doc);

    let format = &blocks(&result)[0].paragraph().unwrap().format;
    assert_eq!(format.alignment, Some(Alignment::Center));
    assert_eq!(format.space_before, Some(12.0));
    assert_eq!(format.space_after, Some(6.0));
    assert_eq!(format.indent_left, Some(36.0));
}

#[test]
fn a_hanging_indent_stays_hanging() {
    let mut doc = Document::new();
    let mut para = Paragraph::with_text("Hanging");
    para.format.indent_first_line = Some(-18.0);
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let result = round_trip(&doc);

    assert_eq!(
        blocks(&result)[0]
            .paragraph()
            .unwrap()
            .format
            .indent_first_line,
        Some(-18.0)
    );
}

#[test]
fn character_formatting_survives_export() {
    let mut doc = Document::new();
    let mut para = Paragraph::new();
    para.runs = vec![Run {
        id: tw_model::NodeId::new(),
        format: CharFormat {
            font_family: Some("Georgia".into()),
            font_size: Some(18.0),
            bold: Some(true),
            italic: Some(true),
            underline: Some(UnderlineStyle::Single),
            color: Some(Color {
                r: 0xC0,
                g: 0x39,
                b: 0x2B,
                a: 255,
            }),
            ..Default::default()
        },
        content: tw_model::RunContent::Text("Styled".into()),
        revision: None,
    }];
    doc.sections[0].blocks = vec![Block::Paragraph(para)];

    let result = round_trip(&doc);

    let format = &blocks(&result)[0].paragraph().unwrap().runs[0].format;
    assert_eq!(format.font_family.as_deref(), Some("Georgia"));
    assert_eq!(format.font_size, Some(18.0));
    assert_eq!(format.bold, Some(true));
    assert_eq!(format.italic, Some(true));
    assert!(format.underline.is_some());
    assert_eq!(
        format.color,
        Some(Color {
            r: 0xC0,
            g: 0x39,
            b: 0x2B,
            a: 255
        })
    );
}

#[test]
fn surrounding_spaces_are_not_collapsed() {
    let doc = Document::with_paragraph("  spaced  ");

    let result = round_trip(&doc);

    assert_eq!(
        blocks(&result)[0].paragraph().unwrap().full_text(),
        "  spaced  "
    );
}

#[test]
fn page_size_and_margins_survive_export() {
    let mut doc = Document::new();
    doc.sections[0].format.page_width = 842.0;
    doc.sections[0].format.page_height = 595.0;
    doc.sections[0].format.margin_left = 54.0;
    doc.sections[0].format.margin_top = 36.0;

    let result = round_trip(&doc);

    let format = &result.sections[0].format;
    assert_eq!(format.page_width, 842.0);
    assert_eq!(format.page_height, 595.0);
    assert_eq!(format.margin_left, 54.0);
    assert_eq!(format.margin_top, 36.0);
}

#[test]
fn a_header_reference_is_carried_across_a_save() {
    // The header part itself passes through untouched; the reference to it
    // lives in sectPr, which export regenerates.
    let xml = r#"<w:document><w:body><w:p><w:r><w:t>Body</w:t></w:r></w:p>
        <w:sectPr><w:headerReference w:type="default" r:id="rId7"/>
        <w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#;
    let imported = import(&minimal_docx(xml, &[("word/header1.xml", b"<hdr/>")])).unwrap();

    let mut doc = imported.document.clone();
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        *para.runs[0].text_mut().unwrap() = "Edited".into();
    }
    let exported = export(&doc, &imported.package).unwrap();

    let reimported = import(&exported).unwrap();
    let document_xml =
        String::from_utf8(reimported.package.parts["word/document.xml"].clone()).unwrap();
    assert!(
        document_xml.contains(r#"r:id="rId7""#),
        "the header reference was dropped: {document_xml}"
    );
}

// -- passthrough safety -------------------------------------------------------

#[test]
fn an_edit_that_forgot_to_flag_itself_is_still_written() {
    let xml = r#"<w:document><w:body><w:p><w:r><w:t>Original</w:t></w:r></w:p></w:body></w:document>"#;
    let imported = import(&minimal_docx(xml, &[])).unwrap();

    let mut doc = imported.document.clone();
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        *para.runs[0].text_mut().unwrap() = "Edited".into();
    }
    // Deliberately no `mark_modified`: the package still looks pristine.
    let exported = export(&doc, &imported.package).unwrap();

    assert_eq!(
        import(&exported).unwrap().document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Edited",
        "passthrough returned the original bytes and lost the edit"
    );
}

#[test]
fn an_untouched_document_still_passes_through_byte_for_byte() {
    let xml = r#"<w:document><w:body><w:p><w:r><w:t>Untouched</w:t></w:r></w:p></w:body></w:document>"#;
    let source = minimal_docx(xml, &[("word/theme/theme1.xml", b"<theme/>")]);
    let imported = import(&source).unwrap();

    let exported = export(&imported.document, &imported.package).unwrap();

    assert_eq!(exported, source, "passthrough should be byte for byte");
}

#[test]
fn track_changes_wrapper_round_trip() {
    let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:ins w:id="42" w:author="Alice" w:date="2024-01-01T00:00:00Z"><w:r><w:t>Added</w:t></w:r></w:ins><w:del w:id="43" w:author="Bob" w:date="2024-01-02T00:00:00Z"><w:r><w:t>Removed</w:t></w:r></w:del></w:p></w:body></w:document>"#;
    let imported = import(&minimal_docx(xml, &[])).unwrap();
    let para = imported.document.sections[0].blocks[0]
        .paragraph()
        .unwrap();
    assert_eq!(para.runs.len(), 2);
    assert!(matches!(
        para.runs[0].revision.as_ref().map(|r| r.revision_type),
        Some(tw_model::RevisionType::Insert)
    ));
    assert!(matches!(
        para.runs[1].revision.as_ref().map(|r| r.revision_type),
        Some(tw_model::RevisionType::Delete)
    ));

    let reimported = round_trip(&imported.document);
    let para = reimported.sections[0].blocks[0].paragraph().unwrap();
    assert_eq!(para.runs.len(), 2);
    assert!(para.runs[0].revision.is_some());
    assert!(para.runs[1].revision.is_some());
}

#[test]
fn numbering_xml_is_written_for_fresh_exports() {
    let doc = Document::with_paragraph("List item");
    let bytes = export(&doc, &DocxPackage::minimal()).unwrap();
    let imported = import(&bytes).unwrap();
    assert!(
        imported.package.parts.contains_key("word/numbering.xml"),
        "fresh export must include numbering definitions"
    );
    assert!(
        imported.document.settings.numbering.get(1).is_some(),
        "default bullet definition should round-trip"
    );
}

#[test]
fn a_document_with_no_source_package_is_always_serialized() {
    let doc = Document::with_paragraph("Fresh");

    let exported = export(&doc, &DocxPackage::minimal()).unwrap();

    assert_eq!(
        import(&exported).unwrap().document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .full_text(),
        "Fresh"
    );
}
