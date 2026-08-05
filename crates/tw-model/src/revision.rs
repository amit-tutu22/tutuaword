use crate::ids::NodeId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RevisionType {
    Insert,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Revision {
    pub id: NodeId,
    pub revision_type: RevisionType,
    pub author: String,
    pub timestamp: DateTime<Utc>,
}

impl Revision {
    pub fn insert(author: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            revision_type: RevisionType::Insert,
            author: author.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn delete(author: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            revision_type: RevisionType::Delete,
            author: author.into(),
            timestamp: Utc::now(),
        }
    }
}
