//! Each run is laid out in the font its format asks for, not the default.

use std::collections::BTreeSet;
use tw_layout::{LayoutBox, LayoutEngine};
use tw_model::{Block, CharFormat, Document, Paragraph, Run};

fn glyph_fonts(layout: &tw_layout::DocumentLayout) -> Vec<(char, u32)> {
    let mut out = Vec::new();
    for page in &layout.pages {
        for b in &page.boxes {
            if let LayoutBox::TextLine(line) = b {
                for g in &line.glyphs {
                    out.push(('?', g.font_id));
                }
            }
        }
    }
    out
}

fn distinct_fonts(layout: &tw_layout::DocumentLayout) -> BTreeSet<u32> {
    glyph_fonts(layout).into_iter().map(|(_, f)| f).collect()
}

fn document_of(runs: Vec<Run>) -> Document {
    let mut para = Paragraph::new();
    para.runs = runs;
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(para)];
    doc
}

fn run_with(text: &str, format: CharFormat) -> Run {
    let mut run = Run::new_text(text);
    run.format = format;
    run
}

#[test]
fn two_families_in_one_paragraph_use_two_faces() {
    let doc = document_of(vec![
        run_with(
            "serif ",
            CharFormat {
                font_family: Some("Times New Roman".into()),
                ..Default::default()
            },
        ),
        run_with(
            "sans",
            CharFormat {
                font_family: Some("Arial".into()),
                ..Default::default()
            },
        ),
    ]);

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);

    assert_eq!(distinct_fonts(&layout).len(), 2);
}

#[test]
fn a_bold_run_uses_a_different_face_than_its_neighbour() {
    let doc = document_of(vec![
        run_with(
            "regular ",
            CharFormat {
                font_family: Some("Arial".into()),
                ..Default::default()
            },
        ),
        run_with(
            "bold",
            CharFormat {
                font_family: Some("Arial".into()),
                bold: Some(true),
                ..Default::default()
            },
        ),
    ]);

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);

    assert_eq!(distinct_fonts(&layout).len(), 2, "bold should not share the regular face");
}

#[test]
fn a_bold_run_is_wider_than_the_same_text_unbolded() {
    let format = CharFormat {
        font_family: Some("Arial".into()),
        font_size: Some(24.0),
        ..Default::default()
    };
    let plain = document_of(vec![run_with("Handgloves", format.clone())]);
    let bold = document_of(vec![run_with(
        "Handgloves",
        CharFormat {
            bold: Some(true),
            ..format
        },
    )]);

    let mut engine = LayoutEngine::new();
    let plain_width = line_width(&engine.layout_document(&plain));
    let bold_width = line_width(&engine.layout_document(&bold));

    assert!(
        bold_width > plain_width,
        "bold {bold_width} should exceed regular {plain_width}"
    );
}

fn line_width(layout: &tw_layout::DocumentLayout) -> f32 {
    layout
        .pages
        .iter()
        .flat_map(|p| &p.boxes)
        .find_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.width),
            _ => None,
        })
        .unwrap_or(0.0)
}

#[test]
fn every_run_inherits_the_document_default_font() {
    let mut doc = document_of(vec![
        run_with("first ", CharFormat::default()),
        run_with("second", CharFormat::default()),
    ]);
    doc.styles.defaults.char_format.font_family = Some("Times New Roman".into());

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);

    // Both runs resolve to the same face, and it is the default family's, not
    // the engine's built-in fallback.
    let fonts = distinct_fonts(&layout);
    assert_eq!(fonts.len(), 1, "both runs should share the default family");

    let mut plain = LayoutEngine::new();
    let arial = document_of(vec![run_with(
        "first second",
        CharFormat {
            font_family: Some("Arial".into()),
            ..Default::default()
        },
    )]);
    let arial_fonts = distinct_fonts(&plain.layout_document(&arial));
    assert_ne!(fonts, arial_fonts, "default family should be honoured");
}
