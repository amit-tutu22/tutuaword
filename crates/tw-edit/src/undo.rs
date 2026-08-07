use crate::{apply, can_coalesce_insert, Command, DocRange, EditError, EditResult};
use std::time::{Duration, Instant};
use tw_model::Document;

/// One undo/redo step — may contain multiple coalesced or transactional commands.
#[derive(Debug, Clone)]
pub struct UndoEntry {
    pub commands: Vec<Command>,
    pub results: Vec<EditResult>,
    pub selection_before: Option<DocRange>,
    pub timestamp: Instant,
}

impl UndoEntry {
    pub fn inverse_commands(&self) -> Result<Vec<Command>, EditError> {
        let mut inverses = Vec::with_capacity(self.commands.len());
        for (cmd, result) in self.commands.iter().zip(self.results.iter()).rev() {
            inverses.push(cmd.inverse(result)?);
        }
        Ok(inverses)
    }
}

/// Groups multiple commands into a single undo step with atomic commit/abort.
pub struct Transaction {
    commands: Vec<Command>,
    results: Vec<EditResult>,
    selection_before: Option<DocRange>,
    committed: bool,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            results: Vec::new(),
            selection_before: None,
            committed: true,
        }
    }
}

pub struct TransactionGuard<'a> {
    session: &'a mut EditSessionInner,
    transaction: Transaction,
}

#[doc(hidden)]
pub struct EditSessionInner {
    pub document: Document,
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
    coalesce_window: Duration,
    test_clock: Option<Instant>,
}

impl EditSessionInner {
    pub fn new() -> Self {
        Self::with_coalesce_window(Duration::from_secs(1))
    }

    pub fn from_document(doc: Document) -> Self {
        let mut s = Self::new();
        s.document = doc;
        s
    }

    pub fn with_coalesce_window(window: Duration) -> Self {
        Self {
            document: Document::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            coalesce_window: window,
            test_clock: None,
        }
    }

    /// Test hook: override the coalescing clock (defaults to `Instant::now()`).
    #[doc(hidden)]
    pub fn set_test_clock(&mut self, now: Instant) {
        self.test_clock = Some(now);
    }

    fn now(&self) -> Instant {
        if let Some(t) = self.test_clock {
            return t;
        }
        Instant::now()
    }

    pub fn apply(&mut self, command: Command) -> Result<EditResult, EditError> {
        let result = apply(&mut self.document, command.clone())?;
        self.redo_stack.clear();
        let now = self.now();

        if let Some(entry) = self.undo_stack.last_mut() {
            if entry.commands.len() == 1
                && can_coalesce_insert(
                    &entry.commands[0],
                    &entry.results[0],
                    &command,
                    now,
                    entry.timestamp,
                    self.coalesce_window,
                )
            {
                merge_insert_into_entry(entry, &command, &result);
                return Ok(result);
            }
        }

        self.push_undo_entry(UndoEntry {
            commands: vec![command],
            results: vec![result.clone()],
            selection_before: None,
            timestamp: self.now(),
        });
        Ok(result)
    }

    pub fn push_undo_entry(&mut self, entry: UndoEntry) {
        self.redo_stack.clear();
        self.undo_stack.push(entry);
    }

    pub fn begin_transaction(&mut self, selection: Option<DocRange>) -> TransactionGuard<'_> {
        TransactionGuard {
            session: self,
            transaction: Transaction {
                commands: Vec::new(),
                results: Vec::new(),
                selection_before: selection,
                committed: false,
            },
        }
    }

    pub fn undo(&mut self) -> Result<Option<(EditResult, Option<DocRange>)>, EditError> {
        let Some(entry) = self.undo_stack.pop() else {
            return Ok(None);
        };
        let selection_before = entry.selection_before.clone();
        let mut last_result = EditResult::default();
        for inverse in entry.inverse_commands()? {
            last_result = apply(&mut self.document, inverse)?;
        }
        self.redo_stack.push(entry);
        Ok(Some((last_result, selection_before)))
    }

    pub fn redo(&mut self) -> Result<Option<EditResult>, EditError> {
        let Some(entry) = self.redo_stack.pop() else {
            return Ok(None);
        };
        let mut last_result = EditResult::default();
        for command in &entry.commands {
            last_result = apply(&mut self.document, command.clone())?;
        }
        self.undo_stack.push(entry);
        Ok(Some(last_result))
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn selection_for_undo(&self, index: usize) -> Option<&DocRange> {
        self.undo_stack
            .get(index)
            .and_then(|e| e.selection_before.as_ref())
    }
}

impl TransactionGuard<'_> {
    pub fn apply(&mut self, command: Command) -> Result<EditResult, EditError> {
        let result = apply(&mut self.session.document, command.clone())?;
        self.transaction.commands.push(command);
        self.transaction.results.push(result.clone());
        Ok(result)
    }

    pub fn commit(mut self) {
        self.transaction.committed = true;
        let tx = std::mem::take(&mut self.transaction);
        if !tx.commands.is_empty() {
            self.session.push_undo_entry(UndoEntry {
                commands: tx.commands,
                results: tx.results,
                selection_before: tx.selection_before,
                timestamp: self.session.now(),
            });
        }
    }

    pub fn abort(mut self) -> Result<(), EditError> {
        self.transaction.committed = true;
        let tx = std::mem::take(&mut self.transaction);
        for (cmd, result) in tx.commands.iter().zip(tx.results.iter()).rev() {
            let inverse = cmd.inverse(result)?;
            apply(&mut self.session.document, inverse)?;
        }
        Ok(())
    }
}

impl Drop for TransactionGuard<'_> {
    fn drop(&mut self) {
        if self.transaction.committed || self.transaction.commands.is_empty() {
            return;
        }
        for (cmd, result) in self
            .transaction
            .commands
            .iter()
            .zip(self.transaction.results.iter())
            .rev()
        {
            if let Ok(inverse) = cmd.inverse(result) {
                let _ = apply(&mut self.session.document, inverse);
            }
        }
    }
}

fn merge_insert_into_entry(entry: &mut UndoEntry, next: &Command, next_result: &EditResult) {
    let Command::InsertText {
        run_id,
        offset,
        text: next_text,
    } = next
    else {
        return;
    };

    let Command::InsertText {
        run_id: prev_run,
        offset: prev_offset,
        text: prev_text,
    } = &mut entry.commands[0]
    else {
        return;
    };

    debug_assert_eq!(*prev_run, *run_id);
    debug_assert_eq!(*prev_offset + prev_text.chars().count(), *offset);

    prev_text.push_str(next_text);
    entry.results[0]
        .affected_nodes
        .extend(next_result.affected_nodes.iter().copied());
}
