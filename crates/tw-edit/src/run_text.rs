//! Run text helpers — single store is `RunContent::Text(String)` in the document tree.

use std::ops::Range;

use tw_model::{Document, NodeId, Run};
use tw_text::slice_chars;

pub fn run_with_id<'a>(doc: &'a Document, run_id: NodeId) -> Option<&'a Run> {
    let loc = doc.find_run_location(run_id)?;
    doc.run_at(loc)
}

pub fn run_char_len(run: &Run) -> usize {
    run.text().chars().count()
}

pub fn run_char_len_by_id(doc: &Document, run_id: NodeId) -> usize {
    run_with_id(doc, run_id).map(run_char_len).unwrap_or(0)
}

pub fn run_slice(run: &Run, char_range: Range<usize>) -> String {
    slice_chars(run.text(), char_range).into_owned()
}

pub fn run_slice_by_id(doc: &Document, run_id: NodeId, char_range: Range<usize>) -> String {
    run_with_id(doc, run_id)
        .map(|run| run_slice(run, char_range))
        .unwrap_or_default()
}
