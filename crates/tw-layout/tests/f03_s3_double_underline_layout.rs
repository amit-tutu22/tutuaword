//! F03.S3 — double underline decoration kind in layout output.

use tw_layout::{DecorationKind, LayoutBox, LayoutEngine};
use tw_model::{Block, CharFormat, Document, Paragraph, UnderlineStyle};

/// U-F03-S3-double-underline-layout: double underline maps to `DoubleUnderline` decorations.
#[test]
fn u_f03_s3_double_underline_layout() {
    let mut doc = Document::new();
    let mut run = tw_model::Run::new_text("Double");
    run.format = CharFormat {
        underline: Some(UnderlineStyle::Double),
        ..Default::default()
    };
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![run],
    });

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
        line.decorations
            .iter()
            .any(|d| d.kind == DecorationKind::DoubleUnderline),
        "double underline should emit DoubleUnderline decoration kind"
    );
    assert!(
        !line.decorations
            .iter()
            .any(|d| d.kind == DecorationKind::Underline),
        "double underline should not use single Underline kind"
    );
}

#[test]
fn character_spacing_widens_line() {
    let mut doc = Document::new();
    let mut spaced = tw_model::Run::new_text("Spaced");
    spaced.format = CharFormat {
        character_spacing: Some(4.0),
        ..Default::default()
    };
    doc.sections[0].blocks[0] = Block::Paragraph(Paragraph {
        id: doc.sections[0].blocks[0].paragraph().unwrap().id,
        format: Default::default(),
        style_id: None,
        runs: vec![spaced],
    });

    let plain = Document::with_paragraph("Spaced");
    let mut engine = LayoutEngine::new();
    let spaced_layout = engine.layout_document(&doc);
    let plain_layout = engine.layout_document(&plain);

    let spaced_width = spaced_layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.width),
            _ => None,
        })
        .unwrap_or(0.0);
    let plain_width = plain_layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.width),
            _ => None,
        })
        .unwrap_or(0.0);

    assert!(
        spaced_width > plain_width,
        "character spacing should widen the line (spaced={spaced_width}, plain={plain_width})"
    );
}
