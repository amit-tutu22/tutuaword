use std::collections::HashMap;
use thiserror::Error;
use tw_edit::Command;
use tw_model::NodeId;

#[derive(Debug, Error)]
pub enum CrdtError {
    #[error("crdt translation not implemented")]
    NotImplemented,
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

pub struct YjsTranslator;

impl CrdtTranslator for YjsTranslator {
    fn local_to_crdt(&self, _command: &Command) -> Result<Vec<CrdtOperation>, CrdtError> {
        Err(CrdtError::NotImplemented)
    }

    fn crdt_to_local(&self, _ops: &[CrdtOperation]) -> Result<Vec<Command>, CrdtError> {
        Err(CrdtError::NotImplemented)
    }
}
