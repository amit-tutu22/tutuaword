//! Parallel semantic document tree for accessibility (F21.S1).
//!
//! Built from the model (not the display list): headings, paragraphs, and tables
//! in document order, with headings nested by outline level.

use crate::outline::resolved_outline_level;
use crate::table::cell_visible_text;
use crate::{Block, Document, NodeId, Paragraph, Table};

/// Role of a node in the accessibility tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRole {
    Heading,
    Paragraph,
    Table,
    /// Cell content container inside a table (row-major walk).
    TableCell,
}

/// One node in the parallel accessibility tree.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticNode {
    pub id: NodeId,
    pub role: SemanticRole,
    /// Heading outline level (`0` = H1 … `8` = H9). `None` for non-headings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<SemanticNode>,
}

/// Build the accessibility tree for `doc` (section body blocks only).
pub fn semantic_document_tree(doc: &Document) -> Vec<SemanticNode> {
    let mut roots = Vec::new();
    for section in &doc.sections {
        append_blocks(doc, &section.blocks, &mut roots);
    }
    roots
}

fn append_blocks(doc: &Document, blocks: &[Block], roots: &mut Vec<SemanticNode>) {
    // Stack of (outline level, index path into roots) for open headings.
    // We store indexes as paths so we can mutate nested children safely.
    let mut heading_stack: Vec<(u8, Vec<usize>)> = Vec::new();

    for block in blocks {
        match block {
            Block::Paragraph(para) => {
                if let Some(node) = paragraph_node(doc, para) {
                    if node.role == SemanticRole::Heading {
                        let level = node.level.expect("heading has level");
                        while heading_stack
                            .last()
                            .is_some_and(|(open, _)| *open >= level)
                        {
                            heading_stack.pop();
                        }
                        let path = push_child(roots, &heading_stack, node);
                        heading_stack.push((level, path));
                    } else {
                        push_child(roots, &heading_stack, node);
                    }
                }
            }
            Block::Table(table) => {
                let node = table_node(doc, table);
                push_child(roots, &heading_stack, node);
            }
            _ => {}
        }
    }
}

fn paragraph_node(doc: &Document, para: &Paragraph) -> Option<SemanticNode> {
    let text = para.full_text().trim().to_string();
    if let Some(level) = resolved_outline_level(doc, para) {
        return Some(SemanticNode {
            id: para.id,
            role: SemanticRole::Heading,
            level: Some(level),
            text,
            children: Vec::new(),
        });
    }
    if text.is_empty() {
        return None;
    }
    Some(SemanticNode {
        id: para.id,
        role: SemanticRole::Paragraph,
        level: None,
        text,
        children: Vec::new(),
    })
}

fn table_node(doc: &Document, table: &Table) -> SemanticNode {
    let mut children = Vec::new();
    let mut cell_texts = Vec::new();
    for row in &table.rows {
        for cell in &row.cells {
            let visible = cell_visible_text(cell);
            if !visible.is_empty() {
                cell_texts.push(visible);
            }
            let mut cell_children = Vec::new();
            append_blocks(doc, &cell.blocks, &mut cell_children);
            if !cell_children.is_empty() {
                children.push(SemanticNode {
                    id: cell.id,
                    role: SemanticRole::TableCell,
                    level: None,
                    text: cell_visible_text(cell),
                    children: cell_children,
                });
            }
        }
    }
    SemanticNode {
        id: table.id,
        role: SemanticRole::Table,
        level: None,
        text: cell_texts.join(" "),
        children,
    }
}

/// Append `node` under the deepest open heading (or as a root) and return its path.
fn push_child(
    roots: &mut Vec<SemanticNode>,
    heading_stack: &[(u8, Vec<usize>)],
    node: SemanticNode,
) -> Vec<usize> {
    if let Some((_, parent_path)) = heading_stack.last() {
        let parent = node_at_mut(roots, parent_path);
        parent.children.push(node);
        let mut path = parent_path.clone();
        path.push(parent.children.len() - 1);
        path
    } else {
        roots.push(node);
        vec![roots.len() - 1]
    }
}

fn node_at_mut<'a>(roots: &'a mut [SemanticNode], path: &[usize]) -> &'a mut SemanticNode {
    assert!(!path.is_empty());
    let mut node = &mut roots[path[0]];
    for &index in &path[1..] {
        node = &mut node.children[index];
    }
    node
}

/// Preorder walk yielding `(depth, node)` for tests and exporters.
pub fn walk_semantic_tree<'a>(
    roots: &'a [SemanticNode],
) -> impl Iterator<Item = (usize, &'a SemanticNode)> + 'a {
    SemanticWalk {
        stack: roots
            .iter()
            .rev()
            .map(|n| (0usize, n))
            .collect(),
    }
}

struct SemanticWalk<'a> {
    stack: Vec<(usize, &'a SemanticNode)>,
}

impl<'a> Iterator for SemanticWalk<'a> {
    type Item = (usize, &'a SemanticNode);

    fn next(&mut self) -> Option<Self::Item> {
        let (depth, node) = self.stack.pop()?;
        for child in node.children.iter().rev() {
            self.stack.push((depth + 1, child));
        }
        Some((depth, node))
    }
}
