use tw_layout::{LayoutBox, LayoutEngine, TextLine};
use tw_model::{Block, Document, Paragraph};

fn first_line(text: &str) -> TextLine {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(text.to_string()))];

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.clone()),
            _ => None,
        })
        .expect("text line")
}

#[test]
fn tabs_do_not_render_as_notdef_boxes() {
    let line = first_line("Age\t\t\t:\t42 years");
    assert!(
        line.glyphs.iter().all(|g| g.glyph_id != 0),
        "tab characters must not be shaped into .notdef glyphs"
    );
}

#[test]
fn a_tab_advances_to_the_next_half_inch_stop() {
    let line = first_line("A\tB");
    let origin = line.x;

    // "A" is well under 36pt wide, so "B" must land on the first stop.
    let after_tab = line
        .glyphs
        .last()
        .expect("glyph for B after the tab");
    let offset = after_tab.x - origin;
    assert!(
        (offset - 36.0).abs() < 2.0,
        "expected the glyph after a tab near the 36pt stop, got {offset}"
    );
}

#[test]
fn consecutive_tabs_each_advance_one_stop() {
    let one = first_line("A\tB");
    let two = first_line("A\t\tB");
    let advance = (two.glyphs.last().unwrap().x) - (one.glyphs.last().unwrap().x);
    assert!(
        (advance - 36.0).abs() < 2.0,
        "a second tab should add one more 36pt stop, added {advance}"
    );
}

#[test]
fn indenting_a_paragraph_does_not_move_its_tab_grid() {
    // Word measures the default tab grid from the left margin, so an indented
    // paragraph and a flush one land their tabs on the same stops.
    let flush = first_line("A\tB");

    let mut doc = Document::new();
    let mut para = Paragraph::with_text("A\tB".to_string());
    para.format.indent_left = Some(18.0);
    doc.sections[0].blocks = vec![Block::Paragraph(para)];
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let indented = layout.pages[0]
        .boxes
        .iter()
        .find_map(|b| match b {
            LayoutBox::TextLine(l) => Some(l.clone()),
            _ => None,
        })
        .expect("text line");

    assert!(
        indented.x > flush.x,
        "the indented paragraph should start further right"
    );
    let flush_tab = flush.glyphs.last().unwrap().x;
    let indented_tab = indented.glyphs.last().unwrap().x;
    assert!(
        (indented_tab - flush_tab).abs() < 1.0,
        "tab stop moved with the indent: {flush_tab} vs {indented_tab}"
    );
}

#[test]
fn a_tab_never_moves_the_cursor_backwards() {
    // Text longer than one stop must push the tab out to the following stop.
    let line = first_line("A considerably longer label than one stop\tvalue");
    let origin = line.x;
    let mut previous = origin;
    for g in &line.glyphs {
        assert!(
            g.x >= previous - 1.0,
            "glyph at {} went backwards from {}",
            g.x,
            previous
        );
        previous = g.x;
    }
}
