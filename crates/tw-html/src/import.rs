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
            } else if matches!(name, "p" | "div" | "li" | "tr")
                || (name == "br" && !tag.starts_with('/'))
            {
                // Block boundaries (and <br>) become real paragraphs — never
                // store a literal `\n` run that would paint as tofu.
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                );
            } else if matches!(name, "td" | "th") && !tag.starts_with('/') {
                if !current_runs.is_empty() {
                    // Separate table cells with a space when they share a line.
                    push_decoded_text(&mut current_runs, &current_format, " ");
                }
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
            }
        } else if ch == '&' {
            let entity = read_html_entity(&mut chars);
            let decoded = decode_entity(&entity);
            push_decoded_text(&mut current_runs, &current_format, &decoded);
        } else if !ch.is_whitespace() || !current_runs.is_empty() {
            let mapped = tw_edit_map_char(ch);
            if let Some(mapped) = mapped {
                push_decoded_text(&mut current_runs, &current_format, &mapped.to_string());
            }
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

fn push_decoded_text(runs: &mut Vec<Run>, format: &CharFormat, text: &str) {
    if text.is_empty() {
        return;
    }
    let mut run = Run::new_text(text.to_string());
    run.format = format.clone();
    if let Some(last) = runs.last_mut() {
        if last.format == run.format && last.revision.is_none() {
            if let Some(existing) = last.text_mut() {
                existing.push_str(text);
                return;
            }
        }
    }
    runs.push(run);
}

fn tw_edit_map_char(ch: char) -> Option<char> {
    // Keep a local subset so tw-html does not depend on tw-edit.
    match ch {
        '\n' | '\r' => None,
        '\t' => Some('\t'),
        '\u{00A0}' => Some(' '),
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{00AD}' => None,
        '\u{F0E0}' | '\u{F0E1}' | '\u{F0E2}' | '\u{F0E3}' | '\u{F0E4}' | '\u{F0E5}'
        | '\u{F0E6}' | '\u{F0E7}' | '\u{F0E8}' | '\u{F0E9}' | '\u{F0EA}' | '\u{F0EB}'
        | '\u{F0EC}' | '\u{F0ED}' | '\u{F0EE}' | '\u{F0EF}' => Some('→'),
        '\u{F0B6}' | '\u{F0B7}' | '\u{F0A7}' | '\u{F0A8}' | '\u{F035}' => Some('•'),
        c if ('\u{F000}'..='\u{F8FF}').contains(&c) => Some('·'),
        c if c.is_control() => None,
        c => Some(c),
    }
}

fn read_html_entity(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut entity = String::from("&");
    while let Some(&next) = chars.peek() {
        entity.push(next);
        chars.next();
        if next == ';' || entity.len() > 16 {
            break;
        }
        if next != '#' && !next.is_ascii_alphanumeric() && next != 'x' && next != 'X' {
            break;
        }
    }
    entity
}

fn decode_entity(entity: &str) -> String {
    match entity {
        "&nbsp;" => " ".into(),
        "&lt;" => "<".into(),
        "&gt;" => ">".into(),
        "&amp;" => "&".into(),
        "&quot;" => "\"".into(),
        "&#39;" | "&apos;" => "'".into(),
        "&rarr;" | "&rightarrow;" => "→".into(),
        "&larr;" | "&leftarrow;" => "←".into(),
        "&bull;" | "&middot;" => "•".into(),
        "&mdash;" => "—".into(),
        "&ndash;" => "–".into(),
        "&hellip;" => "…".into(),
        other if other.starts_with("&#x") || other.starts_with("&#X") => {
            let hex = other
                .trim_start_matches("&#x")
                .trim_start_matches("&#X")
                .trim_end_matches(';');
            u32::from_str_radix(hex, 16)
                .ok()
                .and_then(char::from_u32)
                .map(|c| tw_edit_map_char(c).unwrap_or(c).to_string())
                .unwrap_or_else(|| other.to_string())
        }
        other if other.starts_with("&#") => {
            let num = other.trim_start_matches("&#").trim_end_matches(';');
            num.parse::<u32>()
                .ok()
                .and_then(char::from_u32)
                .map(|c| tw_edit_map_char(c).unwrap_or(c).to_string())
                .unwrap_or_else(|| other.to_string())
        }
        other => other.to_string(),
    }
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
