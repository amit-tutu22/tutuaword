use crate::{Document, ListMarkerFormat, NodeId, Paragraph};

/// One navigable heading / outline-numbered paragraph.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OutlineEntry {
    pub paragraph_id: NodeId,
    /// Word `outlineLvl` (0 = top level, up to 8).
    pub level: u8,
    pub text: String,
    pub run_id: NodeId,
}

/// Returns outline level from a built-in heading style name, if any.
pub fn outline_level_from_style_name(name: &str) -> Option<u8> {
    if name == "Heading 1" {
        return Some(0);
    }
    if let Some(suffix) = name.strip_prefix("Heading ") {
        if let Ok(n) = suffix.parse::<u8>() {
            if (1..=9).contains(&n) {
                return Some(n - 1);
            }
        }
    }
    None
}

/// Whether a numbering definition level should appear in the document outline.
pub fn numbering_contributes_to_outline(
    doc: &Document,
    numbering_id: u32,
    level: u32,
) -> bool {
    doc.settings
        .numbering
        .get(numbering_id)
        .and_then(|def| def.levels.iter().find(|l| l.level == level))
        .is_some_and(|lvl| lvl.format != ListMarkerFormat::Bullet)
}

/// Resolved outline level for navigation (direct, style, numbering, or heading name).
pub fn resolved_outline_level(doc: &Document, para: &Paragraph) -> Option<u8> {
    let resolved = doc
        .styles
        .resolve_para_format(para.style_id, &para.format);

    if let Some(level) = resolved.outline_level {
        return (level <= 8).then_some(level);
    }

    if let Some(style_id) = para.style_id {
        if let Some(name) = doc
            .styles
            .paragraph_styles
            .get(&style_id)
            .map(|s| s.name.as_str())
        {
            if let Some(level) = outline_level_from_style_name(name) {
                return Some(level);
            }
        }
    }

    if let Some(nr) = resolved.numbering {
        if numbering_contributes_to_outline(doc, nr.numbering_id, nr.level) {
            return Some(nr.level.min(8) as u8);
        }
    }

    None
}

/// Walk the document and collect outline entries in block order.
pub fn document_outline(doc: &Document) -> Vec<OutlineEntry> {
    let mut entries = Vec::new();
    for section in &doc.sections {
        for block in &section.blocks {
            let Some(para) = block.paragraph() else {
                continue;
            };
            let Some(level) = resolved_outline_level(doc, para) else {
                continue;
            };
            let text = para.full_text().trim().to_string();
            if text.is_empty() {
                continue;
            }
            let Some(run_id) = para.runs.first().map(|r| r.id) else {
                continue;
            };
            entries.push(OutlineEntry {
                paragraph_id: para.id,
                level,
                text,
                run_id,
            });
        }
    }
    entries
}
