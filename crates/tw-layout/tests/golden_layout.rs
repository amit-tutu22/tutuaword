//! Layout golden snapshot: stable fingerprint of glyph positions for regression gates.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Alignment, Block, Document, Paragraph, ParaFormat};

fn layout_fingerprint(doc: &Document) -> u64 {
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(doc);
    let mut hasher = DefaultHasher::new();
    for page in &layout.pages {
        page.page_index.hash(&mut hasher);
        for layout_box in &page.boxes {
            match layout_box {
                LayoutBox::TextLine(line) => {
                    "line".hash(&mut hasher);
                    line.x.to_bits().hash(&mut hasher);
                    line.y.to_bits().hash(&mut hasher);
                    line.width.to_bits().hash(&mut hasher);
                    line.glyphs.len().hash(&mut hasher);
                    for g in &line.glyphs {
                        g.x.to_bits().hash(&mut hasher);
                        g.glyph_id.hash(&mut hasher);
                    }
                }
                LayoutBox::Table(table) => {
                    "table".hash(&mut hasher);
                    table.cells.len().hash(&mut hasher);
                }
                LayoutBox::Image(img) => {
                    "image".hash(&mut hasher);
                    img.x.to_bits().hash(&mut hasher);
                    img.y.to_bits().hash(&mut hasher);
                }
                LayoutBox::Rect { x, y, width, height, .. } => {
                    "rect".hash(&mut hasher);
                    x.to_bits().hash(&mut hasher);
                    y.to_bits().hash(&mut hasher);
                    width.to_bits().hash(&mut hasher);
                    height.to_bits().hash(&mut hasher);
                }
            }
        }
    }
    hasher.finish()
}

#[test]
fn golden_simple_paragraph_layout() {
    let doc = Document::with_paragraph("Golden layout snapshot");
    let fp = layout_fingerprint(&doc);
    // Update this constant only when layout changes are intentional.
    assert_eq!(fp, 17898229970638997377);
}

#[test]
fn justified_lines_record_space_stops_for_inter_word_expansion() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph({
        let mut p = Paragraph::with_text(
            "Word one two three four five six seven eight nine ten eleven twelve thirteen fourteen",
        );
        p.format = ParaFormat {
            alignment: Some(Alignment::Justify),
            ..Default::default()
        };
        p
    })];
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
    assert!(
        !line.justify_stops.is_empty(),
        "justified paragraphs should record space positions"
    );
}
