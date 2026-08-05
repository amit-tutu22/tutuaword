use ropey::Rope;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;
use tw_model::NodeId;

pub struct TextBuffer {
    ropes: HashMap<NodeId, Rope>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            ropes: HashMap::new(),
        }
    }

    pub fn register(&mut self, run_id: NodeId, text: &str) {
        self.ropes.insert(run_id, Rope::from_str(text));
    }

    pub fn unregister(&mut self, run_id: NodeId) {
        self.ropes.remove(&run_id);
    }

    pub fn insert(&mut self, run_id: NodeId, char_offset: usize, text: &str) {
        if let Some(rope) = self.ropes.get_mut(&run_id) {
            let byte_offset = rope.char_to_byte(char_offset);
            rope.insert(byte_offset, text);
        }
    }

    pub fn delete(&mut self, run_id: NodeId, char_range: Range<usize>) {
        if let Some(rope) = self.ropes.get_mut(&run_id) {
            let start = rope.char_to_byte(char_range.start);
            let end = rope.char_to_byte(char_range.end);
            rope.remove(start..end);
        }
    }

    pub fn slice(&self, run_id: NodeId, char_range: Range<usize>) -> Cow<'_, str> {
        if let Some(rope) = self.ropes.get(&run_id) {
            let start = rope.char_to_byte(char_range.start);
            let end = rope.char_to_byte(char_range.end);
            Cow::Owned(rope.slice(start..end).to_string())
        } else {
            Cow::Borrowed("")
        }
    }

    pub fn len(&self, run_id: NodeId) -> usize {
        self.ropes.get(&run_id).map(|r| r.len_chars()).unwrap_or(0)
    }

    pub fn to_string(&self, run_id: NodeId) -> String {
        self.ropes
            .get(&run_id)
            .map(|r| r.to_string())
            .unwrap_or_default()
    }

    pub fn sync_from_run(&mut self, run_id: NodeId, text: &str) {
        self.ropes.insert(run_id, Rope::from_str(text));
    }
}
