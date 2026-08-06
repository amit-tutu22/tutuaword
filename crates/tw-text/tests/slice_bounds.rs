//! TextBuffer slice must not panic when the char range end exceeds run length.

use tw_model::NodeId;
use tw_text::TextBuffer;

#[test]
fn slice_clamps_end_to_run_length() {
    let run_id = NodeId::new();
    let mut buffer = TextBuffer::new();
    buffer.register(run_id, "hello");

    let slice = buffer.slice(run_id, 0..8);
    assert_eq!(slice, "hello");
}

#[test]
fn slice_end_at_run_length_does_not_panic() {
    let run_id = NodeId::new();
    let mut buffer = TextBuffer::new();
    buffer.register(run_id, "hello");
    let slice = buffer.slice(run_id, 0..5);
    assert_eq!(slice, "hello");
    let empty = buffer.slice(run_id, 5..5);
    assert_eq!(empty, "");
}

#[test]
fn slice_start_past_end_returns_empty() {
    let run_id = NodeId::new();
    let mut buffer = TextBuffer::new();
    buffer.register(run_id, "hi");
    assert_eq!(buffer.slice(run_id, 3..10), "");
}
