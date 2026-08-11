//! Apply AI text suggestions as undoable document Commands (F28.S2).

use tw_edit::{replace_run_range, replace_run_range_commands, Command, EditSession};
use tw_model::NodeId;

use crate::provider::AiError;
use crate::service::AiResponse;

/// Map an [`AiResponse`] text suggestion into DeleteRange/InsertText commands.
pub fn suggestion_to_commands(
    run_id: NodeId,
    start: usize,
    end: usize,
    response: &AiResponse,
) -> Result<Vec<Command>, AiError> {
    let text = match response {
        AiResponse::TextSuggestion { text } => text.as_str(),
        _ => {
            return Err(AiError::CompletionFailed(
                "AI response is not a text suggestion".into(),
            ))
        }
    };
    Ok(replace_run_range_commands(run_id, start, end, text))
}

/// Apply a rewrite / grammar suggestion to a run range (single undo step).
pub fn apply_text_suggestion(
    session: &mut EditSession,
    run_id: NodeId,
    start: usize,
    end: usize,
    suggestion: impl AsRef<str>,
) -> Result<(), AiError> {
    replace_run_range(session, run_id, start, end, suggestion.as_ref())
        .map_err(|e| AiError::CompletionFailed(e.to_string()))
}

/// Apply [`AiResponse::TextSuggestion`] to the document.
pub fn apply_ai_response(
    session: &mut EditSession,
    run_id: NodeId,
    start: usize,
    end: usize,
    response: &AiResponse,
) -> Result<(), AiError> {
    let text = match response {
        AiResponse::TextSuggestion { text } => text.as_str(),
        _ => {
            return Err(AiError::CompletionFailed(
                "AI response is not a text suggestion".into(),
            ))
        }
    };
    apply_text_suggestion(session, run_id, start, end, text)
}
