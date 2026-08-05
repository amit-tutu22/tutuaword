use tw_model::Document;

use crate::RtfError;

pub fn import_rtf(source: &[u8]) -> Result<Document, RtfError> {
    let text = String::from_utf8_lossy(source);
    let trimmed = text.trim_start();
    if !trimmed.starts_with("{\\rtf") {
        return Err(RtfError::InvalidFormat);
    }
    Ok(Document::from_plain_text(&extract_rtf_text(trimmed)))
}

fn extract_rtf_text(rtf: &str) -> String {
    let mut out = String::new();
    let mut chars = rtf.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.next() {
                Some('\'') => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        out.push(byte as char);
                    }
                }
                Some('*') => skip_optional_destination(&mut chars),
                Some('\n' | '\r') => {}
                Some(c) if c.is_ascii_alphabetic() => {
                    let word = read_control_word(&mut chars, c);
                    apply_control_word(&word, &mut out);
                }
                Some(c) => apply_control_symbol(c, &mut out),
                None => break,
            },
            '{' | '}' => {}
            c if !c.is_control() => out.push(c),
            _ => {}
        }
    }
    out
}

fn read_control_word(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, first: char) -> String {
    let mut word = String::from(first);
    while let Some(&c) = chars.peek() {
        if c.is_ascii_alphabetic() {
            word.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if chars.peek() == Some(&'-') || chars.peek().map(|c| c.is_ascii_digit()) == Some(true) {
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() || c == '-' {
                chars.next();
            } else {
                break;
            }
        }
    }
    if chars.peek() == Some(&' ') {
        chars.next();
    }
    word
}

fn apply_control_word(word: &str, out: &mut String) {
    match word {
        "par" | "line" => out.push('\n'),
        "tab" => out.push('\t'),
        "emdash" => out.push('—'),
        "endash" => out.push('–'),
        "bullet" => out.push('•'),
        "lquote" | "rquote" => out.push('\''),
        "ldblquote" | "rdblquote" => out.push('"'),
        _ => {}
    }
}

fn apply_control_symbol(ch: char, out: &mut String) {
    match ch {
        '~' => out.push('\u{00A0}'),
        '-' => out.push('-'),
        ':' => out.push(':'),
        '\\' => out.push('\\'),
        '{' => out.push('{'),
        '}' => out.push('}'),
        _ => {}
    }
}

fn skip_optional_destination(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    let mut depth = 0;
    while let Some(ch) = chars.next() {
        match ch {
            '{' => depth += 1,
            '}' if depth == 0 => break,
            '}' => depth -= 1,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
