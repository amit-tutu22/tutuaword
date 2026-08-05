use crate::{apply, Command, EditError, EditResult};
use tw_model::Document;
use tw_text::TextBuffer;

pub struct EditSession {
    pub document: Document,
    pub buffer: TextBuffer,
    undo_stack: Vec<(Command, EditResult)>,
    redo_stack: Vec<(Command, EditResult)>,
}

impl EditSession {
    pub fn new() -> Self {
        let mut session = Self {
            document: Document::new(),
            buffer: TextBuffer::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };
        session.sync_buffer();
        session
    }

    pub fn from_document(doc: Document) -> Self {
        let mut session = Self {
            document: doc,
            buffer: TextBuffer::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };
        session.sync_buffer();
        session
    }

    fn sync_buffer(&mut self) {
        self.buffer = TextBuffer::new();
        for para in self.document.paragraphs_mut() {
            for run in &para.runs {
                self.buffer.register(run.id, run.text());
            }
        }
    }

    pub fn apply(&mut self, command: Command) -> Result<EditResult, EditError> {
        let result = apply(&mut self.document, &mut self.buffer, command.clone())?;
        self.redo_stack.clear();
        self.undo_stack.push((command, result.clone()));
        Ok(result)
    }

    pub fn undo(&mut self) -> Result<Option<EditResult>, EditError> {
        let Some((command, result)) = self.undo_stack.pop() else {
            return Ok(None);
        };
        if let Some(inverse) = command.inverse(&result) {
            let inverse_result = apply(&mut self.document, &mut self.buffer, inverse.clone())?;
            self.redo_stack.push((inverse, inverse_result.clone()));
            return Ok(Some(inverse_result));
        }
        Ok(None)
    }

    pub fn redo(&mut self) -> Result<Option<EditResult>, EditError> {
        let Some((command, result)) = self.redo_stack.pop() else {
            return Ok(None);
        };
        let redo_result = apply(&mut self.document, &mut self.buffer, command.clone())?;
        self.undo_stack.push((command, result));
        Ok(Some(redo_result))
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl Default for EditSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::{CharFormat, NodeId};

    #[test]
    fn insert_and_undo() {
        let mut session = EditSession::new();
        let run_id = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .id;

        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "Hello".into(),
            })
            .unwrap();

        assert_eq!(
            session.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text(),
            "Hello"
        );

        session.undo().unwrap();
        assert_eq!(
            session.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text(),
            ""
        );
    }

    #[test]
    fn undo_redo_100_ops() {
        let mut session = EditSession::new();
        let run_id = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .id;

        for _ in 0..100 {
            let offset = session.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text()
                .chars()
                .count();
            session
                .apply(Command::InsertText {
                    run_id,
                    offset,
                    text: "x".into(),
                })
                .unwrap();
        }

        for _ in 0..100 {
            session.undo().unwrap();
        }
        assert_eq!(
            session.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text(),
            ""
        );

        for _ in 0..100 {
            let offset = session.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text()
                .chars()
                .count();
            session
                .apply(Command::InsertText {
                    run_id,
                    offset,
                    text: "x".into(),
                })
                .unwrap();
        }
        assert_eq!(
            session.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text()
                .len(),
            100
        );
    }

    #[test]
    fn bold_formatting() {
        let mut session = EditSession::new();
        let para = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap();
        let run_id = para.runs[0].id;

        session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: "Bold".into(),
            })
            .unwrap();

        session
            .apply(Command::SetCharFormat {
                run_id,
                start: 0,
                end: 4,
                format: CharFormat {
                    bold: Some(true),
                    ..Default::default()
                },
                merge: true,
            })
            .unwrap();

        let run = &session.document.paragraph_at(0, 0).unwrap().runs[0];
        assert_eq!(run.format.bold, Some(true));
    }
}
