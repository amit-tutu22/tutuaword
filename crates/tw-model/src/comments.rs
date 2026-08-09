//! Comment threads anchored in the document body (F17.S3).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Block, NodeId, Paragraph};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentMessage {
    pub id: NodeId,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub body: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentThread {
    pub id: NodeId,
    pub comment_id: i32,
    pub anchor_run: NodeId,
    pub anchor_offset: usize,
    pub messages: Vec<CommentMessage>,
    pub resolved: bool,
}

impl CommentThread {
    pub fn new(
        comment_id: i32,
        anchor_run: NodeId,
        anchor_offset: usize,
        author: impl Into<String>,
        body_text: impl Into<String>,
    ) -> Self {
        let author = author.into();
        Self {
            id: NodeId::new(),
            comment_id,
            anchor_run,
            anchor_offset,
            messages: vec![CommentMessage {
                id: NodeId::new(),
                author: author.clone(),
                timestamp: Utc::now(),
                body: vec![Block::Paragraph(Paragraph::with_text(body_text))],
            }],
            resolved: false,
        }
    }
}
