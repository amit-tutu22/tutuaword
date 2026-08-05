use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use thiserror::Error;
use tw_model::{Block, CharFormat, Document, Paragraph, Run};

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
                flush_paragraph(&mut blocks, &mut current_runs, heading_level.take());
                heading_level = Some(level as u8);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_paragraph(&mut blocks, &mut current_runs, heading_level.take());
            }
            Event::Start(Tag::Paragraph) => {
                flush_paragraph(&mut blocks, &mut current_runs, heading_level.take());
            }
            Event::End(TagEnd::Paragraph) => {
                flush_paragraph(&mut blocks, &mut current_runs, heading_level.take());
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
    flush_paragraph(&mut blocks, &mut current_runs, heading_level.take());

    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }
    if let Some(section) = doc.sections.first_mut() {
        section.blocks = blocks;
    }
    Ok(doc)
}

fn flush_paragraph(blocks: &mut Vec<Block>, runs: &mut Vec<Run>, heading_level: Option<u8>) {
    if runs.is_empty() {
        return;
    }
    let mut para = Paragraph::new();
    para.runs = std::mem::take(runs);
    if let Some(level) = heading_level {
        if level == 1 {
            if let Some(id) = Document::new()
                .styles
                .find_style_by_name("Heading 1")
                .map(|s| s.id)
            {
                para.style_id = Some(id);
            }
            for run in &mut para.runs {
                run.format.bold = Some(true);
            }
        }
    }
    blocks.push(Block::Paragraph(para));
}

pub fn export(doc: &Document) -> Result<Vec<u8>, MarkdownError> {
    let mut out = String::new();
    if let Some(section) = doc.sections.first() {
        for block in &section.blocks {
            if let Block::Paragraph(para) = block {
                let is_heading = para.style_id.is_some();
                if is_heading {
                    out.push_str("# ");
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
                if is_heading {
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
        assert!(first.style_id.is_some() || first.runs[0].format.bold == Some(true));
        assert!(doc.sections[0].blocks.len() >= 2);
    }

    #[test]
    fn export_preserves_heading_marker() {
        let doc = import(b"# Heading\n\nBody").unwrap();
        let md = export(&doc).unwrap();
        let text = String::from_utf8(md).unwrap();
        assert!(text.contains('#'));
        assert!(text.contains("Body"));
    }
}
