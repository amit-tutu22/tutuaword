//! F08.S2 — PAGE field evaluates per page in header layout (U-F08-S2-page-field-increments).

#[path = "f08_layout_helpers/mod.rs"]
mod helpers;

use helpers::{header_line, layout, margin_top, paginated_document};
use tw_model::{
    Block, FieldData, FieldEvalContext, FieldType, HeaderFooter, HeaderFooterType, NodeId,
    Paragraph, Run, RunContent, evaluate_field,
};

#[test]
fn u_f08_s2_page_field_increments() {
    let mut doc = paginated_document(80);
    let field_run = Run {
        id: NodeId::new(),
        format: Default::default(),
        content: RunContent::Field(FieldData {
            field_type: FieldType::Page,
            instruction: Some(" PAGE ".into()),
            display_text: None,
            form: None,
            merge_name: None,
        }),
        revision: None,
    };
    let field_id = field_run.id;
    doc.sections[0].headers.insert(
        HeaderFooterType::Default,
        HeaderFooter {
            blocks: vec![Block::Paragraph(Paragraph {
                id: NodeId::new(),
                format: Default::default(),
                style_id: None,
                runs: vec![field_run],
            })],
            plain_text: None,
        },
    );

    let document_layout = layout(&doc);
    assert!(
        document_layout.pages.len() >= 2,
        "fixture should paginate, got {} pages",
        document_layout.pages.len()
    );

    let margin = margin_top(&doc);
    let page_count = document_layout.pages.len() as u32;
    let field = FieldData {
        field_type: FieldType::Page,
        instruction: Some(" PAGE ".into()),
        display_text: None,
        form: None,
            merge_name: None,
    };

    for (index, page) in document_layout.pages.iter().enumerate().take(2) {
        let ctx = FieldEvalContext::for_page(index as u32, page_count);
        let expected = evaluate_field(&field, &ctx);
        let line = header_line(page, margin)
            .unwrap_or_else(|| panic!("page {} missing header line", index + 1));
        assert!(
            line.run_map.iter().any(|(_, _, id, _)| *id == field_id),
            "page {} header should reference PAGE field run",
            index + 1
        );
        assert!(
            !line.glyphs.is_empty(),
            "page {} header should shape PAGE field as {expected:?}",
            index + 1
        );
        if index == 1 {
            assert_eq!(expected, "2", "page 2 PAGE field should evaluate to 2");
        }
    }
}
