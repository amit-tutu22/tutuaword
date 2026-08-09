//! `word/comments.xml` import/export (F17.S3).

use chrono::{DateTime, Utc};
use tw_model::{Block, CommentMessage, CommentThread, NodeId, Paragraph};

pub const COMMENTS_PART: &str = "word/comments.xml";

pub fn serialize_comments_xml(threads: &[CommentThread]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    );
    for thread in threads {
        let message = thread.messages.first();
        let author = message.map(|m| m.author.as_str()).unwrap_or("Author");
        let date = message
            .map(|m| m.timestamp.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        let text = message
            .and_then(|m| m.body.first())
            .and_then(|b| b.paragraph())
            .map(|p| p.full_text())
            .unwrap_or_default();
        xml.push_str(&format!(
            r#"<w:comment w:id="{}" w:author="{}" w:date="{}" w:initials="{}">"#,
            thread.comment_id,
            escape_xml(author),
            date,
            escape_xml(&initials(author)),
        ));
        xml.push_str(&format!(
            r#"<w:p><w:r><w:t>{}</w:t></w:r></w:p>"#,
            escape_xml(&text),
        ));
        xml.push_str("</w:comment>");
    }
    xml.push_str("</w:comments>");
    xml
}

pub fn parse_comments_xml(xml: &str) -> Vec<CommentThread> {
    let mut threads = Vec::new();
    for element in crate::xml_util::split_elements(xml, "w:comment") {
        let comment_id = crate::xml_util::read_own_attr(element, "w:id")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let author = crate::xml_util::read_own_attr(element, "w:author")
            .unwrap_or_else(|| "Author".into());
        let date = crate::xml_util::read_own_attr(element, "w:date")
            .and_then(|v| DateTime::parse_from_rfc3339(&v).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let text = crate::xml_util::extract_plain_text(element);
        threads.push(CommentThread {
            id: NodeId::new(),
            comment_id,
            anchor_run: NodeId::new(),
            anchor_offset: 0,
            messages: vec![CommentMessage {
                id: NodeId::new(),
                author: author.into(),
                timestamp: date,
                body: vec![Block::Paragraph(Paragraph::with_text(text))],
            }],
            resolved: false,
        });
    }
    threads
}

fn initials(author: &str) -> String {
    author
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect()
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
