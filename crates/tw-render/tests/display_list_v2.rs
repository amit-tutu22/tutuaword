use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, Document, ImageBlock, Table};
use tw_render::DisplayListBuilder;

#[test]
fn display_list_v2_round_trip_preserves_glyphs() {
    let doc = Document::with_paragraph("Hello World");
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = layout.pages.first().unwrap();
    let list = DisplayListBuilder::from_page(page, engine.atlas(), 42);
    let bytes = DisplayListBuilder::to_bytes(&list);
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();

    assert_eq!(decoded.version, 42);
    assert!(!decoded.atlas_batch.transforms.is_empty());
    assert_eq!(
        decoded.atlas_batch.transforms.len(),
        list.atlas_batch.transforms.len()
    );
}

#[test]
fn display_list_v2_includes_table_grid_paths() {
    let mut doc = Document::new();
    doc.sections[0]
        .blocks
        .push(Block::Table(Table::new(2, 2)));

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = layout.pages.first().unwrap();
    let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);
    let bytes = DisplayListBuilder::to_bytes(&list);
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();

    assert!(!decoded.path_batch.points.is_empty());
}

#[test]
fn display_list_v2_includes_image_batch() {
    let mut doc = Document::new();
    doc.sections[0]
        .blocks
        .push(Block::ImageBlock(ImageBlock::placeholder(100.0, 80.0)));

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = layout.pages.first().unwrap();
    let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);

    assert_eq!(list.image_batch.transforms.len(), 2);
    assert_eq!(list.image_batch.sizes.len(), 2);
    assert_eq!(list.image_batch.asset_ids.len(), 1);

    let bytes = DisplayListBuilder::to_bytes(&list);
    let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.image_batch.asset_ids.len(), 1);
}

#[test]
fn display_list_v2_includes_cell_background_rects() {
    use tw_model::Color;

    let mut doc = Document::new();
    doc.sections[0].blocks.push(Block::Table(Table::new(1, 1)));
    if let Some(table) = doc.sections[0].blocks[1].table_mut() {
        table.rows[0].cells[0].format.background = Some(Color::BLACK);
    }

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = layout.pages.first().unwrap();
    let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);

    assert!(!list.rect_batch.rects.is_empty());
}

#[test]
fn multi_page_document_produces_per_page_display_lists() {
    let mut doc = Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..80 {
        doc.sections[0].blocks.push(Block::Paragraph(tw_model::Paragraph::with_text(
            format!("Fill paragraph {i} with content."),
        )));
    }

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    assert!(layout.pages.len() > 1);

    for page in &layout.pages {
        let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);
        let bytes = DisplayListBuilder::to_bytes(&list);
        assert!(!bytes.is_empty());
        let has_content = page.boxes.iter().any(|b| match b {
            LayoutBox::TextLine(l) => !l.glyphs.is_empty(),
            LayoutBox::Table(_) | LayoutBox::Image(_) | LayoutBox::Shape(_) => true,
            LayoutBox::Rect { .. } => true,
        });
        assert!(has_content || page.page_index > 0);
    }
}
