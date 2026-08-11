//! Character slice must not panic when the char range end exceeds run length.

use tw_text::{char_len, slice_chars};

#[test]
fn slice_clamps_end_beyond_length() {
    let text = "hello";
    assert_eq!(char_len(text), 5);
    let slice = slice_chars(text, 0..8);
    assert_eq!(slice, "hello");
}

#[test]
fn slice_exact_range() {
    let text = "hello";
    let slice = slice_chars(text, 0..5);
    assert_eq!(slice, "hello");
    let empty = slice_chars(text, 5..5);
    assert_eq!(empty, "");
}

#[test]
fn slice_start_beyond_length_returns_empty() {
    let text = "hi";
    assert_eq!(slice_chars(text, 3..10), "");
}
