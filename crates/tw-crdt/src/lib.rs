use std::collections::HashMap;
use thiserror::Error;
use tw_edit::Command;
use tw_model::{Document, NodeId};

#[derive(Debug, Error)]
pub enum CrdtError {
    #[error("crdt translation not implemented")]
    NotImplemented,
    #[error("failed to serialize document: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Placeholder for Yjs map values until `yrs` integration (Phase 5).
pub type YMap = HashMap<String, YValue>;

#[derive(Debug, Clone, PartialEq)]
pub enum YValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct YNode {
    pub node_type: String,
    pub attrs: YMap,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrdtOperation {
    Insert {
        node_id: NodeId,
        offset: u32,
        text: String,
        attrs: YMap,
    },
    Delete {
        node_id: NodeId,
        offset: u32,
        length: u32,
    },
    SetAttr {
        node_id: NodeId,
        key: String,
        value: YValue,
    },
    InsertNode {
        parent_id: NodeId,
        index: u32,
        node: YNode,
    },
    DeleteNode {
        node_id: NodeId,
    },
    MoveNode {
        node_id: NodeId,
        new_parent: NodeId,
        new_index: u32,
    },
}

pub trait CrdtTranslator {
    fn local_to_crdt(&self, command: &Command) -> Result<Vec<CrdtOperation>, CrdtError>;
    fn crdt_to_local(&self, ops: &[CrdtOperation]) -> Result<Vec<Command>, CrdtError>;
}

/// Basic CRDT translator stub: embeds a full document JSON snapshot in a YMap
/// attached to each translated operation. Replace with fine-grained ops when
/// `yrs` integration lands (Phase 5).
pub struct YjsTranslator {
    document: Document,
}

impl YjsTranslator {
    pub fn new(document: Document) -> Self {
        Self { document }
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = document;
    }

    fn snapshot_attrs(&self) -> Result<YMap, CrdtError> {
        let json = serde_json::to_string(&self.document)?;
        let mut attrs = HashMap::new();
        attrs.insert("document".to_string(), YValue::String(json));
        Ok(attrs)
    }
}

impl CrdtTranslator for YjsTranslator {
    fn local_to_crdt(&self, command: &Command) -> Result<Vec<CrdtOperation>, CrdtError> {
        let attrs = self.snapshot_attrs()?;

        let op = match command {
            Command::InsertText {
                run_id,
                offset,
                text,
            } => CrdtOperation::Insert {
                node_id: *run_id,
                offset: *offset as u32,
                text: text.clone(),
                attrs,
            },
            _ => {
                // Basic stub: non-text commands carry the full document snapshot
                // in attrs until fine-grained CRDT ops are implemented.
                CrdtOperation::Insert {
                    node_id: NodeId::default(),
                    offset: 0,
                    text: String::new(),
                    attrs,
                }
            }
        };

        Ok(vec![op])
    }

    fn crdt_to_local(&self, _ops: &[CrdtOperation]) -> Result<Vec<Command>, CrdtError> {
        Err(CrdtError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_to_crdt_embeds_document_json_in_ymap() {
        let doc = Document::with_paragraph("hello");
        let translator = YjsTranslator::new(doc);
        let run_id = translator.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .id;

        let ops = translator
            .local_to_crdt(&Command::InsertText {
                run_id,
                offset: 0,
                text: "x".into(),
            })
            .unwrap();

        assert_eq!(ops.len(), 1);
        let CrdtOperation::Insert { attrs, .. } = &ops[0] else {
            panic!("expected Insert");
        };
        let YValue::String(json) = attrs.get("document").unwrap() else {
            panic!("expected document JSON string");
        };
        assert!(json.contains("hello"));
    }
}
