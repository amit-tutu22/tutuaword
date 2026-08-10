use tw_model::{
    BookmarkAnchor, BreakType, CommentRef, Document, FieldData, FieldType, FootnoteRef,
    HyperlinkTarget, Paragraph, Run, RunContent,
};

use crate::retention::ImportRetentionReport;
use crate::styles::{parse_char_properties, parse_para_default_char_format, parse_para_properties};
use crate::table::{parse_inline_image, MediaResolver};
use crate::xml_util::{
    extract_plain_text, extract_run_text, next_run_level_tag, read_attr_on_element,
    read_attr_value, read_int_attr, split_elements, take_element, RunLevelTag,
};

/// The `<w:pPr>` slice of a paragraph, or the whole element when it has none.
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
    parse_paragraph_with_retention(doc, para_xml, None, None)
}

pub fn parse_paragraph_with_retention(
    doc: &Document,
    para_xml: &str,
    retention: Option<&mut ImportRetentionReport>,
    media: Option<&dyn MediaResolver>,
) -> Option<Paragraph> {
    let mut para = Paragraph::new();

    if para_xml.contains("<w:pPr") {
        let ppr = paragraph_properties_xml(para_xml);
        para.format = parse_para_properties(ppr);
        let para_defaults = parse_para_default_char_format(ppr);

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

        let body_xml = match para_xml.find("</w:pPr>") {
            Some(end) => &para_xml[end..],
            None => para_xml,
        };
        let mut runs = parse_paragraph_runs(doc, body_xml, 0, retention, media);
        for run in &mut runs {
            let direct = run.format.clone();
            run.format = para_defaults.clone();
            run.format.merge(&direct);
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
        return Some(para);
    }

    let body_xml = match para_xml.find("</w:pPr>") {
        Some(end) => &para_xml[end..],
        None => para_xml,
    };
    let mut runs = parse_paragraph_runs(doc, body_xml, 0, retention, media);

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

fn parse_paragraph_runs(
    doc: &Document,
    body_xml: &str,
    depth: u32,
    mut retention: Option<&mut ImportRetentionReport>,
    media: Option<&dyn MediaResolver>,
) -> Vec<Run> {
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
                let body = element
                    .strip_prefix("<w:r")
                    .and_then(|s| s.find('>').map(|i| &s[i + 1..]))
                    .and_then(|s| s.strip_suffix("</w:r>"))
                    .unwrap_or(element);
                runs.extend(parse_run(doc, body, retention.as_deref_mut(), media));
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
                for run in parse_paragraph_runs(doc, inner, depth + 1, retention.as_deref_mut(), media)
                {
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
                for run in parse_paragraph_runs(doc, inner, depth + 1, retention.as_deref_mut(), media)
                {
                    let mut run = run;
                    run.revision = Some(tw_model::Revision::delete(author.clone()));
                    runs.push(run);
                }
                rest = after;
            }
            RunLevelTag::Hyperlink => {
                if let Some(r) = retention.as_deref_mut() {
                    r.record_encountered("hyperlink");
                }
                let Some((element, after)) = take_element(rest, "w:hyperlink") else {
                    break;
                };
                let inner = element
                    .find('>')
                    .map(|i| &element[i + 1..])
                    .and_then(|s| s.strip_suffix("</w:hyperlink>"))
                    .unwrap_or("");
                let target = parse_hyperlink_target(element);
                let inner_runs = parse_paragraph_runs(doc, inner, depth, retention.as_deref_mut(), media);
                let text = inner_runs
                    .iter()
                    .map(|r| r.text())
                    .collect::<String>();
                if !text.is_empty() || target.url != "#" {
                    if let Some(r) = retention.as_deref_mut() {
                        r.record_retained("hyperlink");
                    }
                    let format = inner_runs
                        .first()
                        .map(|r| r.format.clone())
                        .unwrap_or_default();
                    runs.push(Run {
                        id: tw_model::NodeId::new(),
                        format,
                        content: RunContent::Hyperlink { target, text },
                        revision: None,
                    });
                }
                rest = after;
            }
            RunLevelTag::FieldSimple => {
                if let Some(r) = retention.as_deref_mut() {
                    r.record_encountered("fldSimple");
                }
                let Some((element, after)) = take_element(rest, "w:fldSimple") else {
                    break;
                };
                let instr = read_attr_on_element(element, "w:fldSimple", "w:instr");
                let inner = element
                    .find('>')
                    .map(|i| &element[i + 1..])
                    .and_then(|s| s.strip_suffix("</w:fldSimple>"))
                    .unwrap_or("");
                let display = extract_plain_text(inner);
                if let Some(key) = instr
                    .as_deref()
                    .and_then(crate::bibliography::parse_citation_key)
                {
                    if let Some(r) = retention.as_deref_mut() {
                        r.record_retained("fldSimple");
                    }
                    runs.push(Run {
                        id: tw_model::NodeId::new(),
                        format: tw_model::CharFormat::default(),
                        content: RunContent::CitationRef(tw_model::CitationRef {
                            source_key: key,
                            display_text: if display.is_empty() {
                                None
                            } else {
                                Some(display)
                            },
                        }),
                        revision: None,
                    });
                    rest = after;
                    continue;
                }
                let field_type = instr
                    .as_deref()
                    .map(parse_field_type)
                    .unwrap_or(FieldType::Other("unknown".into()));
                if let Some(r) = retention.as_deref_mut() {
                    r.record_retained("fldSimple");
                }
                runs.push(Run {
                    id: tw_model::NodeId::new(),
                    format: tw_model::CharFormat::default(),
                    content: RunContent::Field(field_data_from_import(field_type, instr, display)),
                    revision: None,
                });
                rest = after;
            }
            RunLevelTag::BookmarkStart => {
                if let Some(r) = retention.as_deref_mut() {
                    r.record_encountered("bookmarkStart");
                }
                let Some((element, after)) = take_element(rest, "w:bookmarkStart") else {
                    break;
                };
                let name = read_attr_on_element(element, "w:bookmarkStart", "w:name")
                    .unwrap_or_else(|| "_".into());
                let bookmark_id = read_int_attr(element, "w:bookmarkStart", "w:id");
                if let Some(r) = retention.as_deref_mut() {
                    r.record_retained("bookmarkStart");
                }
                runs.push(Run {
                    id: tw_model::NodeId::new(),
                    format: tw_model::CharFormat::default(),
                    content: RunContent::Bookmark(BookmarkAnchor {
                        name,
                        bookmark_id,
                    }),
                    revision: None,
                });
                rest = after;
            }
            RunLevelTag::OfficeMath => {
                if let Some(r) = retention.as_deref_mut() {
                    r.record_encountered("oMath");
                }
                let Some((element, after)) = take_element(rest, "m:oMath") else {
                    break;
                };
                if let Some(r) = retention.as_deref_mut() {
                    r.record_retained("oMath");
                }
                runs.push(Run {
                    id: tw_model::NodeId::new(),
                    format: tw_model::CharFormat::default(),
                    content: RunContent::OfficeMath {
                        xml: element.to_string(),
                    },
                    revision: None,
                });
                rest = after;
            }
        }
    }
    runs
}

fn parse_hyperlink_target(element: &str) -> HyperlinkTarget {
    let anchor = read_attr_on_element(element, "w:hyperlink", "w:anchor");
    let url = read_attr_on_element(element, "w:hyperlink", "r:id")
        .map(|id| format!("r:id:{id}"))
        .or_else(|| anchor.clone().map(|a| format!("#{a}")))
        .unwrap_or_else(|| "#".into());
    HyperlinkTarget {
        url,
        anchor,
        tooltip: read_attr_on_element(element, "w:hyperlink", "w:tooltip"),
    }
}

fn parse_field_type(instr: &str) -> FieldType {
    let upper = instr.trim().to_ascii_uppercase();
    // Form fields before PAGE — FORMTEXT does not contain PAGE, but keep order explicit (F26.S1).
    if upper.contains("FORMCHECKBOX") {
        FieldType::FormCheckbox
    } else if upper.contains("FORMTEXT") {
        FieldType::FormText
    } else if upper.contains("MERGEFIELD") {
        FieldType::MergeField
    } else if upper.contains("PAGE") && !upper.contains("NUMPAGES") {
        FieldType::Page
    } else if upper.contains("NUMPAGES") {
        FieldType::NumPages
    } else if upper.contains("DATE") {
        FieldType::Date
    } else if upper.contains("TIME") {
        FieldType::Time
    } else if upper.contains("FILENAME") {
        FieldType::Filename
    } else if upper.contains("AUTHOR") {
        FieldType::Author
    } else if upper.contains("TITLE") {
        FieldType::Title
    } else if upper.contains("REF") || upper.contains("PAGEREF") {
        FieldType::CrossRef
    } else {
        FieldType::Other(instr.trim().to_string())
    }
}

fn field_data_from_import(
    field_type: FieldType,
    instr: Option<String>,
    display: String,
) -> FieldData {
    use tw_model::{
        form_checkbox_checked_from_display, form_checkbox_display, merge_field_placeholder,
        FormFieldMeta,
    };
    let display_text = if display.is_empty() {
        None
    } else {
        Some(display.clone())
    };
    match field_type {
        FieldType::FormText => FieldData {
            field_type,
            instruction: instr.or_else(|| Some(" FORMTEXT ".into())),
            display_text: display_text.clone(),
            form: Some(FormFieldMeta {
                name: None,
                checked: None,
                default_text: display_text,
            }),
            merge_name: None,
        },
        FieldType::FormCheckbox => {
            let checked = form_checkbox_checked_from_display(&display);
            FieldData {
                field_type,
                instruction: instr.or_else(|| Some(" FORMCHECKBOX ".into())),
                display_text: Some(form_checkbox_display(checked)),
                form: Some(FormFieldMeta {
                    name: None,
                    checked: Some(checked),
                    default_text: None,
                }),
                merge_name: None,
            }
        }
        FieldType::MergeField => {
            let name = instr
                .as_deref()
                .and_then(|s| {
                    let upper = s.to_ascii_uppercase();
                    let idx = upper.find("MERGEFIELD")?;
                    let rest = s[idx + "MERGEFIELD".len()..].trim();
                    let n = rest
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_matches('"')
                        .trim();
                    if n.is_empty() {
                        None
                    } else {
                        Some(n.to_string())
                    }
                })
                .unwrap_or_else(|| "Name".into());
            let placeholder = merge_field_placeholder(&name);
            FieldData {
                field_type,
                instruction: instr.or_else(|| Some(format!(" MERGEFIELD {name} "))),
                display_text: display_text.or(Some(placeholder)),
                form: None,
                merge_name: Some(name),
            }
        }
        _ => FieldData {
            field_type,
            instruction: instr,
            display_text,
            form: None,
            merge_name: None,
        },
    }
}

fn parse_run(
    doc: &Document,
    run_xml: &str,
    mut retention: Option<&mut ImportRetentionReport>,
    media: Option<&dyn MediaResolver>,
) -> Vec<Run> {
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

    if run_xml.contains("w:type=\"page\"") || run_xml.contains("w:type='page'") {
        runs.push(Run {
            id: tw_model::NodeId::new(),
            format: format.clone(),
            content: RunContent::Break(BreakType::Page),
            revision: revision.clone(),
        });
    }

    if run_xml.contains("<w:footnoteReference") {
        if let Some(r) = retention.as_deref_mut() {
            r.record_encountered("footnoteReference");
            r.record_retained("footnoteReference");
        }
        let note_id = split_elements(run_xml, "w:footnoteReference")
            .into_iter()
            .next()
            .and_then(|el| read_int_attr(el, "w:footnoteReference", "w:id"))
            .unwrap_or(0);
        runs.push(Run {
            id: tw_model::NodeId::new(),
            format: format.clone(),
            content: RunContent::FootnoteRef(FootnoteRef {
                note_id,
                display_number: None,
            }),
            revision: revision.clone(),
        });
    }

    if run_xml.contains("<w:commentReference") {
        if let Some(r) = retention.as_deref_mut() {
            r.record_encountered("commentReference");
            r.record_retained("commentReference");
        }
        let comment_id = split_elements(run_xml, "w:commentReference")
            .into_iter()
            .next()
            .and_then(|el| read_int_attr(el, "w:commentReference", "w:id"))
            .unwrap_or(0);
        runs.push(Run {
            id: tw_model::NodeId::new(),
            format: format.clone(),
            content: RunContent::CommentRef(CommentRef {
                comment_id,
                display_number: None,
            }),
            revision: revision.clone(),
        });
    }

    if run_xml.contains("<w:drawing") || run_xml.contains("<w:pict") {
        if let Some(r) = retention.as_deref_mut() {
            r.record_encountered("drawing");
        }
        if let Some(resolver) = media {
            if let Some(inline) = parse_inline_image(run_xml, resolver) {
                if let Some(r) = retention.as_deref_mut() {
                    r.record_retained("drawing");
                }
                runs.push(Run {
                    id: tw_model::NodeId::new(),
                    format: format.clone(),
                    content: RunContent::InlineImage(inline),
                    revision: revision.clone(),
                });
            }
        }
    }

    if run_xml.contains("<m:oMath") {
        if let Some(r) = retention.as_deref_mut() {
            r.record_encountered("oMath");
        }
        if let Some((element, _)) = take_element(run_xml, "m:oMath") {
            if let Some(r) = retention.as_deref_mut() {
                r.record_retained("oMath");
            }
            runs.push(Run {
                id: tw_model::NodeId::new(),
                format: format.clone(),
                content: RunContent::OfficeMath {
                    xml: element.to_string(),
                },
                revision: revision.clone(),
            });
            return runs;
        }
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
