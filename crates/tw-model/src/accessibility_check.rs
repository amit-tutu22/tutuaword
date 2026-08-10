//! Document accessibility checker (F21.S4).
//!
//! Rules: missing image alt text, empty headings, low-contrast text (WCAG AA warning).

use serde::{Deserialize, Serialize};

use crate::format::Color;
use crate::ids::NodeId;
use crate::outline::resolved_outline_level;
use crate::{Block, Document, Paragraph, Run, RunContent, Table};

/// WCAG 2.x contrast ratio for normal text (AA).
const MIN_CONTRAST_NORMAL: f32 = 4.5;
/// WCAG 2.x contrast ratio for large text (AA): ≥18pt or ≥14pt bold.
const MIN_CONTRAST_LARGE: f32 = 3.0;

/// Checker rule identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilityRule {
    MissingAlt,
    EmptyHeading,
    LowContrast,
}

/// Issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilitySeverity {
    Error,
    Warning,
}

/// One accessibility issue in document order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessibilityIssue {
    pub rule: AccessibilityRule,
    pub severity: AccessibilitySeverity,
    pub message: String,
    /// Primary target: image block, heading paragraph, or containing paragraph.
    pub node_id: NodeId,
    /// Run to place the caret on when jumping (headings / contrast).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<NodeId>,
}

/// Run all F21.S4 accessibility rules against `doc`.
pub fn check_accessibility(doc: &Document) -> Vec<AccessibilityIssue> {
    let mut issues = Vec::new();
    let page_bg = doc
        .sections
        .first()
        .and_then(|s| s.format.page_color)
        .unwrap_or(Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        });
    for section in &doc.sections {
        let section_bg = section.format.page_color.unwrap_or(page_bg);
        check_blocks(doc, &section.blocks, section_bg, None, &mut issues);
    }
    issues
}

fn check_blocks(
    doc: &Document,
    blocks: &[Block],
    page_bg: Color,
    cell_bg: Option<Color>,
    issues: &mut Vec<AccessibilityIssue>,
) {
    for block in blocks {
        match block {
            Block::Paragraph(para) => check_paragraph(doc, para, page_bg, cell_bg, issues),
            Block::ImageBlock(image) => {
                if image.alt_text.as_deref().map(str::trim).unwrap_or("").is_empty() {
                    issues.push(AccessibilityIssue {
                        rule: AccessibilityRule::MissingAlt,
                        severity: AccessibilitySeverity::Error,
                        message: "Picture is missing alternative text".into(),
                        node_id: image.id,
                        run_id: None,
                    });
                }
            }
            Block::Table(table) => check_table(doc, table, page_bg, issues),
            Block::ShapeBlock(_) => {}
        }
    }
}

fn check_table(
    doc: &Document,
    table: &Table,
    page_bg: Color,
    issues: &mut Vec<AccessibilityIssue>,
) {
    for row in &table.rows {
        for cell in &row.cells {
            check_blocks(
                doc,
                &cell.blocks,
                page_bg,
                cell.format.background,
                issues,
            );
        }
    }
}

fn check_paragraph(
    doc: &Document,
    para: &Paragraph,
    page_bg: Color,
    cell_bg: Option<Color>,
    issues: &mut Vec<AccessibilityIssue>,
) {
    if resolved_outline_level(doc, para).is_some() && para.full_text().trim().is_empty() {
        issues.push(AccessibilityIssue {
            rule: AccessibilityRule::EmptyHeading,
            severity: AccessibilitySeverity::Error,
            message: "Heading is empty".into(),
            node_id: para.id,
            run_id: para.runs.first().map(|r| r.id),
        });
    }

    let para_format = doc.styles.resolve_para_format(para.style_id, &para.format);
    let para_bg = para_format.shading.or(cell_bg).unwrap_or(page_bg);

    for run in &para.runs {
        if run.format.hidden == Some(true) {
            continue;
        }
        let text = run_visible_text(run);
        if text.trim().is_empty() {
            continue;
        }
        let resolved = doc.styles.resolve_char_format(para.style_id, &run.format);
        let fg = resolved.color.unwrap_or(Color::BLACK);
        let bg = resolved.highlight.unwrap_or(para_bg);
        let ratio = contrast_ratio(fg, bg);
        let min = min_contrast_for_text(&resolved);
        if ratio + f32::EPSILON < min {
            issues.push(AccessibilityIssue {
                rule: AccessibilityRule::LowContrast,
                severity: AccessibilitySeverity::Warning,
                message: format!(
                    "Text contrast ratio {ratio:.1}:1 is below {min:.1}:1 (WCAG AA)"
                ),
                node_id: para.id,
                run_id: Some(run.id),
            });
        }
    }
}

fn run_visible_text(run: &Run) -> &str {
    match &run.content {
        RunContent::Text(t) | RunContent::Hyperlink { text: t, .. } => t.as_str(),
        _ => "",
    }
}

fn min_contrast_for_text(format: &crate::CharFormat) -> f32 {
    let size = format.font_size.unwrap_or(11.0);
    let bold = format.bold == Some(true);
    if size >= 18.0 || (bold && size >= 14.0) {
        MIN_CONTRAST_LARGE
    } else {
        MIN_CONTRAST_NORMAL
    }
}

/// WCAG 2 relative luminance contrast ratio between two opaque colors.
pub fn contrast_ratio(fg: Color, bg: Color) -> f32 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

fn relative_luminance(c: Color) -> f32 {
    fn channel(v: u8) -> f32 {
        let s = v as f32 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_on_white_passes() {
        let ratio = contrast_ratio(Color::BLACK, Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        });
        assert!(ratio > 20.0, "ratio={ratio}");
    }

    #[test]
    fn light_gray_on_white_fails_aa() {
        let ratio = contrast_ratio(
            Color {
                r: 200,
                g: 200,
                b: 200,
                a: 255,
            },
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        );
        assert!(ratio < MIN_CONTRAST_NORMAL, "ratio={ratio}");
    }
}
