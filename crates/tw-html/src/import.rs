use tw_model::{Block, CharFormat, Document, Paragraph, Run, StyleSheet};

use crate::HtmlError;

pub fn import_html(source: &[u8]) -> Result<Document, HtmlError> {
    let text = String::from_utf8_lossy(source);
    let trimmed = text.trim_start();
    if !(trimmed.starts_with('<') || trimmed.to_ascii_lowercase().contains("<html")) {
        return Err(HtmlError::InvalidFormat);
    }
    Ok(parse_html(&sanitize_html(&text)))
}

/// Strip script/style blocks before parsing clipboard HTML.
pub fn sanitize_html(html: &str) -> String {
    let mut out = strip_tag_blocks(html, "script");
    out = strip_tag_blocks(&out, "style");
    out
}

fn strip_tag_blocks(html: &str, tag: &str) -> String {
    let mut result = html.to_string();
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    loop {
        let lower = result.to_ascii_lowercase();
        let Some(start) = lower.find(&open) else {
            break;
        };
        let Some(rel) = lower[start..].find(&close) else {
            break;
        };
        let end = start + rel + close.len();
        result.replace_range(start..end, "");
    }
    result
}

fn parse_html(html: &str) -> Document {
    let mut doc = Document::new();
    let mut blocks = Vec::new();
    let mut current_runs: Vec<Run> = Vec::new();
    let mut current_format = CharFormat::default();
    let mut heading_level: Option<u8> = None;

    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut tag = String::new();
            while let Some(next) = chars.next() {
                if next == '>' {
                    break;
                }
                tag.push(next);
            }
            let tag = tag.trim().to_ascii_lowercase();
            let name = tag_name(&tag);

            if let Some(level) = heading_level_for_name(name) {
                if tag.starts_with('/') {
                    let apply = heading_level.take().or(Some(level));
                    flush_paragraph(&doc.styles, &mut blocks, &mut current_runs, apply);
                } else {
                    flush_paragraph(&doc.styles, &mut blocks, &mut current_runs, None);
                    heading_level = Some(level);
                }
            } else if matches!(name, "p" | "div") {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                );
            } else if matches!(name, "b" | "strong") {
                current_format.bold = if tag.starts_with('/') {
                    None
                } else {
                    Some(true)
                };
            } else if matches!(name, "i" | "em") {
                current_format.italic = if tag.starts_with('/') {
                    None
                } else {
                    Some(true)
                };
            } else if name == "br" && !tag.starts_with('/') {
                current_runs.push(Run::new_text("\n"));
            }
        } else if !ch.is_whitespace() || !current_runs.is_empty() {
            let mut run = Run::new_text(ch.to_string());
            run.format = current_format.clone();
            if let Some(last) = current_runs.last_mut() {
                if last.format == run.format && last.revision.is_none() {
                    if let Some(text) = last.text_mut() {
                        text.push(ch);
                        continue;
                    }
                }
            }
            current_runs.push(run);
        }
    }
    flush_paragraph(
        &doc.styles,
        &mut blocks,
        &mut current_runs,
        heading_level.take(),
    );

    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::with_text(strip_html(html))));
    }
    if let Some(section) = doc.sections.first_mut() {
        section.blocks = blocks;
    }
    doc
}

/// Local name of an HTML tag (`"/strong class=x"` → `"strong"`).
fn tag_name(tag: &str) -> &str {
    let t = tag.trim().trim_start_matches('/').trim_end_matches('/');
    t.split(|c: char| c.is_whitespace()).next().unwrap_or(t)
}

fn heading_level_for_name(name: &str) -> Option<u8> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

fn flush_paragraph(
    styles: &StyleSheet,
    blocks: &mut Vec<Block>,
    runs: &mut Vec<Run>,
    heading_level: Option<u8>,
) {
    if runs.is_empty() {
        return;
    }
    let mut para = Paragraph::new();
    para.runs = std::mem::take(runs);
    if let Some(level) = heading_level {
        let name = format!("Heading {level}");
        if let Some(id) = styles.find_style_by_name(&name).map(|s| s.id) {
            para.style_id = Some(id);
        }
    }
    blocks.push(Block::Paragraph(para));
}

fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(ch);
        }
    }
    decode_html_entities(out.trim()).replace("\n\n\n", "\n\n")
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_heading_and_bold() {
        let html = "<html><body><h1>Title</h1><p>Hello <b>World</b></p></body></html>";
        let doc = import_html(html.as_bytes()).unwrap();
        assert!(doc.sections[0].blocks.len() >= 2);
        let heading = doc.sections[0].blocks[0].paragraph().unwrap();
        let h1 = doc.styles.find_style_by_name("Heading 1").unwrap().id;
        assert_eq!(heading.style_id, Some(h1));
        assert_eq!(heading.full_text(), "Title");
        assert_ne!(heading.runs[0].format.bold, Some(true));
        let body = doc.sections[0].blocks.last().unwrap().paragraph().unwrap();
        assert!(body.runs.iter().any(|r| r.format.bold == Some(true)));
    }
}
