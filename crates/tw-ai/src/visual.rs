//! Visual assistance — suggest table / diagram / timeline and insert as blocks (F28.S5).

use serde_json::Value;
use tw_edit::{Command, EditError, EditSession};
use tw_model::{DiagramKind, NodeId};

use crate::context::DocumentContext;
use crate::provider::AiError;
use crate::service::{AiResponse, AiService};

/// Suggested visual insert kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisualKind {
    Table {
        rows: u32,
        cols: u32,
    },
    Diagram {
        kind: DiagramKind,
    },
    /// Timeline stages → inserted as a 1×N table block.
    Timeline {
        stages: Vec<String>,
    },
}

impl VisualKind {
    pub fn as_label(&self) -> &'static str {
        match self {
            VisualKind::Table { .. } => "table",
            VisualKind::Diagram { .. } => "diagram",
            VisualKind::Timeline { .. } => "timeline",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualSuggestion {
    pub kind: VisualKind,
    pub title: String,
    pub rationale: String,
}

impl VisualSuggestion {
    pub fn prompt_instruction(topic: &str) -> String {
        format!(
            "Suggest one visual for: {topic}. Reply with ONLY JSON, no markdown fences. \
             Schema examples: \
             {{\"kind\":\"table\",\"rows\":3,\"cols\":3,\"title\":\"…\",\"rationale\":\"…\"}} \
             {{\"kind\":\"diagram\",\"diagram\":\"process|hierarchy|cycle\",\"title\":\"…\",\"rationale\":\"…\"}} \
             {{\"kind\":\"timeline\",\"stages\":[\"A\",\"B\"],\"title\":\"…\",\"rationale\":\"…\"}}"
        )
    }
}

/// Parse model JSON into a [`VisualSuggestion`].
pub fn parse_visual_suggestion(text: &str) -> Result<VisualSuggestion, AiError> {
    let trimmed = text.trim();
    let json_slice = extract_json_object(trimmed).ok_or_else(|| {
        AiError::CompletionFailed("visual suggestion is not valid JSON".into())
    })?;
    let value: Value = serde_json::from_str(json_slice)
        .map_err(|e| AiError::CompletionFailed(format!("visual JSON: {e}")))?;

    let kind_str = value
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    let title = value
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Suggested visual")
        .to_string();
    let rationale = value
        .get("rationale")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let kind = match kind_str.as_str() {
        "table" => {
            let rows = value
                .get("rows")
                .and_then(|v| v.as_u64())
                .unwrap_or(3)
                .clamp(1, 63) as u32;
            let cols = value
                .get("cols")
                .and_then(|v| v.as_u64())
                .unwrap_or(3)
                .clamp(1, 63) as u32;
            VisualKind::Table { rows, cols }
        }
        "diagram" => {
            let diagram = value
                .get("diagram")
                .and_then(|v| v.as_str())
                .unwrap_or("process")
                .to_lowercase();
            let kind = match diagram.as_str() {
                "hierarchy" | "org" => DiagramKind::Hierarchy,
                "cycle" => DiagramKind::Cycle,
                _ => DiagramKind::Process,
            };
            VisualKind::Diagram { kind }
        }
        "timeline" => {
            let stages = value
                .get("stages")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let stages = if stages.is_empty() {
                vec!["Start".into(), "Finish".into()]
            } else {
                stages
            };
            VisualKind::Timeline { stages }
        }
        other => {
            return Err(AiError::CompletionFailed(format!(
                "unknown visual kind: {other}"
            )))
        }
    };

    Ok(VisualSuggestion {
        kind,
        title,
        rationale,
    })
}

/// Build insert Commands for a suggestion (after [after_block_id]).
pub fn visual_suggestion_commands(
    after_block_id: NodeId,
    suggestion: &VisualSuggestion,
) -> Vec<Command> {
    match &suggestion.kind {
        VisualKind::Table { rows, cols } => vec![Command::InsertTable {
            after_block_id,
            rows: *rows,
            cols: *cols,
        }],
        VisualKind::Diagram { kind } => vec![Command::InsertDiagram {
            after_block_id,
            width: 432.0,
            height: 216.0,
            kind: *kind,
        }],
        VisualKind::Timeline { stages } => {
            let cols = (stages.len() as u32).clamp(2, 8);
            vec![Command::InsertTable {
                after_block_id,
                rows: 1,
                cols,
            }]
        }
    }
}

/// Apply suggestion as a single undoable edit.
pub fn apply_visual_suggestion(
    session: &mut EditSession,
    after_block_id: NodeId,
    suggestion: &VisualSuggestion,
) -> Result<(), AiError> {
    let cmds = visual_suggestion_commands(after_block_id, suggestion);
    if cmds.is_empty() {
        return Ok(());
    }
    let mut tx = session.begin_transaction(None);
    for cmd in cmds {
        tx.apply(cmd)
            .map_err(|e: EditError| AiError::CompletionFailed(e.to_string()))?;
    }
    tx.commit();
    Ok(())
}

/// Ask the AI for a visual suggestion for [topic].
pub fn suggest_visual(
    service: &impl AiService,
    topic: &str,
    ctx: &DocumentContext,
) -> Result<VisualSuggestion, AiError> {
    let instruction = VisualSuggestion::prompt_instruction(topic);
    let response = service.generate(ctx, &instruction)?;
    let text = match response {
        AiResponse::TextSuggestion { text } => text,
        _ => {
            return Err(AiError::CompletionFailed(
                "visual suggest did not return text".into(),
            ))
        }
    };
    parse_visual_suggestion(&text)
}

fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&text[start..=end])
}
