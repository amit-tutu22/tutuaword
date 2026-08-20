use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use thiserror::Error;
use tw_model::{
    cell_text_at_grid, cell_visible_text, hyperlink_run, Block, CharFormat, Document,
    NumberingRef, Paragraph, Run, RunContent, StyleSheet, Table, TableCell, TableRow,
};

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
    let mut pending_numbering: Option<NumberingRef> = None;
    let mut list_stack: Vec<u32> = Vec::new();
    let mut link_dest: Option<String> = None;
    let mut link_text = String::new();
    let mut table: Option<TableBuilder> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
                heading_level = Some(level as u8);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
            }
            Event::Start(Tag::Paragraph) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
            }
            Event::End(TagEnd::Paragraph) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
            }
            Event::Start(Tag::List(start)) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
                let numbering_id = if start.is_some() { 2 } else { 1 };
                list_stack.push(numbering_id);
            }
            Event::End(TagEnd::List(_)) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
                list_stack.pop();
            }
            Event::Start(Tag::Item) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
                if let Some(&numbering_id) = list_stack.last() {
                    let level = list_stack.len().saturating_sub(1) as u32;
                    pending_numbering = Some(NumberingRef {
                        numbering_id,
                        level,
                    });
                }
            }
            Event::End(TagEnd::Item) => {}
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_dest = Some(dest_url.to_string());
                link_text.clear();
            }
            Event::End(TagEnd::Link) => {
                if let Some(url) = link_dest.take() {
                    let text = if link_text.is_empty() {
                        url.clone()
                    } else {
                        std::mem::take(&mut link_text)
                    };
                    let mut run = hyperlink_run(url, text, None);
                    run.format.merge(&current_format);
                    current_runs.push(run);
                }
            }
            Event::Start(Tag::CodeBlock(_)) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
                current_format.font_family = Some("Courier New".into());
            }
            Event::End(TagEnd::CodeBlock) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
                current_format.font_family = None;
            }
            Event::Start(Tag::Table(_)) => {
                flush_paragraph(
                    &doc.styles,
                    &mut blocks,
                    &mut current_runs,
                    heading_level.take(),
                    &mut pending_numbering,
                );
                table = Some(TableBuilder::new());
            }
            Event::End(TagEnd::Table) => {
                if let Some(builder) = table.as_mut() {
                    builder.end_row();
                }
                if let Some(builder) = table.take() {
                    if let Some(table_block) = builder.finish() {
                        blocks.push(Block::Table(table_block));
                    }
                }
            }
            Event::Start(Tag::TableHead) => {
                if let Some(builder) = table.as_mut() {
                    builder.start_row();
                }
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(builder) = table.as_mut() {
                    builder.end_row();
                }
            }
            Event::Start(Tag::TableRow) => {
                if let Some(builder) = table.as_mut() {
                    builder.start_row();
                }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(builder) = table.as_mut() {
                    builder.end_row();
                }
            }
            Event::Start(Tag::TableCell) => {
                if let Some(builder) = table.as_mut() {
                    builder.start_cell();
                }
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(builder) = table.as_mut() {
                    builder.end_cell(&doc.styles);
                }
            }
            Event::Start(Tag::Strong) => current_format.bold = Some(true),
            Event::End(TagEnd::Strong) => current_format.bold = None,
            Event::Start(Tag::Emphasis) => current_format.italic = Some(true),
            Event::End(TagEnd::Emphasis) => current_format.italic = None,
            Event::Text(text) => {
                if let Some(builder) = table.as_mut() {
                    if builder.in_cell {
                        builder.push_cell_text(&text, &current_format);
                    } else if link_dest.is_some() {
                        link_text.push_str(&text);
                    } else {
                        push_merged_text_run(&mut current_runs, &text, &current_format);
                    }
                } else if link_dest.is_some() {
                    link_text.push_str(&text);
                } else {
                    push_merged_text_run(&mut current_runs, &text, &current_format);
                }
            }
            Event::Code(text) => {
                let mut run = Run::new_text(text.to_string());
                run.format = CharFormat {
                    font_family: Some("Courier New".into()),
                    ..current_format.clone()
                };
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
        &mut pending_numbering,
    );

    if blocks.is_empty() {
        blocks.push(Block::Paragraph(Paragraph::new()));
    }
    if let Some(section) = doc.sections.first_mut() {
        section.blocks = blocks;
    }
    Ok(doc)
}

struct TableBuilder {
    rows: Vec<TableRow>,
    current_row: Vec<TableCell>,
    cell_runs: Vec<Run>,
    in_cell: bool,
}

impl TableBuilder {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            current_row: Vec::new(),
            cell_runs: Vec::new(),
            in_cell: false,
        }
    }

    fn start_row(&mut self) {
        self.current_row.clear();
    }

    fn end_row(&mut self) {
        if !self.current_row.is_empty() {
            self.rows.push(TableRow::with_cells(std::mem::take(
                &mut self.current_row,
            )));
        }
    }

    fn start_cell(&mut self) {
        self.cell_runs.clear();
        self.in_cell = true;
    }

    fn push_cell_text(&mut self, text: &str, format: &CharFormat) {
        if !self.in_cell {
            return;
        }
        push_merged_text_run(&mut self.cell_runs, text, format);
    }

    fn end_cell(&mut self, styles: &StyleSheet) {
        self.in_cell = false;
        let mut para = Paragraph::new();
        if self.cell_runs.is_empty() {
            para.runs = vec![Run::new_text("")];
        } else {
            para.runs = std::mem::take(&mut self.cell_runs);
        }
        let _ = styles;
        let mut cell = TableCell::new();
        cell.blocks = vec![Block::Paragraph(para)];
        self.current_row.push(cell);
    }

    fn finish(self) -> Option<Table> {
        if self.rows.is_empty() {
            return None;
        }
        let cols = self
            .rows
            .iter()
            .map(|r| r.cells.len())
            .max()
            .unwrap_or(1) as u32;
        let mut table = Table::new(self.rows.len() as u32, cols);
        for (row_idx, row) in self.rows.into_iter().enumerate() {
            if let Some(dest) = table.rows.get_mut(row_idx) {
                dest.cells = row.cells;
            }
        }
        Some(table)
    }
}

fn push_merged_text_run(runs: &mut Vec<Run>, text: &str, format: &CharFormat) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = runs.last_mut() {
        if last.format == *format
            && last.revision.is_none()
            && matches!(last.content, RunContent::Text(_))
        {
            if let Some(existing) = last.text_mut() {
                existing.push_str(text);
                return;
            }
        }
    }
    let mut run = Run::new_text(text.to_string());
    run.format = format.clone();
    runs.push(run);
}

fn flush_paragraph(
    styles: &StyleSheet,
    blocks: &mut Vec<Block>,
    runs: &mut Vec<Run>,
    heading_level: Option<u8>,
    numbering: &mut Option<NumberingRef>,
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
    if let Some(numbering) = numbering.take() {
        para.format.numbering = Some(numbering);
    }
    blocks.push(Block::Paragraph(para));
}

fn markdown_heading_prefix(
    styles: &StyleSheet,
    style_id: Option<tw_model::StyleId>,
) -> Option<&'static str> {
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
            match block {
                Block::Paragraph(para) => {
                    let heading = markdown_heading_prefix(&doc.styles, para.style_id);
                    if let Some(prefix) = heading {
                        out.push_str(prefix);
                    }
                    for run in &para.runs {
                        match &run.content {
                            RunContent::Hyperlink { target, text } => {
                                out.push_str(&format!("[{}]({})", text, target.url));
                            }
                            _ => {
                                let mut text = run.text().to_string();
                                if run.format.bold == Some(true) {
                                    text = format!("**{text}**");
                                }
                                if run.format.italic == Some(true) {
                                    text = format!("*{text}*");
                                }
                                out.push_str(&text);
                            }
                        }
                    }
                    out.push('\n');
                    if heading.is_some() {
                        out.push('\n');
                    }
                }
                Block::Table(table) => {
                    for (row_idx, row) in table.rows.iter().enumerate() {
                        out.push('|');
                        for cell in &row.cells {
                            let text = cell_visible_text(cell);
                            out.push(' ');
                            out.push_str(&text);
                            out.push_str(" |");
                        }
                        out.push('\n');
                        if row_idx == 0 {
                            let cols = row.cells.len().max(1);
                            out.push('|');
                            for _ in 0..cols {
                                out.push_str(" --- |");
                            }
                            out.push('\n');
                        }
                    }
                    out.push('\n');
                }
                _ => {}
            }
        }
    }
    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::RunContent;

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

    #[test]
    fn u_f23_s5_md_link_imports_hyperlink_run() {
        let doc = import(b"Visit [Example](https://example.com) now.").unwrap();
        let para = doc.sections[0].blocks[0].paragraph().unwrap();
        let link = para.runs.iter().find_map(|r| match &r.content {
            RunContent::Hyperlink { target, text } => Some((target.url.as_str(), text.as_str())),
            _ => None,
        });
        assert_eq!(
            link,
            Some(("https://example.com", "Example")),
            "runs: {:?}",
            para.runs
        );
    }

    #[test]
    fn u_f23_s5_md_bullet_list_gets_numbering() {
        let doc = import(b"- One\n- Two").unwrap();
        let blocks: Vec<_> = doc.sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph())
            .collect();
        assert!(blocks.len() >= 2);
        assert_eq!(
            blocks[0].format.numbering.map(|n| n.numbering_id),
            Some(1)
        );
    }

    #[test]
    fn u_f23_s5_md_loose_list_keeps_numbering() {
        let doc = import(b"- One\n\n- Two").unwrap();
        let blocks: Vec<_> = doc.sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph())
            .collect();
        assert!(blocks.len() >= 2, "blocks={}", blocks.len());
        assert_eq!(
            blocks[0].format.numbering.map(|n| n.numbering_id),
            Some(1),
            "loose list item lost numbering"
        );
        assert_eq!(
            blocks[1].format.numbering.map(|n| n.numbering_id),
            Some(1)
        );
    }

    #[test]
    fn u_f23_s5_md_table_imports_block() {
        let doc = import(b"| A | B |\n| --- | --- |\n| 1 | 2 |").unwrap();
        let table = doc.sections[0].blocks.iter().find_map(|b| b.table());
        assert!(table.is_some(), "expected table block");
        let table = table.unwrap();
        assert_eq!(table.rows.len(), 2);
        assert!(cell_text_at_grid(table, 0, 0).unwrap().contains('A'));
        assert!(cell_text_at_grid(table, 1, 0).unwrap().contains('1'));
    }
}
