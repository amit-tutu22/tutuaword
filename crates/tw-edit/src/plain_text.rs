//! Map offsets in [`tw_core::snapshot::document_plain_text`] back to document positions.

use tw_model::{Block, Document};

use crate::command::{DocPosition, DocRange};
use crate::EditError;

struct PlainLine<'a> {
    para: &'a tw_model::Paragraph,
}

impl<'a> PlainLine<'a> {
    fn visible_len(&self) -> usize {
        self.para.visible_text().len()
    }

    fn position_at(&self, local: usize) -> Result<DocPosition, EditError> {
        let mut idx = 0usize;
        for run in &self.para.runs {
            if run.format.hidden == Some(true) {
                continue;
            }
            let text = run.text();
            let len = text.len();
            if local < idx + len {
                return Ok(DocPosition {
                    run_id: run.id,
                    char_offset: local - idx,
                });
            }
            idx += len;
        }
        if local == idx {
            return self.end_position();
        }
        Err(EditError::InvalidRange)
    }

    fn end_position(&self) -> Result<DocPosition, EditError> {
        let mut last: Option<DocPosition> = None;
        for run in &self.para.runs {
            if run.format.hidden == Some(true) {
                continue;
            }
            if matches!(run.content, tw_model::RunContent::Text(_)) {
                last = Some(DocPosition {
                    run_id: run.id,
                    char_offset: run.text().len(),
                });
            }
        }
        last.ok_or(EditError::InvalidRange)
    }
}

fn collect_lines<'a>(doc: &'a Document) -> Vec<PlainLine<'a>> {
    let mut lines = Vec::new();
    for section in &doc.sections {
        push_block_lines(&section.blocks, &mut lines);
    }
    lines
}

/// Plain text from body blocks only (matches spell-check input).
pub fn body_plain_text(doc: &Document) -> String {
    let lines = collect_lines(doc);
    lines
        .iter()
        .map(|l| l.para.visible_text())
        .collect::<Vec<_>>()
        .join("\n")
}

fn push_block_lines<'a>(blocks: &'a [Block], lines: &mut Vec<PlainLine<'a>>) {
    for block in blocks {
        match block {
            Block::Paragraph(para) => lines.push(PlainLine { para }),
            Block::Table(table) => {
                for row in &table.rows {
                    for cell in &row.cells {
                        push_block_lines(&cell.blocks, lines);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Locate a byte offset in the document plain-text stream (paragraphs joined with `\n`).
pub fn doc_position_at_plain_offset(
    doc: &Document,
    target: usize,
) -> Result<DocPosition, EditError> {
    let lines = collect_lines(doc);
    let mut global = 0usize;
    for (index, line) in lines.iter().enumerate() {
        let len = line.visible_len();
        if target < global + len {
            return line.position_at(target - global);
        }
        global += len;
        let has_next = index + 1 < lines.len();
        if has_next {
            if target == global {
                return line.end_position();
            }
            global += 1;
            if target < global {
                return line.end_position();
            }
        }
    }
    Err(EditError::InvalidRange)
}

/// Build a document-order range from plain-text byte offsets.
pub fn doc_range_from_plain_text(
    doc: &Document,
    start: usize,
    end: usize,
) -> Result<DocRange, EditError> {
    if start >= end {
        return Err(EditError::InvalidRange);
    }
    Ok(DocRange {
        start: doc_position_at_plain_offset(doc, start)?,
        end: doc_position_at_plain_offset(doc, end)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::Document;

    #[test]
    fn maps_plain_offset_into_first_paragraph() {
        let doc = Document::with_paragraph("hello world");
        let pos = doc_position_at_plain_offset(&doc, 6).expect("position");
        let para = doc.sections[0].blocks[0].paragraph().unwrap();
        let run = para.runs.iter().find(|r| r.id == pos.run_id).unwrap();
        assert!(run.text()[pos.char_offset..].starts_with("world"));
    }
}
