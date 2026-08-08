#[path = "f05_layout_helpers/mod.rs"]
mod f05_layout_helpers;

use f05_layout_helpers::list_markers;
use tw_model::{Block, Document, NumberingRef, Paragraph};

fn numbered_paragraph(text: &str, restart: bool) -> Paragraph {
    let mut para = Paragraph::with_text(text);
    para.format.numbering = Some(NumberingRef {
        numbering_id: 2,
        level: 0,
    });
    if restart {
        para.format.num_restart = Some(true);
    }
    para
}

/// U-F05-S3-restart-counter: counter resets at a `num_restart` marker.
#[test]
fn restart_counter_resets_at_marker() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(numbered_paragraph("One", false)),
        Block::Paragraph(numbered_paragraph("Two", false)),
        Block::Paragraph(numbered_paragraph("Restart", true)),
        Block::Paragraph(numbered_paragraph("Four", false)),
    ];

    assert_eq!(
        list_markers(&doc),
        vec!["1.", "2.", "1.", "2."],
        "restart paragraph should reset to 1"
    );
}

#[test]
fn without_restart_markers_stay_sequential() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(numbered_paragraph("One", false)),
        Block::Paragraph(numbered_paragraph("Two", false)),
        Block::Paragraph(numbered_paragraph("Three", false)),
    ];

    assert_eq!(list_markers(&doc), vec!["1.", "2.", "3."]);
}

#[test]
fn double_restart_restarts_twice() {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![
        Block::Paragraph(numbered_paragraph("A", false)),
        Block::Paragraph(numbered_paragraph("B", true)),
        Block::Paragraph(numbered_paragraph("C", false)),
        Block::Paragraph(numbered_paragraph("D", true)),
        Block::Paragraph(numbered_paragraph("E", false)),
    ];

    assert_eq!(list_markers(&doc), vec!["1.", "1.", "2.", "1.", "2."]);
}
