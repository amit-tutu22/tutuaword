//! Normalize clipboard text so paste does not store `.notdef` tofu glyphs.
//!
//! Word / PowerPoint often put Wingdings/Symbol Private Use Area codepoints or
//! literal `\n` into plain text. Calibri cannot draw PUA, and our layout shapes
//! `\n` like any other character → empty square boxes. Map those to Unicode or
//! strip them before insert.

/// Map a single clipboard character to what we store in the document.
///
/// Returns `None` to drop the character (controls other than tab; handled
/// newlines are split into paragraphs by the caller).
pub fn map_paste_char(ch: char) -> Option<char> {
    match ch {
        '\n' | '\r' => None,
        '\t' => Some('\t'),
        '\u{00A0}' => Some(' '), // NBSP
        // Zero-width / BOM / soft hyphen — keep layout clean.
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{00AD}' => None,
        // Common Symbol / Wingdings PUA arrows & bullets (Windows Word paste).
        '\u{F0E0}' | '\u{F0E1}' | '\u{F0E2}' | '\u{F0E3}' | '\u{F0E4}' | '\u{F0E5}'
        | '\u{F0E6}' | '\u{F0E7}' | '\u{F0E8}' | '\u{F0E9}' | '\u{F0EA}' | '\u{F0EB}'
        | '\u{F0EC}' | '\u{F0ED}' | '\u{F0EE}' | '\u{F0EF}' => Some('→'),
        '\u{F0B6}' | '\u{F0B7}' | '\u{F0A7}' | '\u{F0A8}' => Some('•'),
        '\u{F035}' | '\u{F0FC}' | '\u{F0FD}' | '\u{F0FE}' | '\u{F0FF}' => Some('•'),
        // Generic PUA (F000–F8FF): middle dot rather than .notdef tofu.
        c if ('\u{F000}'..='\u{F8FF}').contains(&c) => Some('·'),
        // Other C0/C1 controls (except tab handled above).
        c if c.is_control() => None,
        c => Some(c),
    }
}

/// Normalize paste text: map symbols, drop junk controls. Newlines become `\n`
/// so callers can split into paragraphs.
pub fn normalize_paste_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push('\n');
            continue;
        }
        if ch == '\n' {
            out.push('\n');
            continue;
        }
        if let Some(mapped) = map_paste_char(ch) {
            out.push(mapped);
        }
    }
    out
}

/// Split normalized paste text into paragraph lines (keeps empty lines).
pub fn split_paste_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split('\n').collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_wingdings_pua_to_readable() {
        assert_eq!(map_paste_char('\u{F035}'), Some('•'));
        assert_eq!(map_paste_char('\u{F0E0}'), Some('→'));
        assert_eq!(map_paste_char('\u{F050}'), Some('·'));
    }

    #[test]
    fn normalize_collapses_crlf_and_strips_controls() {
        let got = normalize_paste_text("a\r\nb\u{0007}c\n");
        assert_eq!(got, "a\nbc\n");
    }

    #[test]
    fn split_keeps_empty_lines() {
        assert_eq!(split_paste_lines("a\n\nb"), vec!["a", "", "b"]);
    }
}
