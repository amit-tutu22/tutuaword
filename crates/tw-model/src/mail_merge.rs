//! Mail merge: CSV data source + field replacement (F26.S2).

use std::collections::BTreeMap;

use crate::document::Document;
use crate::nodes::{Block, RunContent};
use crate::vocabulary::{merge_field_placeholder, FieldType};

/// Parsed CSV mail-merge data source (F26.S2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailMergeDataSource {
    pub headers: Vec<String>,
    pub rows: Vec<BTreeMap<String, String>>,
}

/// Errors from CSV / mail-merge operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MailMergeError {
    EmptyCsv,
    MissingHeader,
    ColumnCount { row: usize, expected: usize, got: usize },
}

impl std::fmt::Display for MailMergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCsv => write!(f, "CSV is empty"),
            Self::MissingHeader => write!(f, "CSV has no header row"),
            Self::ColumnCount {
                row,
                expected,
                got,
            } => write!(
                f,
                "CSV row {row} has {got} columns, expected {expected}"
            ),
        }
    }
}

impl std::error::Error for MailMergeError {}

/// Parse a simple CSV (comma-separated; quoted fields with `""` escapes).
pub fn parse_mail_merge_csv(csv: &str) -> Result<MailMergeDataSource, MailMergeError> {
    let records = parse_csv_records(csv);
    if records.is_empty() {
        return Err(MailMergeError::EmptyCsv);
    }
    let headers = records[0].clone();
    if headers.is_empty() || headers.iter().all(|h| h.trim().is_empty()) {
        return Err(MailMergeError::MissingHeader);
    }
    let headers: Vec<String> = headers.into_iter().map(|h| h.trim().to_string()).collect();
    let expected = headers.len();
    let mut rows = Vec::with_capacity(records.len().saturating_sub(1));
    for (i, record) in records.into_iter().skip(1).enumerate() {
        if record.len() != expected {
            return Err(MailMergeError::ColumnCount {
                row: i + 1,
                expected,
                got: record.len(),
            });
        }
        let mut map = BTreeMap::new();
        for (header, value) in headers.iter().zip(record.into_iter()) {
            map.insert(header.clone(), value);
        }
        rows.push(map);
    }
    Ok(MailMergeDataSource { headers, rows })
}

fn parse_csv_records(csv: &str) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = csv.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            }
            '"' => in_quotes = true,
            ',' if !in_quotes => {
                current.push(std::mem::take(&mut field));
            }
            '\n' if !in_quotes => {
                current.push(std::mem::take(&mut field));
                if !current.iter().all(|f| f.is_empty()) {
                    records.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
            '\r' if !in_quotes => {
                // ignore CR; LF handles record end
            }
            _ => field.push(c),
        }
    }
    if in_quotes || !field.is_empty() || !current.is_empty() {
        current.push(field);
        if !current.iter().all(|f| f.is_empty()) {
            records.push(current);
        }
    }
    records
}

/// Replace merge fields (and literal `«Name»` placeholders) with row values.
pub fn apply_mail_merge_row(doc: &mut Document, row: &BTreeMap<String, String>) {
    for section in &mut doc.sections {
        for block in &mut section.blocks {
            apply_mail_merge_block(block, row);
        }
        for hf in section.headers.values_mut().chain(section.footers.values_mut()) {
            for block in &mut hf.blocks {
                apply_mail_merge_block(block, row);
            }
        }
    }
}

fn apply_mail_merge_block(block: &mut Block, row: &BTreeMap<String, String>) {
    match block {
        Block::Paragraph(para) => {
            for run in &mut para.runs {
                match &mut run.content {
                    RunContent::Field(field) if field.field_type == FieldType::MergeField => {
                        let name = field
                            .merge_name
                            .clone()
                            .or_else(|| merge_name_from_instruction(field.instruction.as_deref()))
                            .unwrap_or_default();
                        let value = lookup_row_value(row, &name)
                            .unwrap_or_default()
                            .to_string();
                        run.content = RunContent::Text(value);
                    }
                    RunContent::Text(text) => {
                        *text = replace_guillemet_placeholders(text, row);
                    }
                    _ => {}
                }
            }
        }
        Block::Table(table) => {
            for row_cells in &mut table.rows {
                for cell in &mut row_cells.cells {
                    for nested in &mut cell.blocks {
                        apply_mail_merge_block(nested, row);
                    }
                }
            }
        }
        _ => {}
    }
}

fn merge_name_from_instruction(instr: Option<&str>) -> Option<String> {
    let instr = instr?;
    let upper = instr.to_ascii_uppercase();
    let idx = upper.find("MERGEFIELD")?;
    let rest = instr[idx + "MERGEFIELD".len()..].trim();
    let name = rest
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches('"')
        .trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn lookup_row_value<'a>(row: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    if let Some(v) = row.get(name) {
        return Some(v.as_str());
    }
    let lower = name.to_ascii_lowercase();
    row.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(&lower))
        .map(|(_, v)| v.as_str())
}

fn replace_guillemet_placeholders(text: &str, row: &BTreeMap<String, String>) -> String {
    let mut out = text.to_string();
    for (key, value) in row {
        let needle = merge_field_placeholder(key);
        if out.contains(&needle) {
            out = out.replace(&needle, value);
        }
    }
    out
}

/// Clone the template once per CSV row and apply merge replacement.
pub fn generate_mail_merge_documents(
    template: &Document,
    data: &MailMergeDataSource,
) -> Vec<Document> {
    data.rows
        .iter()
        .map(|row| {
            let mut doc = template.clone();
            apply_mail_merge_row(&mut doc, row);
            doc
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_csv() {
        let src = parse_mail_merge_csv("Name,City\nAda,Paris\nGrace,London\n").unwrap();
        assert_eq!(src.headers, vec!["Name", "City"]);
        assert_eq!(src.rows.len(), 2);
        assert_eq!(src.rows[0].get("Name").map(String::as_str), Some("Ada"));
    }
}
