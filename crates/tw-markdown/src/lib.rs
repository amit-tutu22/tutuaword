use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use thiserror::Error;
use tw_model::{Block, CharFormat, Document, Paragraph, Run, StyleSheet};

#[derive(Debug, Error)]
pub enum MarkdownError {
    #[error("markdown file is not valid utf-8")]
    InvalidUtf8,
    #[error("markdown export failed")]
    ExportFailed,
}

pub fn import(source: &[u8]) -> Result<Document, MarkdownError> {
    let text = std::str::from_utf8(source).map_err(|_| MarkdownError::InvalidUtf8)?;
    let parser = Parser::new_ext(text, Options::all());
    let mut doc = Document::new();
    let mut blocks = Vec::new();
    let mut current_runs: Vec<Run> = Vec::new();
    let mut current_format = CharFormat::default();
    let mut heading_level: Option<u8> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_paragraph(&doc.styles, &mut blocks, &mut current_runs, heading_level.take());
                heading_level = Some(level as u8);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_paragraph(&doc.styles, &mut blocks, &mut current_runs, heading_level.take());
            }
            Event::Start(Tag::Paragraph) => {
                flush_paragraph(&doc.styles, &mut blocks, &mut current_runs, heading_level.take());
            }
            Event::End(TagEnd::Paragraph) => {
                flush_paragraph(&doc.styles, &mut blocks, &mut current_runs, heading_level.take());
            }
            Event::Start(Tag::Strong) => current_format.bold = Some(true),
            Event::End(TagEnd::Strong) => current_format.bold = None,
            Event::Start(Tag::Emphasis) => current_format.italic = Some(true),
            Event::End(TagEnd::Emphasis) => current_format.italic = None,
            Event::Start(Tag::List(_)) | Event::End(TagEnd::List(_)) => {}
            Event::Start(Tag::Item) | Event::End(TagEnd::Item) => {}
            Event::Text(text) => {
                let mut run = Run::new_text(text.to_string());
                run.format = current_format.clone();
                current_runs.push(run);
            }
            Event::SoftBreak | Event::HardBreak => {
                current_runs.push(Run::new_text("\n"));
            }
            _ => {}
        }
    }
    flush_paragraph(
        &doc.styles,
        &mut blocks,
        &mut current_runs,
        heading_level.take(),
    );

    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }
    if let Some(section) = doc.sections.first_mut() {
        section.blocks = blocks;
    }
    Ok(doc)
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
        if level == 1 {
            for run in &mut para.runs {
                run.format.bold = Some(true);
            }
        }
    }
    blocks.push(Block::Paragraph(para));
}

fn markdown_heading_prefix(styles: &StyleSheet, style_id: Option<tw_model::StyleId>) -> Option<&'static str> {
    let id = style_id?;
    let style = styles.paragraph_styles.get(&id)?;
    match style.name.as_str() {
        "Heading 1" => Some("# "),
        "Heading 2" => Some("## "),
        "Heading 3" => Some("### "),
        "Heading 4" => Some("#### "),
        "Heading 5" => Some("##### "),
        "Heading 6" => Some("###### "),
        _ => None,
    }
}

pub fn export(doc: &Document) -> Result<Vec<u8>, MarkdownError> {
    let mut out = String::new();
    if let Some(section) = doc.sections.first() {
        for block in &section.blocks {
            if let Block::Paragraph(para) = block {
                let heading = markdown_heading_prefix(&doc.styles, para.style_id);
                if let Some(prefix) = heading {
                    out.push_str(prefix);
                }
                for run in &para.runs {
                    let mut text = run.text().to_string();
                    if run.format.bold == Some(true) {
                        text = format!("**{text}**");
                    }
                    if run.format.italic == Some(true) {
                        text = format!("*{text}*");
                    }
                    out.push_str(&text);
                }
                out.push('\n');
                if heading.is_some() {
                    out.push('\n');
                }
            }
        }
    }
    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_heading_and_bold() {
        let doc = import(b"# Title\n\nHello **world**").unwrap();
        let first = doc.sections[0].blocks[0].paragraph().unwrap();
        let h1 = doc.styles.find_style_by_name("Heading 1").unwrap().id;
        assert_eq!(first.style_id, Some(h1));
        assert!(doc.sections[0].blocks.len() >= 2);
    }

    #[test]
    fn export_preserves_heading_marker() {
        let doc = import(b"# Heading\n\nBody").unwrap();
        let md = export(&doc).unwrap();
        let text = String::from_utf8(md).unwrap();
        assert!(text.starts_with("# "));
        assert!(text.contains("Body"));
        assert!(!text.contains("# Body"));
    }

    #[test]
    fn u_f23_s3_md_heading_not_every_style() {
        let mut doc = Document::new();
        let quote = doc.styles.find_style_by_name("Quote").unwrap().id;
        let mut para = Paragraph::with_text("Quoted");
        para.style_id = Some(quote);
        doc.sections[0].blocks = vec![Block::Paragraph(para)];
        let text = String::from_utf8(export(&doc).unwrap()).unwrap();
        assert!(!text.starts_with('#'), "Quote must not export as ATX heading");
        assert!(text.contains("Quoted"));
    }
}
