//! U-F11-S3-word-art-display-list — WordArt arc path and inner glyphs.

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, ShapeBlock, ShapeKind};
use tw_render::DisplayListBuilder;

fn doc_with_word_art(text: &str) -> tw_model::Document {
    let mut doc = tw_model::Document::new();
    doc.sections[0]
        .blocks
        .push(Block::ShapeBlock(ShapeBlock::word_art(text, 220.0, 72.0)));
    doc
}

#[test]
fn u_f11_s3_word_art_display_list() {
    let doc = doc_with_word_art("Style");
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];

    assert!(
        page.boxes.iter().any(|b| matches!(b, LayoutBox::Shape(_))),
        "WordArt shape should appear in layout"
    );
    assert!(
        page.boxes.iter().any(|b| matches!(b, LayoutBox::TextLine(_))),
        "WordArt inner paragraph should produce text lines"
    );

    let list = DisplayListBuilder::from_page_without_atlas(page, 1);
    assert!(
        !list.path_batch.points.is_empty(),
        "WordArt should emit decorative arc path segments"
    );
    assert!(
        !list.atlas_batch.transforms.is_empty(),
        "WordArt text should emit glyph batch transforms"
    );
}

#[test]
fn u_f11_s3_text_box_display_list_has_shape_and_text() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks.push(Block::ShapeBlock(ShapeBlock::text_box(
        180.0,
        90.0,
        tw_model::ShapeStyle::inserted_default(),
    )));

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];
    let list = DisplayListBuilder::from_page_without_atlas(page, 1);

    assert!(
        page.boxes.iter().any(|b| {
            matches!(
                b,
                LayoutBox::Shape(s) if s.shape_type == ShapeKind::TextBox
            )
        }),
        "text box shape layout expected"
    );
    assert!(
        !list.rect_batch.rects.is_empty(),
        "text box fill/stroke should produce rect batch entries"
    );
}
