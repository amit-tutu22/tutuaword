//! Character-index string utilities for run text (R2.2 single storage).

use std::borrow::Cow;
use std::ops::Range;

/// Character length of a UTF-8 string.
pub fn char_len(text: &str) -> usize {
    text.chars().count()
}

/// Slice by character indices; clamps when `end` exceeds length.
pub fn slice_chars(text: &str, char_range: Range<usize>) -> Cow<'_, str> {
    let len = char_len(text);
    let start = char_range.start.min(len);
    let end = char_range.end.min(len);
    if start >= end {
        return Cow::Borrowed("");
    }
    let start_byte = text
        .char_indices()
        .nth(start)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    let end_byte = if end >= len {
        text.len()
    } else {
        text.char_indices().nth(end).map(|(i, _)| i).unwrap_or(text.len())
    };
    Cow::Owned(text[start_byte..end_byte].to_string())
}
