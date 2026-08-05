use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Document, ImageBlock, NumberingRef, Paragraph, Table};

#[test]
fn heading1_style_changes_line_metrics() {
    let mut doc = Document::new();
    let heading_id = doc
        .styles
        .find_style_by_name("Heading 1")
        .expect("Heading 1 style")
        .id;
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.style_id = Some(heading_id);
        para.runs[0].format.bold = Some(true);
        para.runs[0].format.font_size = Some(16.0);
    }

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let line = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l),
            _ => None,
        })
        .expect("text line");

    assert!(line.ascent >= 16.0);
}

#[test]
fn table_with_merged_cells_layouts_single_span() {
    let mut doc = Document::new();
    doc.sections[0].blocks.push(Block::Table(Table::new(2, 2)));
    if let Some(table) = doc.sections[0].blocks[1].table_mut() {
        table.rows[0].cells[0].format.colspan = 2;
    }

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let table = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::Table(t) => Some(t),
            _ => None,
        })
        .expect("table layout");

    assert_eq!(table.cells.len(), 3);
}

#[test]
fn header_footer_text_on_page() {
    let mut doc = Document::new();
    doc.sections[0].format.header_text = Some("Header".into());
    doc.sections[0].format.footer_text = Some("Footer".into());
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph::with_text("Body"));

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let text_lines: Vec<_> = layout.pages[0]
        .boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.clone()),
            _ => None,
        })
        .collect();

    assert!(text_lines.len() >= 3);
}

#[test]
fn numbered_list_renders_markers_on_lines() {
    let mut doc = Document::new();
    if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
        para.format.numbering = Some(NumberingRef {
            numbering_id: 2,
            level: 0,
        });
        *para = Paragraph::with_text("First item");
        para.format.numbering = Some(NumberingRef {
            numbering_id: 2,
            level: 0,
        });
    }

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let line = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l),
            _ => None,
        })
        .expect("text line");

    assert!(line.list_marker.is_some());
    assert!(!line.glyphs.is_empty());
}

#[test]
fn image_block_produces_image_layout_box() {
    let mut doc = Document::new();
    doc.sections[0]
        .blocks
        .push(Block::ImageBlock(ImageBlock::placeholder(120.0, 80.0)));

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let has_image = layout.pages[0]
        .boxes
        .iter()
        .any(|b| matches!(b, LayoutBox::Image(_)));

    assert!(has_image);
}

#[test]
fn long_document_paginates_to_multiple_pages() {
    let mut doc = Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..100 {
        doc.sections[0].blocks.push(Block::Paragraph(Paragraph::with_text(
            format!("Paragraph {i} with enough text to consume vertical space on the page."),
        )));
    }

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    assert!(layout.pages.len() > 1);
    assert_eq!(engine.page_count(), layout.pages.len());
}

#[test]
fn table_cell_background_propagates_to_layout() {
    use tw_model::Color;

    let mut doc = Document::new();
    doc.sections[0].blocks.push(Block::Table(Table::new(1, 1)));
    if let Some(table) = doc.sections[0].blocks[1].table_mut() {
        table.rows[0].cells[0].format.background = Some(Color {
            r: 255,
            g: 200,
            b: 200,
            a: 255,
        });
    }

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let cell = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::Table(t) => t.cells.first(),
            _ => None,
        })
        .expect("table cell");

    assert!(cell.background.is_some());
}
