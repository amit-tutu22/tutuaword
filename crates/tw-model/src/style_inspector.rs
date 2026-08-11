use crate::format::{CharFormat, ParaFormat, UnderlineStyle};
use crate::styles::StyleSheet;
use crate::{Document, NodeId};

/// Resolved style sources at the caret for the inspector pane (F06.S4).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleInspectorSummary {
    pub paragraph_style: Option<String>,
    pub direct_char_labels: Vec<String>,
    pub direct_para_labels: Vec<String>,
}

impl StyleInspectorSummary {
    /// Word-style one-line summary, e.g. `Heading 1 + Bold direct`.
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        parts.push(
            self.paragraph_style
                .clone()
                .unwrap_or_else(|| "Normal".into()),
        );
        for label in &self.direct_char_labels {
            parts.push(format!("{label} direct"));
        }
        for label in &self.direct_para_labels {
            parts.push(format!("{label} direct"));
        }
        parts.join(" + ")
    }
}

/// Inspect paragraph/run formatting sources at `run_id`.
pub fn style_inspector_at(doc: &Document, run_id: NodeId) -> Option<StyleInspectorSummary> {
    let loc = doc.find_run_location(run_id)?;
    let para = doc.paragraph_at_loc(loc)?;
    let run = para.runs.get(loc.run_index)?;
    let paragraph_style = para.style_id.and_then(|id| {
        doc.styles
            .paragraph_styles
            .get(&id)
            .map(|style| style.name.clone())
    });
    let style_char = doc
        .styles
        .resolve_char_format(para.style_id, &CharFormat::default());
    Some(StyleInspectorSummary {
        paragraph_style,
        direct_char_labels: direct_char_labels(&run.format, &style_char),
        direct_para_labels: direct_para_labels(&para.format, para.style_id, &doc.styles),
    })
}

fn direct_char_labels(direct: &CharFormat, style_resolved: &CharFormat) -> Vec<String> {
    let mut labels = Vec::new();
    if direct.bold.is_some() && direct.bold != style_resolved.bold {
        if direct.bold == Some(true) {
            labels.push("Bold".into());
        } else {
            labels.push("Not bold".into());
        }
    }
    if direct.italic.is_some() && direct.italic != style_resolved.italic {
        if direct.italic == Some(true) {
            labels.push("Italic".into());
        } else {
            labels.push("Not italic".into());
        }
    }
    if direct.underline.is_some() && direct.underline != style_resolved.underline {
        if direct.underline.is_some() && direct.underline != Some(UnderlineStyle::None) {
            labels.push("Underline".into());
        } else {
            labels.push("No underline".into());
        }
    }
    if direct.strikethrough.is_some() && direct.strikethrough != style_resolved.strikethrough {
        if direct.strikethrough == Some(true) {
            labels.push("Strikethrough".into());
        }
    }
    if direct.superscript.is_some() && direct.superscript != style_resolved.superscript {
        if direct.superscript == Some(true) {
            labels.push("Superscript".into());
        }
    }
    if direct.subscript.is_some() && direct.subscript != style_resolved.subscript {
        if direct.subscript == Some(true) {
            labels.push("Subscript".into());
        }
    }
    if direct.all_caps.is_some() && direct.all_caps != style_resolved.all_caps {
        if direct.all_caps == Some(true) {
            labels.push("All caps".into());
        }
    }
    if direct.small_caps.is_some() && direct.small_caps != style_resolved.small_caps {
        if direct.small_caps == Some(true) {
            labels.push("Small caps".into());
        }
    }
    if direct.hidden.is_some() && direct.hidden != style_resolved.hidden {
        if direct.hidden == Some(true) {
            labels.push("Hidden".into());
        }
    }
    if direct.font_size.is_some() && direct.font_size != style_resolved.font_size {
        if let Some(size) = direct.font_size {
            labels.push(format!("{size} pt"));
        }
    }
    if direct.font_family.is_some() && direct.font_family != style_resolved.font_family {
        if let Some(family) = &direct.font_family {
            labels.push(family.clone());
        }
    }
    labels
}

fn direct_para_labels(
    direct: &ParaFormat,
    style_id: Option<crate::StyleId>,
    styles: &StyleSheet,
) -> Vec<String> {
    let style_only = style_id
        .map(|id| styles.resolve_para_format(Some(id), &ParaFormat::default()))
        .unwrap_or_default();
    let mut labels = Vec::new();
    if direct.alignment.is_some() && direct.alignment != style_only.alignment {
        if let Some(alignment) = direct.alignment {
            labels.push(format!("{:?}", alignment).to_lowercase());
        }
    }
    if direct.indent_left.is_some() && direct.indent_left != style_only.indent_left {
        if let Some(v) = direct.indent_left {
            labels.push(format!("Left indent {v} pt"));
        }
    }
    if direct.space_before.is_some() && direct.space_before != style_only.space_before {
        if let Some(v) = direct.space_before {
            labels.push(format!("Space before {v} pt"));
        }
    }
    if direct.space_after.is_some() && direct.space_after != style_only.space_after {
        if let Some(v) = direct.space_after {
            labels.push(format!("Space after {v} pt"));
        }
    }
    labels
}
