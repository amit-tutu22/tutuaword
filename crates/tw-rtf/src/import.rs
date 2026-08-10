use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

use tw_model::{Block, Document, NumberingRef, Paragraph, Run, Table, TableCell};

use crate::RtfError;

pub fn import_rtf(source: &[u8]) -> Result<Document, RtfError> {
    let text = String::from_utf8_lossy(source);
    let trimmed = text.trim_start();
    if !trimmed.starts_with("{\\rtf") {
        return Err(RtfError::InvalidFormat);
    }
    Ok(parse_rtf(trimmed))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
    Bullet,
    Numbered,
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    blocks: Vec<Block>,
    text: String,
    /// Paragraph list kind from `\pn*` / `\ls` mapping.
    para_list: Option<ListKind>,
    para_level: u32,
    /// Pending kind from the last `{\*\pn...}` destination.
    pending_pn: Option<ListKind>,
    /// `\lsN` → bullet / numbered from a light `\listtable` scan.
    ls_kinds: HashMap<i32, ListKind>,
    current_ls: Option<i32>,
    /// Accumulating table rows (each row = cell strings).
    table_rows: Option<Vec<Vec<String>>>,
    row_cells: Vec<String>,
    in_table: bool,
}

fn parse_rtf(rtf: &str) -> Document {
    let mut p = Parser {
        chars: rtf.chars().peekable(),
        blocks: Vec::new(),
        text: String::new(),
        para_list: None,
        para_level: 0,
        pending_pn: None,
        ls_kinds: HashMap::new(),
        current_ls: None,
        table_rows: None,
        row_cells: Vec::new(),
        in_table: false,
    };
    p.run();
    p.flush_paragraph();
    p.flush_table();

    let mut doc = Document::new();
    if p.blocks.is_empty() {
        p.blocks.push(Block::Paragraph(Paragraph::new()));
    }
    if let Some(section) = doc.sections.first_mut() {
        section.blocks = p.blocks;
    }
    // Ensure default bullet/numbered catalogs exist for imported refs.
    if doc.settings.numbering.definitions.is_empty() {
        doc.settings.numbering = tw_model::NumberingCatalog::with_defaults();
    }
    doc
}

impl Parser<'_> {
    fn run(&mut self) {
        while let Some(ch) = self.chars.next() {
            match ch {
                '\\' => self.handle_backslash(),
                '{' => self.handle_group_start(),
                '}' => {}
                c if !c.is_control() => self.text.push(c),
                _ => {}
            }
        }
    }

    fn handle_backslash(&mut self) {
        match self.chars.next() {
            Some('\'') => {
                let hex: String = self.chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    // Latin-1 / CP1252-ish single-byte hex escapes.
                    self.text.push(byte as char);
                }
            }
            Some('*') => self.handle_star_destination(),
            Some('\n' | '\r') => {}
            Some(c) if c.is_ascii_alphabetic() => {
                let (word, param) = read_control_word(&mut self.chars, c);
                self.apply_control_word(&word, param);
            }
            Some(c) => apply_control_symbol(c, &mut self.text),
            None => {}
        }
    }

    fn handle_group_start(&mut self) {
        // `{\pntext ...}` is the visible list marker — skip so body text stays clean.
        if self.peek_control_word() == Some("pntext".into()) {
            self.skip_group_from_here();
        }
        // Otherwise ignore the brace; content is processed at top level.
    }

    fn handle_star_destination(&mut self) {
        // Optional destination: `{\*\pn...}` or `{\*\listtable...}` etc.
        // We are positioned after `\*`; a control word usually follows.
        if self.chars.peek() == Some(&'\\') {
            self.chars.next();
            if let Some(c) = self.chars.next() {
                if c.is_ascii_alphabetic() {
                    let (word, param) = read_control_word(&mut self.chars, c);
                    match word.as_str() {
                        "pn" => {
                            let kind = self.scan_pn_destination();
                            if kind.is_some() {
                                self.pending_pn = kind;
                            }
                            return;
                        }
                        "listtable" => {
                            self.scan_listtable_destination();
                            return;
                        }
                        "listoverridetable" => {
                            self.skip_balanced_from_depth(0);
                            return;
                        }
                        _ => {
                            // Unknown `{\*\word...}` — skip rest of destination group.
                            let _ = param;
                            self.skip_balanced_from_depth(0);
                            return;
                        }
                    }
                }
            }
        }
        self.skip_balanced_from_depth(0);
    }

    /// Scan until the closing `}` of the current destination, recording list kind.
    fn scan_pn_destination(&mut self) -> Option<ListKind> {
        let mut kind = None;
        let mut depth = 0;
        while let Some(ch) = self.chars.next() {
            match ch {
                '{' => depth += 1,
                '}' if depth == 0 => break,
                '}' => depth -= 1,
                '\\' => match self.chars.next() {
                    Some(c) if c.is_ascii_alphabetic() => {
                        let (word, _) = read_control_word(&mut self.chars, c);
                        match word.as_str() {
                            "pnlvlblt" => kind = Some(ListKind::Bullet),
                            "pnlvlbody" | "pndec" => kind = Some(ListKind::Numbered),
                            _ => {}
                        }
                    }
                    Some('\'') => {
                        let _hex: String = self.chars.by_ref().take(2).collect();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        kind
    }

    /// Light pass over `\listtable` for `\listidN` + `\levelnfc` (0=decimal, 23=bullet).
    fn scan_listtable_destination(&mut self) {
        let mut depth = 0;
        let mut current_id: Option<i32> = None;
        let mut current_kind: Option<ListKind> = None;
        while let Some(ch) = self.chars.next() {
            match ch {
                '{' => depth += 1,
                '}' if depth == 0 => break,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        // end of a top-level list entry inside listtable
                        if let (Some(id), Some(kind)) = (current_id, current_kind) {
                            self.ls_kinds.insert(id, kind);
                        }
                        current_id = None;
                        current_kind = None;
                    }
                }
                '\\' => match self.chars.next() {
                    Some(c) if c.is_ascii_alphabetic() => {
                        let (word, param) = read_control_word(&mut self.chars, c);
                        match word.as_str() {
                            "listid" => {
                                if let Some(id) = param {
                                    current_id = Some(id);
                                    if let Some(kind) = current_kind {
                                        self.ls_kinds.insert(id, kind);
                                    }
                                }
                            }
                            "levelnfc" | "levelnfcn" => {
                                current_kind = match param {
                                    Some(23) | Some(255) => Some(ListKind::Bullet),
                                    Some(0) => Some(ListKind::Numbered),
                                    Some(n) if (1..=6).contains(&n) => Some(ListKind::Numbered),
                                    _ => current_kind.or(Some(ListKind::Bullet)),
                                };
                                if let (Some(id), Some(kind)) = (current_id, current_kind) {
                                    self.ls_kinds.insert(id, kind);
                                }
                            }
                            _ => {}
                        }
                    }
                    Some('\'') => {
                        let _hex: String = self.chars.by_ref().take(2).collect();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn apply_control_word(&mut self, word: &str, param: Option<i32>) {
        match word {
            "par" | "line" => {
                if self.in_table {
                    // Soft break inside a cell.
                    self.text.push('\n');
                } else {
                    self.flush_table();
                    self.flush_paragraph();
                }
            }
            "pard" => {
                // Reset paragraph properties; finish any open table first if leaving it.
                if !self.in_table {
                    self.flush_table();
                }
                self.para_list = None;
                self.para_level = 0;
                self.current_ls = None;
            }
            "tab" => self.text.push('\t'),
            "emdash" => self.text.push('—'),
            "endash" => self.text.push('–'),
            "bullet" => {
                self.text.push('•');
                if self.para_list.is_none() {
                    self.para_list = Some(ListKind::Bullet);
                }
            }
            "lquote" | "rquote" => self.text.push('\''),
            "ldblquote" | "rdblquote" => self.text.push('"'),
            "pnlvlblt" => {
                self.para_list = Some(ListKind::Bullet);
                self.pending_pn = Some(ListKind::Bullet);
            }
            "pnlvlbody" | "pndec" => {
                self.para_list = Some(ListKind::Numbered);
                self.pending_pn = Some(ListKind::Numbered);
            }
            "ls" => {
                if let Some(id) = param {
                    self.current_ls = Some(id);
                    let kind = self
                        .ls_kinds
                        .get(&id)
                        .copied()
                        .or(self.pending_pn)
                        .unwrap_or(ListKind::Bullet);
                    self.para_list = Some(kind);
                }
            }
            "ilvl" => {
                if let Some(level) = param {
                    self.para_level = level.max(0) as u32;
                }
            }
            "li" => {
                // Heuristic level from left indent (twips → ~720 per level).
                if let Some(twips) = param {
                    if twips > 0 && self.para_list.is_some() {
                        self.para_level = ((twips as u32) / 720).saturating_sub(0).min(8);
                        if twips >= 720 {
                            self.para_level = ((twips as u32 - 1) / 720).min(8);
                        }
                    }
                }
            }
            "trowd" => {
                self.in_table = true;
                if self.table_rows.is_none() {
                    self.flush_paragraph();
                    self.table_rows = Some(Vec::new());
                }
            }
            "intbl" => {
                self.in_table = true;
                if self.table_rows.is_none() {
                    self.table_rows = Some(Vec::new());
                }
            }
            "cell" => {
                self.in_table = true;
                if self.table_rows.is_none() {
                    self.table_rows = Some(Vec::new());
                }
                self.row_cells.push(std::mem::take(&mut self.text));
            }
            "row" => {
                if !self.row_cells.is_empty() || !self.text.is_empty() {
                    if !self.text.is_empty() {
                        self.row_cells.push(std::mem::take(&mut self.text));
                    }
                    let rows = self.table_rows.get_or_insert_with(Vec::new);
                    rows.push(std::mem::take(&mut self.row_cells));
                }
                self.in_table = true;
            }
            "page" => {
                self.flush_table();
                self.flush_paragraph();
            }
            _ => {}
        }
    }

    fn flush_paragraph(&mut self) {
        let raw = std::mem::take(&mut self.text);
        let text = raw.trim_end_matches(['\r', '\n']).to_string();
        // Apply pending pn when paragraph has content and no stronger list signal.
        if self.para_list.is_none() {
            self.para_list = self.pending_pn;
        }
        if text.is_empty() && self.para_list.is_none() {
            self.para_list = None;
            self.para_level = 0;
            self.current_ls = None;
            return;
        }
        if text.is_empty() {
            self.para_list = None;
            self.para_level = 0;
            self.current_ls = None;
            return;
        }

        let mut para = Paragraph::new();
        para.runs = vec![Run::new_text(text)];
        if let Some(kind) = self.para_list.or(self.pending_pn) {
            let numbering_id = match kind {
                ListKind::Bullet => 1,
                ListKind::Numbered => 2,
            };
            para.format.numbering = Some(NumberingRef {
                numbering_id,
                level: self.para_level,
            });
        }
        self.blocks.push(Block::Paragraph(para));
        self.para_list = None;
        self.para_level = 0;
        self.current_ls = None;
        // `pending_pn` persists across items in the same old-style list until `\pard` clears via
        // a non-list paragraph — keep it so consecutive `\par` items stay listed.
    }

    fn flush_table(&mut self) {
        let Some(rows) = self.table_rows.take() else {
            self.row_cells.clear();
            self.in_table = false;
            return;
        };
        if !self.row_cells.is_empty() || !self.text.is_empty() {
            // Incomplete row — drop trailing text into a cell if needed.
            if !self.text.is_empty() {
                self.row_cells.push(std::mem::take(&mut self.text));
            }
        }
        let mut rows = rows;
        if !self.row_cells.is_empty() {
            rows.push(std::mem::take(&mut self.row_cells));
        }
        self.in_table = false;
        if rows.is_empty() {
            return;
        }
        let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        if cols == 0 {
            return;
        }
        let mut table = Table::new(rows.len() as u32, cols as u32);
        for (ri, row) in rows.into_iter().enumerate() {
            for (ci, cell_text) in row.into_iter().enumerate() {
                if let Some(cell) = table.rows.get_mut(ri).and_then(|r| r.cells.get_mut(ci)) {
                    *cell = TableCell::with_text(cell_text.trim());
                }
            }
        }
        self.blocks.push(Block::Table(table));
    }

    fn peek_control_word(&mut self) -> Option<String> {
        let mut clone = self.chars.clone();
        // skip whitespace
        while let Some(&c) = clone.peek() {
            if c == ' ' || c == '\n' || c == '\r' {
                clone.next();
            } else {
                break;
            }
        }
        if clone.next()? != '\\' {
            return None;
        }
        let first = clone.next()?;
        if !first.is_ascii_alphabetic() {
            return None;
        }
        let (word, _) = read_control_word(&mut clone, first);
        Some(word)
    }

    fn skip_group_from_here(&mut self) {
        // Current `{` already consumed; skip until matching `}`.
        self.skip_balanced_from_depth(0);
    }

    fn skip_balanced_from_depth(&mut self, mut depth: i32) {
        while let Some(ch) = self.chars.next() {
            match ch {
                '{' => depth += 1,
                '}' if depth == 0 => break,
                '}' => depth -= 1,
                '\\' => match self.chars.next() {
                    Some('\'') => {
                        let _hex: String = self.chars.by_ref().take(2).collect();
                    }
                    Some(c) if c.is_ascii_alphabetic() => {
                        let _ = read_control_word(&mut self.chars, c);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn read_control_word(
    chars: &mut Peekable<Chars<'_>>,
    first: char,
) -> (String, Option<i32>) {
    let mut word = String::from(first);
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            word.push(c);
            chars.next();
        } else {
            break;
        }
    }
    let mut param = None;
    if chars.peek() == Some(&'-') || chars.peek().map(|c| c.is_ascii_digit()) == Some(true) {
        let mut num = String::new();
        if chars.peek() == Some(&'-') {
            num.push('-');
            chars.next();
        }
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                num.push(c);
                chars.next();
            } else {
                break;
            }
        }
        param = num.parse().ok();
    }
    if chars.peek() == Some(&' ') {
        chars.next();
    }
    (word, param)
}

fn apply_control_symbol(ch: char, out: &mut String) {
    match ch {
        '~' => out.push('\u{00A0}'),
        '-' => out.push('-'),
        '_' => out.push('-'),
        ':' => out.push(':'),
        '\\' => out.push('\\'),
        '{' => out.push('{'),
        '}' => out.push('}'),
        _ => {}
    }
}

trait TableCellExt {
    fn with_text(text: &str) -> Self;
}

impl TableCellExt for TableCell {
    fn with_text(text: &str) -> Self {
        let mut cell = TableCell::new();
        if !text.is_empty() {
            cell.blocks = vec![Block::Paragraph(Paragraph::with_text(text))];
        }
        cell
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::Block;

    #[test]
    fn extracts_rtf_text() {
        let rtf = r#"{\rtf1\ansi Hello \par World}"#;
        let doc = import_rtf(rtf.as_bytes()).unwrap();
        let full = doc
            .sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph())
            .map(|p| p.full_text())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(full.contains("Hello"));
        assert!(full.contains("World"));
    }

    #[test]
    fn u_f23_s4_rtf_list_bullet() {
        let rtf = r#"{\rtf1\ansi\deff0
{\*\pn\pnlvlblt\pnf1\pnindent0{\pntxtb\'B7}}
\pard\li720\fi-360{\pntext\f1\'B7\tab}First\par
\pard\li720\fi-360{\pntext\f1\'B7\tab}Second\par
}"#;
        let doc = import_rtf(rtf.as_bytes()).unwrap();
        let paras: Vec<_> = doc.sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph())
            .collect();
        assert!(paras.len() >= 2, "expected list paragraphs, got {}", paras.len());
        let first = paras[0];
        let num = first.format.numbering.expect("bullet numbering");
        assert_eq!(num.numbering_id, 1);
        assert!(first.full_text().contains("First"));
        // Marker group `{\pntext...}` is skipped — body should not be only the bullet glyph.
        assert_ne!(first.full_text().trim(), "•");
        let second = paras[1];
        assert_eq!(
            second.format.numbering.map(|n| n.numbering_id),
            Some(1)
        );
        assert!(second.full_text().contains("Second"));
    }

    #[test]
    fn u_f23_s4_rtf_list_numbered() {
        let rtf = r#"{\rtf1\ansi\deff0
{\*\pn\pnlvlbody\pndec\pnstart1\pnindent0{\pntxta.}}
\pard\li720\fi-360{\pntext 1.\tab}One\par
\pard\li720\fi-360{\pntext 2.\tab}Two\par
}"#;
        let doc = import_rtf(rtf.as_bytes()).unwrap();
        let paras: Vec<_> = doc.sections[0]
            .blocks
            .iter()
            .filter_map(|b| b.paragraph())
            .collect();
        assert!(paras.len() >= 2);
        assert_eq!(
            paras[0].format.numbering.map(|n| n.numbering_id),
            Some(2),
            "numbered list should use catalog id 2"
        );
        assert!(paras[0].full_text().contains("One"));
        assert_eq!(
            paras[1].format.numbering.map(|n| n.numbering_id),
            Some(2)
        );
    }

    #[test]
    fn u_f23_s4_rtf_table() {
        let rtf = r#"{\rtf1\ansi
\trowd\cellx2000\cellx4000
\intbl A1\cell B1\cell\row
\trowd\cellx2000\cellx4000
\intbl A2\cell B2\cell\row
\par
}"#;
        let doc = import_rtf(rtf.as_bytes()).unwrap();
        let table = doc.sections[0]
            .blocks
            .iter()
            .find_map(|b| match b {
                Block::Table(t) => Some(t),
                _ => None,
            })
            .expect("expected a table block");
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].cells.len(), 2);
        assert_eq!(
            table.rows[0].cells[0]
                .blocks
                .iter()
                .filter_map(|b| b.paragraph())
                .map(|p| p.full_text())
                .collect::<String>(),
            "A1"
        );
        assert_eq!(
            table.rows[1].cells[1]
                .blocks
                .iter()
                .filter_map(|b| b.paragraph())
                .map(|p| p.full_text())
                .collect::<String>(),
            "B2"
        );
    }
}
