use crate::undo::{EditSessionInner, TransactionGuard};
use crate::{Command, DocRange, EditError, EditResult};
use std::time::Duration;
use tw_model::Document;

pub struct EditSession {
    inner: EditSessionInner,
}

impl EditSession {
    pub fn new() -> Self {
        Self {
            inner: EditSessionInner::new(),
        }
    }

    pub fn from_document(doc: Document) -> Self {
        Self {
            inner: EditSessionInner::from_document(doc),
        }
    }

    pub fn with_coalesce_window(window: Duration) -> Self {
        Self {
            inner: EditSessionInner::with_coalesce_window(window),
        }
    }

    /// Test hook: override the coalescing clock (defaults to `Instant::now()`).
    #[doc(hidden)]
    pub fn set_test_clock(&mut self, now: std::time::Instant) {
        self.inner.set_test_clock(now);
    }

    pub fn apply(&mut self, command: Command) -> Result<EditResult, EditError> {
        self.inner.apply(command)
    }

    pub fn begin_transaction(&mut self, selection: Option<DocRange>) -> TransactionGuard<'_> {
        self.inner.begin_transaction(selection)
    }

    pub fn undo(&mut self) -> Result<Option<EditResult>, EditError> {
        Ok(self.inner.undo()?.map(|(result, _)| result))
    }

    pub fn redo(&mut self) -> Result<Option<EditResult>, EditError> {
        self.inner.redo()
    }

    pub fn can_undo(&self) -> bool {
        self.inner.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.inner.can_redo()
    }

    pub fn undo_stack_len(&self) -> usize {
        self.inner.undo_stack_len()
    }

    pub fn selection_for_undo(&self, index: usize) -> Option<&DocRange> {
        self.inner.selection_for_undo(index)
    }

    pub fn undo_selection(&mut self) -> Result<Option<DocRange>, EditError> {
        Ok(self.inner.undo()?.and_then(|(_, sel)| sel))
    }
}

impl std::ops::Deref for EditSession {
    type Target = EditSessionInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for EditSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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
    use tw_model::CharFormat;

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
    fn coalesced_undo_redo_100_chars() {
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

        assert_eq!(session.undo_stack_len(), 1);

        session.undo().unwrap();
        assert_eq!(
            session.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text(),
            ""
        );

        session.redo().unwrap();
        assert_eq!(
            session.document.sections[0].blocks[0]
                .paragraph()
                .unwrap()
                .full_text(),
            "x".repeat(100)
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
