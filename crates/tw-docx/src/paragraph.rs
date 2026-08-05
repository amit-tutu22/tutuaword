use tw_model::{BreakType, Document, Paragraph, Run, RunContent};

use crate::styles::{parse_char_properties, parse_para_properties};
use crate::xml_util::{extract_plain_text, read_attr_value, split_elements};

pub fn parse_paragraph(doc: &Document, para_xml: &str) -> Option<Paragraph> {
    let mut para = Paragraph::new();

    if para_xml.contains("<w:pPr") {
        let ppr_end = para_xml.find("</w:pPr>").unwrap_or(para_xml.len());
        let ppr = if let Some(start) = para_xml.find("<w:pPr") {
            &para_xml[start..ppr_end]
        } else {
            para_xml
        };
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
    let mut runs = Vec::new();
    for chunk in split_elements(body_xml, "w:r") {
        runs.extend(parse_run(doc, chunk));
    }

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

fn parse_run(doc: &Document, run_xml: &str) -> Vec<Run> {
    let mut format = parse_char_properties(run_xml);

    if let Some(style_id_str) = read_attr_value(run_xml, "w:rStyle", "w:val") {
        if let Some(style) = doc.styles.character_styles.values().find(|s| s.name == style_id_str) {
            format.merge(&style.char_format);
        }
    }

    let mut revision = None;
    if run_xml.contains("<w:ins ") {
        revision = Some(tw_model::Revision::insert("import"));
    } else if run_xml.contains("<w:del ") {
        revision = Some(tw_model::Revision::delete("import"));
    }

    let mut runs = Vec::new();
    if run_xml.contains("<w:br") {
        let break_type = if run_xml.contains("w:type=\"page\"") || run_xml.contains("w:type='page'") {
            BreakType::Page
        } else {
            BreakType::Line
        };
        runs.push(Run {
            id: tw_model::NodeId::new(),
            format: format.clone(),
            content: RunContent::Break(break_type),
            revision: revision.clone(),
        });
    }

    let text = extract_plain_text(run_xml);
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
