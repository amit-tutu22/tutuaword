//! SmartArt node body text is hittable for caret placement.

use tw_edit::{Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::{Block, DiagramKind, ShapeBlock};

#[test]
fn u_f12_smartart_node_text_is_hittable_and_editable() {
    let mut doc = tw_model::Document::new();
    doc.sections[0].blocks = vec![Block::ShapeBlock(ShapeBlock::diagram(432.0, 216.0))];
    let shape = doc.sections[0].blocks[0].shape().unwrap();
    let run_id = shape.paragraphs[0].runs[0].id;
    let rects = tw_layout::diagram_node_rects(
        shape.diagram_kind,
        72.0, // layout uses page margins; use relative via layout engine below
        72.0,
        shape.shape.width,
        shape.shape.height,
    );
    assert_eq!(rects.len(), 3);

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&doc);
    let page = &layout.pages[0];
    let shape_box = page.boxes.iter().find_map(|b| match b {
        tw_layout::LayoutBox::Shape(s) => Some(s),
        _ => None,
    });
    let shape_layout = shape_box.expect("diagram shape box");
    let node = tw_layout::diagram_node_rects(
        tw_model::DiagramKind::Process,
        shape_layout.x,
        shape_layout.y,
        shape_layout.width,
        shape_layout.height,
    )[0];
    let map = engine.line_map(0).expect("line map");
    let hit = map
        .hit_test(node[0] + node[2] * 0.5, node[1] + node[3] * 0.5)
        .expect("first SmartArt node must accept caret");
    assert_eq!(hit.run_id, run_id);

    let mut session = EditSession::new();
    session.document = doc;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Step 1".into(),
        })
        .unwrap();
    let text: String = session.document.sections[0].blocks[0]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs
        .iter()
        .map(|r| r.text())
        .collect();
    assert_eq!(text, "Step 1");
}

#[test]
fn hierarchy_node_emits_glyphs_after_typing() {
    let mut session = EditSession::new();
    let after = match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        other => panic!("expected paragraph, got {other:?}"),
    };
    session
        .apply(Command::InsertDiagram {
            after_block_id: after,
            width: 432.0,
            height: 216.0,
            kind: tw_model::DiagramKind::Hierarchy,
        })
        .unwrap();

    let run_id = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs[0]
        .id;
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Lead".into(),
        })
        .unwrap();

    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(&session.document);
    let has_lead = layout.pages[0].boxes.iter().any(|b| match b {
        tw_layout::LayoutBox::TextLine(line) => line
            .glyphs
            .iter()
            .any(|g| !g.codepoint.is_whitespace()),
        _ => false,
    });
    assert!(
        has_lead,
        "SmartArt node text must rasterize glyphs so typing is visible"
    );
}


#[test]
fn arrow_right_from_filled_node_reaches_next() {
    let mut session = EditSession::new();
    let after = match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        other => panic!("expected paragraph, got {other:?}"),
    };
    session
        .apply(Command::InsertDiagram {
            after_block_id: after,
            width: 432.0,
            height: 216.0,
            kind: DiagramKind::Process,
        })
        .unwrap();
    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    let run0 = shape.paragraphs[0].runs[0].id;
    let run1 = shape.paragraphs[1].runs[0].id;
    let run2 = shape.paragraphs[2].runs[0].id;
    session
        .apply(Command::InsertText {
            run_id: run0,
            offset: 0,
            text: "sdvsvsdsvdv".into(),
        })
        .unwrap();

    let mut engine = LayoutEngine::new();
    engine.layout_document(&session.document);
    let map = engine.line_map(0).expect("line map");
    let (x_end, y, _) = map.caret_at(run0, 11).expect("end of node0 text");
    println!("node0 end caret ({x_end}, {y})");

    let mut found = None;
    for d in [2.0f32, 24.0, 60.0, 110.0, 180.0, 260.0, 360.0, 480.0] {
        let hit = map.hit_test(x_end + d, y);
        println!("+{d} -> {:?}", hit.as_ref().map(|h| h.run_id));
        if let Some(h) = &hit {
            if h.run_id == run1 {
                found = Some(d);
                break;
            }
        }
    }
    assert!(found.is_some(), "Right probes from node0 end must reach node1, got none");
    println!("reached node1 at +{}", found.unwrap());

    // Node1/2 centers should hit their own empty runs.
    let shape_layout = engine.page_layout(0).unwrap().boxes.iter().find_map(|b| match b {
        tw_layout::LayoutBox::Shape(s) => Some(s.clone()),
        _ => None,
    }).unwrap();
    let rects = tw_layout::diagram_node_rects(
        DiagramKind::Process,
        shape_layout.x,
        shape_layout.y,
        shape_layout.width,
        shape_layout.height,
    );
    for (i, run) in [run1, run2].into_iter().enumerate() {
        let r = rects[i + 1];
        let hit = map.hit_test(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5).expect("node hit");
        assert_eq!(hit.run_id, run, "node {} center", i + 1);
    }
}

#[test]
fn filled_node_frame_owns_blank_clicks_inside_box() {
    let mut session = EditSession::new();
    let after = match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        other => panic!("expected paragraph, got {other:?}"),
    };
    session
        .apply(Command::InsertDiagram {
            after_block_id: after,
            width: 432.0,
            height: 216.0,
            kind: DiagramKind::Process,
        })
        .unwrap();
    let run0 = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .paragraphs[0]
        .runs[0]
        .id;
    session
        .apply(Command::InsertText {
            run_id: run0,
            offset: 0,
            text: "Hi".into(),
        })
        .unwrap();

    let mut engine = LayoutEngine::new();
    engine.layout_document(&session.document);
    let map = engine.line_map(0).unwrap();
    let shape_layout = engine.page_layout(0).unwrap().boxes.iter().find_map(|b| match b {
        tw_layout::LayoutBox::Shape(s) => Some(s.clone()),
        _ => None,
    }).unwrap();
    let node0 = tw_layout::diagram_node_rects(
        DiagramKind::Process,
        shape_layout.x,
        shape_layout.y,
        shape_layout.width,
        shape_layout.height,
    )[0];
    // Click near the right padding of node 0 (past short "Hi" glyphs).
    let hit = map
        .hit_test(node0[0] + node0[2] * 0.85, node0[1] + node0[3] * 0.5)
        .expect("padding click");
    assert_eq!(
        hit.run_id, run0,
        "blank area inside a filled SmartArt box must stay on that node"
    );
}
