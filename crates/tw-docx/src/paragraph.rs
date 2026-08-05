use tw_model::{BreakType, Document, Paragraph, Run, RunContent};

use crate::styles::{parse_char_properties, parse_para_properties};
use crate::xml_util::{extract_plain_text, extract_run_text, next_run_level_tag, read_attr_on_element, read_attr_value, take_element, RunLevelTag};

/// The `<w:pPr>` slice of a paragraph, or the whole element when it has none.
/// Scoping matters: read against the full paragraph, `w:spacing` and `w:u`
/// from a run's `w:rPr` would be taken for paragraph properties.
pub fn paragraph_properties_xml(para_xml: &str) -> &str {
    let Some(start) = para_xml.find("<w:pPr") else {
        return para_xml;
    };
    let end = para_xml.find("</w:pPr>").unwrap_or(para_xml.len());
    if end > start {
        &para_xml[start..end]
    } else {
        para_xml
    }
}

pub fn parse_paragraph(doc: &Document, para_xml: &str) -> Option<Paragraph> {
    let mut para = Paragraph::new();

    if para_xml.contains("<w:pPr") {
        let ppr = paragraph_properties_xml(para_xml);
        para.format = parse_para_properties(ppr);

        if let Some(style_id_str) = read_attr_value(ppr, "w:pStyle", "w:val") {
            if let Some(style) = doc.styles.find_style_by_ooxml_id(&style_id_str) {
                para.style_id = Some(style.id);
            } else if let Some(style) = doc.styles.find_style_by_name(&style_id_str) {
                para.style_id = Some(style.id);
            } else {
                let normalized = style_id_str.to_ascii_lowercase();
                if normalized.contains("heading1") || normalized == "heading1" {
                    if let Some(id) = doc.styles.find_style_by_name("Heading 1").map(|s| s.id) {
                        para.style_id = Some(id);
                    }
                }
            }
        }
    }

    // Run properties live in `<w:pPr>` too; only body runs become content.
    let body_xml = match para_xml.find("</w:pPr>") {
        Some(end) => &para_xml[end..],
        None => para_xml,
    };
    let mut runs = parse_paragraph_runs(doc, body_xml, 0);

    if runs.is_empty() {
        let text = extract_plain_text(para_xml);
        if !text.is_empty() {
            runs.push(Run::new_text(text));
        }
    }

    let has_spacing = para.format.space_before.is_some() || para.format.space_after.is_some();
    if runs.is_empty() && !has_spacing {
        return None;
    }

    if runs.is_empty() {
        runs.push(Run::new_text(String::new()));
    }

    para.runs = runs;
    Some(para)
}

pub fn parse_paragraph_xml(para_xml: &str) -> Option<Paragraph> {
    parse_paragraph(&Document::new(), para_xml)
}

const MAX_TRACK_CHANGE_DEPTH: u32 = 64;

fn parse_paragraph_runs(doc: &Document, body_xml: &str, depth: u32) -> Vec<Run> {
    if depth > MAX_TRACK_CHANGE_DEPTH {
        return Vec::new();
    }
    let mut runs = Vec::new();
    let mut rest = body_xml;
    while let Some((offset, tag)) = next_run_level_tag(rest) {
        rest = &rest[offset..];
        match tag {
            RunLevelTag::Run => {
                let Some((element, after)) = take_element(rest, "w:r") else {
                    break;
                };
                // Body slice between `<w:r` and `>` … `</w:r>`.
                let body = element
                    .strip_prefix("<w:r")
                    .and_then(|s| s.find('>').map(|i| &s[i + 1..]))
                    .and_then(|s| s.strip_suffix("</w:r>"))
                    .unwrap_or(element);
                runs.extend(parse_run(doc, body));
                rest = after;
            }
            RunLevelTag::Insert => {
                let Some((element, after)) = take_element(rest, "w:ins") else {
                    break;
                };
                let author = read_attr_on_element(element, "w:ins", "w:author")
                    .unwrap_or_else(|| "import".into());
                let inner = element
                    .find('>')
                    .map(|i| &element[i + 1..])
                    .and_then(|s| s.strip_suffix("</w:ins>"))
                    .unwrap_or("");
                for run in parse_paragraph_runs(doc, inner, depth + 1) {
                    let mut run = run;
                    run.revision = Some(tw_model::Revision::insert(author.clone()));
                    runs.push(run);
                }
                rest = after;
            }
            RunLevelTag::Delete => {
                let Some((element, after)) = take_element(rest, "w:del") else {
                    break;
                };
                let author = read_attr_on_element(element, "w:del", "w:author")
                    .unwrap_or_else(|| "import".into());
                let inner = element
                    .find('>')
                    .map(|i| &element[i + 1..])
                    .and_then(|s| s.strip_suffix("</w:del>"))
                    .unwrap_or("");
                for run in parse_paragraph_runs(doc, inner, depth + 1) {
                    let mut run = run;
                    run.revision = Some(tw_model::Revision::delete(author.clone()));
                    runs.push(run);
                }
                rest = after;
            }
        }
    }
    runs
}

fn parse_run(doc: &Document, run_xml: &str) -> Vec<Run> {
    let mut format = parse_char_properties(run_xml);

    if let Some(style_id_str) = read_attr_value(run_xml, "w:rStyle", "w:val") {
        if let Some(style) = doc.styles.character_styles.values().find(|s| s.name == style_id_str) {
            format.merge(&style.char_format);
        }
    }

    let mut revision = None;
    // Legacy import path: revision markers inside a run (older export builds).
    if run_xml.contains("<w:ins ") {
        revision = Some(tw_model::Revision::insert("import"));
    } else if run_xml.contains("<w:del ") {
        revision = Some(tw_model::Revision::delete("import"));
    }

    let mut runs = Vec::new();
    // Only a page break needs its own run: layout has to split the page on it.
    // A line break is just a newline in the run's text.
    if run_xml.contains("w:type=\"page\"") || run_xml.contains("w:type='page'") {
        runs.push(Run {
            id: tw_model::NodeId::new(),
            format: format.clone(),
            content: RunContent::Break(BreakType::Page),
            revision: revision.clone(),
        });
    }

    let text = extract_run_text(run_xml);
    if !text.is_empty() {
        runs.push(Run {
            id: tw_model::NodeId::new(),
            format,
            content: RunContent::Text(text),
            revision,
        });
    }

    runs
}
