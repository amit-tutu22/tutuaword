//! F21.S1 — semantic accessibility tree.

use tw_model::{
    semantic_document_tree, walk_semantic_tree, Block, Document, Paragraph, SemanticRole, Table,
};

fn heading_para(doc: &Document, style_name: &str, text: &str) -> Paragraph {
    let style_id = doc.styles.find_style_by_name(style_name).unwrap().id;
    let mut para = Paragraph::with_text(text);
    para.style_id = Some(style_id);
    para
}

#[test]
fn u_f21_s1_heading_structure() {
    let mut doc = Document::new();
    let h1 = heading_para(&doc, "Heading 1", "Chapter");
    let h2 = heading_para(&doc, "Heading 2", "Section");
    let body = Paragraph::with_text("Body text");
    doc.sections[0].blocks = vec![
        Block::Paragraph(h1),
        Block::Paragraph(h2),
        Block::Paragraph(body),
    ];

    let tree = semantic_document_tree(&doc);
    assert_eq!(tree.len(), 1, "H1 should be the only root");
    assert_eq!(tree[0].role, SemanticRole::Heading);
    assert_eq!(tree[0].level, Some(0));
    assert_eq!(tree[0].text, "Chapter");

    assert_eq!(tree[0].children.len(), 1);
    assert_eq!(tree[0].children[0].role, SemanticRole::Heading);
    assert_eq!(tree[0].children[0].level, Some(1));
    assert_eq!(tree[0].children[0].text, "Section");

    assert_eq!(tree[0].children[0].children.len(), 1);
    assert_eq!(tree[0].children[0].children[0].role, SemanticRole::Paragraph);
    assert_eq!(tree[0].children[0].children[0].text, "Body text");

    let headings: Vec<_> = walk_semantic_tree(&tree)
        .filter(|(_, n)| n.role == SemanticRole::Heading)
        .map(|(_, n)| (n.level, n.text.as_str()))
        .collect();
    assert_eq!(headings, vec![(Some(0), "Chapter"), (Some(1), "Section")]);
}

#[test]
fn u_f21_s1_sibling_h1_resets_nesting() {
    let mut doc = Document::new();
    let h1a = heading_para(&doc, "Heading 1", "One");
    let h2 = heading_para(&doc, "Heading 2", "Nested");
    let h1b = heading_para(&doc, "Heading 1", "Two");
    doc.sections[0].blocks = vec![
        Block::Paragraph(h1a),
        Block::Paragraph(h2),
        Block::Paragraph(h1b),
    ];

    let tree = semantic_document_tree(&doc);
    assert_eq!(tree.len(), 2);
    assert_eq!(tree[0].text, "One");
    assert_eq!(tree[0].children[0].text, "Nested");
    assert_eq!(tree[1].text, "Two");
    assert!(tree[1].children.is_empty());
}

#[test]
fn u_f21_s1_table_in_tree() {
    let mut doc = Document::new();
    let h1 = heading_para(&doc, "Heading 1", "Data");
    let mut table = Table::new(1, 2);
    table.rows[0].cells[0].blocks = vec![Block::Paragraph(Paragraph::with_text("A"))];
    table.rows[0].cells[1].blocks = vec![Block::Paragraph(Paragraph::with_text("B"))];
    doc.sections[0].blocks = vec![Block::Paragraph(h1), Block::Table(table)];

    let tree = semantic_document_tree(&doc);
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].role, SemanticRole::Heading);
    assert_eq!(tree[0].children.len(), 1);
    assert_eq!(tree[0].children[0].role, SemanticRole::Table);
    assert!(tree[0].children[0].text.contains('A'));
    assert!(tree[0].children[0].text.contains('B'));
    assert_eq!(tree[0].children[0].children.len(), 2);
    assert_eq!(tree[0].children[0].children[0].role, SemanticRole::TableCell);
}
