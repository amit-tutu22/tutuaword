//! Smart editing — suggest headings, auto-format notes, TOC draft (F28.S6).
//!
//! Plans are applied only after explicit user accept (no unsupervised save).

use serde_json::Value;
use tw_edit::{Command, EditError, EditSession};
use tw_model::{Block, Document, NodeId};

use crate::context::DocumentContext;
use crate::provider::AiError;
use crate::service::{AiResponse, AiService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingSuggestion {
    pub paragraph_id: NodeId,
    pub paragraph_index: usize,
    pub style_name: String,
    pub preview_text: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SmartEditPlan {
    pub headings: Vec<HeadingSuggestion>,
    pub insert_toc: bool,
    pub toc_after_block_id: Option<NodeId>,
    /// Non-mutating auto-format observations (messy spacing, etc.).
    pub auto_format_notes: Vec<String>,
}

impl SmartEditPlan {
    pub fn is_empty(&self) -> bool {
        self.headings.is_empty() && !self.insert_toc
    }

    pub fn prompt_instruction() -> &'static str {
        "Analyze the numbered paragraphs and suggest heading styles. \
         Reply with ONLY JSON: \
         {\"headings\":[{\"index\":0,\"style\":\"Heading 1\"}],\"insert_toc\":true}"
    }
}

#[derive(Debug, Clone)]
struct ParaRef {
    index: usize,
    id: NodeId,
    text: String,
    style_name: Option<String>,
}

fn body_paragraphs(doc: &Document) -> Vec<ParaRef> {
    let mut out = Vec::new();
    let mut index = 0usize;
    for section in &doc.sections {
        for block in &section.blocks {
            let Block::Paragraph(para) = block else {
                continue;
            };
            let text = para.full_text();
            if text.trim().is_empty() {
                index += 1;
                continue;
            }
            let style_name = para.style_id.and_then(|id| {
                doc.styles
                    .paragraph_styles
                    .get(&id)
                    .map(|s| s.name.clone())
            });
            out.push(ParaRef {
                index,
                id: para.id,
                text,
                style_name,
            });
            index += 1;
        }
    }
    out
}

fn looks_like_heading(text: &str) -> Option<&'static str> {
    let t = text.trim();
    if t.is_empty() || t.len() > 80 {
        return None;
    }
    if t.ends_with('.') && t.len() > 40 {
        return None;
    }
    let words: Vec<&str> = t.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    let letters: String = t.chars().filter(|c| c.is_alphabetic()).collect();
    let all_caps = !letters.is_empty() && letters.chars().all(|c| c.is_uppercase());
    let title_case = words.iter().filter(|w| {
        w.chars().next().is_some_and(|c| c.is_uppercase())
    }).count()
        >= (words.len() + 1) / 2;
    let short = t.len() <= 48;
    let ends_colon = t.ends_with(':');

    if all_caps && short {
        return Some("Heading 1");
    }
    if ends_colon && short {
        return Some("Heading 2");
    }
    if short && (title_case || !t.contains('.')) {
        // First-line style titles → H1; shorter section labels → H2
        if t.len() <= 28 && title_case {
            return Some("Heading 1");
        }
        return Some("Heading 2");
    }
    None
}

fn already_heading(style: &Option<String>) -> bool {
    style
        .as_deref()
        .is_some_and(|s| s.starts_with("Heading"))
}

/// Deterministic heuristic plan (no network) — auto-format notes + heading suggestions.
pub fn analyze_document_heuristics(doc: &Document) -> SmartEditPlan {
    let paras = body_paragraphs(doc);
    let mut headings = Vec::new();
    let mut notes = Vec::new();
    let mut empty_streak = 0u32;
    let mut messy_spacing = 0u32;

    for section in &doc.sections {
        for block in &section.blocks {
            let Block::Paragraph(para) = block else {
                continue;
            };
            let text = para.full_text();
            if text.trim().is_empty() {
                empty_streak += 1;
                if empty_streak >= 2 {
                    notes.push("Multiple consecutive blank paragraphs detected".into());
                    empty_streak = 0;
                }
            } else {
                empty_streak = 0;
            }
            if text.contains("  ") || text.contains('\t') {
                messy_spacing += 1;
            }
        }
    }
    if messy_spacing > 0 {
        notes.push(format!(
            "{messy_spacing} paragraph(s) have irregular spacing"
        ));
    }
    notes.dedup();

    let mut saw_h1 = false;
    for para in &paras {
        if already_heading(&para.style_name) {
            if para.style_name.as_deref() == Some("Heading 1") {
                saw_h1 = true;
            }
            continue;
        }
        let Some(suggested) = looks_like_heading(&para.text) else {
            continue;
        };
        let style = if suggested == "Heading 1" && !saw_h1 {
            saw_h1 = true;
            "Heading 1"
        } else if suggested == "Heading 1" && saw_h1 {
            "Heading 2"
        } else {
            suggested
        };
        headings.push(HeadingSuggestion {
            paragraph_id: para.id,
            paragraph_index: para.index,
            style_name: style.into(),
            preview_text: para.text.chars().take(60).collect(),
            reason: "heuristic: short title-like paragraph".into(),
        });
    }

    let insert_toc = !headings.is_empty()
        || paras
            .iter()
            .any(|p| already_heading(&p.style_name));
    let toc_after = doc
        .sections
        .first()
        .and_then(|s| s.blocks.first())
        .and_then(|b| match b {
            Block::Paragraph(p) => Some(p.id),
            Block::Table(t) => Some(t.id),
            Block::ImageBlock(i) => Some(i.id),
            Block::ShapeBlock(s) => Some(s.id),
            _ => None,
        });

    SmartEditPlan {
        headings,
        insert_toc,
        toc_after_block_id: toc_after,
        auto_format_notes: notes,
    }
}

/// Merge AI JSON heading indices into a heuristic base plan.
pub fn parse_smart_edit_plan(text: &str, doc: &Document) -> Result<SmartEditPlan, AiError> {
    let mut plan = analyze_document_heuristics(doc);
    let trimmed = text.trim();
    let Some(start) = trimmed.find('{') else {
        return Ok(plan);
    };
    let Some(end) = trimmed.rfind('}') else {
        return Ok(plan);
    };
    if end <= start {
        return Ok(plan);
    }
    let value: Value = serde_json::from_str(&trimmed[start..=end])
        .map_err(|e| AiError::CompletionFailed(format!("smart-edit JSON: {e}")))?;

    let paras = body_paragraphs(doc);
    if let Some(arr) = value.get("headings").and_then(|v| v.as_array()) {
        let mut ai_headings = Vec::new();
        for item in arr {
            let index = item.get("index").and_then(|v| v.as_u64()).unwrap_or(9999) as usize;
            let style = item
                .get("style")
                .and_then(|v| v.as_str())
                .unwrap_or("Heading 1");
            let style = if style.starts_with("Heading") {
                style
            } else {
                "Heading 1"
            };
            if let Some(para) = paras.iter().find(|p| p.index == index) {
                if already_heading(&para.style_name) {
                    continue;
                }
                ai_headings.push(HeadingSuggestion {
                    paragraph_id: para.id,
                    paragraph_index: para.index,
                    style_name: style.into(),
                    preview_text: para.text.chars().take(60).collect(),
                    reason: "ai suggestion".into(),
                });
            }
        }
        if !ai_headings.is_empty() {
            plan.headings = ai_headings;
        }
    }
    if let Some(flag) = value.get("insert_toc").and_then(|v| v.as_bool()) {
        plan.insert_toc = flag;
    }
    Ok(plan)
}

/// Ask the model for a smart-edit plan, falling back to heuristics on parse issues.
pub fn suggest_smart_edit(
    service: &impl AiService,
    doc: &Document,
    ctx: &DocumentContext,
) -> Result<SmartEditPlan, AiError> {
    let paras = body_paragraphs(doc);
    let listing = paras
        .iter()
        .map(|p| format!("{}: {}", p.index, p.text.chars().take(80).collect::<String>()))
        .collect::<Vec<_>>()
        .join("\n");
    let instruction = format!(
        "{}\nParagraphs:\n{listing}",
        SmartEditPlan::prompt_instruction()
    );
    match service.generate(ctx, &instruction) {
        Ok(AiResponse::TextSuggestion { text }) => parse_smart_edit_plan(&text, doc),
        Ok(_) => Ok(analyze_document_heuristics(doc)),
        Err(_) => Ok(analyze_document_heuristics(doc)),
    }
}

/// Build Commands for an accepted plan (headings then TOC).
pub fn smart_edit_commands(doc: &Document, plan: &SmartEditPlan) -> Vec<Command> {
    let mut cmds = Vec::new();
    for h in &plan.headings {
        cmds.push(Command::ApplyParagraphStyle {
            paragraph_id: h.paragraph_id,
            style_name: h.style_name.clone(),
        });
    }
    if plan.insert_toc {
        if let Some(after) = plan.toc_after_block_id.or_else(|| {
            doc.sections
                .first()
                .and_then(|s| s.blocks.first())
                .and_then(|b| b.paragraph().map(|p| p.id))
        }) {
            let outline_len = plan.headings.len().max(1);
            cmds.push(Command::InsertTableOfContents {
                after_block_id: after,
                page_numbers: vec![1; outline_len],
            });
        }
    }
    cmds
}

/// Apply plan after user accept.
///
/// Heading styles are one undo step. TOC insert is applied separately because
/// `InsertTableOfContents` does not yet expose an inverse.
pub fn apply_smart_edit_plan(
    session: &mut EditSession,
    plan: &SmartEditPlan,
) -> Result<(), AiError> {
    let cmds = smart_edit_commands(&session.document, plan);
    let (styles, toc): (Vec<_>, Vec<_>) = cmds.into_iter().partition(|c| {
        matches!(c, Command::ApplyParagraphStyle { .. })
    });
    if !styles.is_empty() {
        let mut tx = session.begin_transaction(None);
        for cmd in styles {
            tx.apply(cmd)
                .map_err(|e: EditError| AiError::CompletionFailed(e.to_string()))?;
        }
        tx.commit();
    }
    for cmd in toc {
        session
            .apply(cmd)
            .map_err(|e: EditError| AiError::CompletionFailed(e.to_string()))?;
    }
    Ok(())
}
