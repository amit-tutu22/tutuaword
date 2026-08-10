//! Selection replace via `DeleteRange` + `InsertText` (F28.S2).

use tw_model::NodeId;

use crate::{Command, EditError, EditSession};

/// Build the commands that replace `[start, end)` in a run with [text].
///
/// Empty ranges yield only `InsertText` (caret insert). Empty [text] yields
/// only `DeleteRange` when the range is non-empty.
pub fn replace_run_range_commands(
    run_id: NodeId,
    start: usize,
    end: usize,
    text: impl Into<String>,
) -> Vec<Command> {
    let text = text.into();
    let mut cmds = Vec::with_capacity(2);
    if start < end {
        cmds.push(Command::DeleteRange {
            run_id,
            start,
            end,
        });
    }
    if !text.is_empty() {
        cmds.push(Command::InsertText {
            run_id,
            offset: start,
            text,
        });
    }
    cmds
}

/// Apply a selection replace as a **single undo step** (transaction).
pub fn replace_run_range(
    session: &mut EditSession,
    run_id: NodeId,
    start: usize,
    end: usize,
    text: impl Into<String>,
) -> Result<(), EditError> {
    let cmds = replace_run_range_commands(run_id, start, end, text);
    if cmds.is_empty() {
        return Ok(());
    }
    let mut tx = session.begin_transaction(None);
    for cmd in cmds {
        tx.apply(cmd)?;
    }
    tx.commit();
    Ok(())
}
