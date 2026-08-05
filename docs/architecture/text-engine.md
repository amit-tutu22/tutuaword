# Text Engine

The text engine (`tw-text`) manages the mutable text buffer, cursor/selection state, and input handling. It sits between the UI (which captures raw input events) and the document model (which stores the tree structure).

## Architecture

```
Flutter Input Events
        │
        ▼
  Input Handler (Dart)
  ├── Keyboard → KeyCommand
  ├── Mouse → SelectionCommand
  ├── IME → CompositionCommand
  └── Clipboard → PasteCommand
        │
        ▼
  tw-edit::apply(command)
        │
        ├── tw-text::TextBuffer (rope operations)
        └── tw-model (structural changes)
```

The text engine does not handle input directly. Flutter captures events, translates them to `Command` enums, and sends them to `tw-edit`. The text engine provides the low-level text manipulation primitives that commands use.

## Text Buffer

Each run's text content is backed by a `ropey::Rope` for O(log n) insert/delete at arbitrary positions.

```rust
pub struct TextBuffer {
    ropes: HashMap<NodeId, Rope>,
}

impl TextBuffer {
    pub fn insert(&mut self, run_id: NodeId, char_offset: usize, text: &str);
    pub fn delete(&mut self, run_id: NodeId, char_range: Range<usize>);
    pub fn slice(&self, run_id: NodeId, char_range: Range<usize>) -> Cow<str>;
    pub fn len(&self, run_id: NodeId) -> usize;
    pub fn char_at(&self, run_id: NodeId, char_offset: usize) -> Option<char>;
}
```

### Why Rope, Not Gap Buffer

| Property | Rope | Gap Buffer |
|----------|------|------------|
| Insert/delete at cursor | O(log n) | O(1) amortized at gap, O(n) to move gap |
| Insert/delete at arbitrary position | O(log n) | O(n) to move gap |
| Memory overhead | ~2x string size | ~1.1x string size |
| Thread safety | Immutable slices possible | Requires exclusive access |
| Large document (500 pages) | Consistent performance | Gap movement dominates |

For a Word-class editor where paste operations, search/replace, and CRDT merges insert at arbitrary positions, rope's consistent O(log n) wins. The ~2x memory overhead is acceptable given the 300 MB budget for a 500-page document.

### Buffer Lifecycle

- Created when a run is created (empty rope)
- Grows/shrinks with insert/delete operations
- Removed when a run is deleted (undo may restore it)
- Not persisted directly — serialized from model on save, rebuilt from model on load

## Position Types

Three position representations are used, each for a different purpose:

```rust
/// Character offset within a single run (UTF-32 codepoint index)
pub struct CharIndex(pub usize);

/// Byte offset within a run's rope (for rope operations)
pub struct ByteIndex(pub usize);

/// Document-wide position: (run_id, char_offset)
pub struct DocPosition {
    pub run_id: NodeId,
    pub char_offset: CharIndex,
}

/// Document-wide range
pub struct DocRange {
    pub start: DocPosition,
    pub end: DocPosition,
}
```

Conversion between `CharIndex` and `ByteIndex` uses the rope's internal UTF-8 indexing. All UI-facing positions are `CharIndex` (grapheme-aligned). All rope operations use `ByteIndex`.

## Cursor

The cursor is a zero-width selection — a collapsed `DocRange`.

```rust
pub struct Cursor {
    pub position: DocPosition,
    pub affinity: CursorAffinity,
    pub preferred_x: Option<f32>,  // for up/down movement
}

pub enum CursorAffinity {
    Upstream,    // cursor sticks to end of previous line
    Downstream,  // cursor sticks to start of next line
}
```

### Cursor Movement

| Input | Command | Behavior |
|-------|---------|----------|
| Arrow left/right | `MoveCursor { direction, extend }` | Move by grapheme cluster |
| Arrow up/down | `MoveCursorVertical { direction, extend }` | Move to same x-position on adjacent line |
| Home | `MoveCursorToLineStart { extend }` | Move to start of current line |
| End | `MoveCursorToLineEnd { extend }` | Move to end of current line |
| Ctrl+Arrow | `MoveCursorByWord { direction, extend }` | Move by word boundary (UAX #29) |
| Ctrl+Home/End | `MoveCursorToDocumentStart/End { extend }` | Move to document boundary |
| Page Up/Down | `MoveCursorByPage { direction, extend }` | Move by viewport height |

Vertical movement uses `preferred_x` to maintain horizontal position across lines of different widths. When the target line is shorter, `CursorAffinity` determines whether the cursor lands at end-of-line or start-of-next-line.

### Word Boundaries

Word boundaries follow Unicode Standard Annex #29 (Unicode Text Segmentation):

- **Word break** before: space, punctuation (in most scripts), script boundaries
- **Word break** after: space, punctuation
- **No break** within: letter sequences, digit sequences, emoji sequences

Implemented via `icu_segmenter::WordSegmenter` with locale-aware rules.

## Selection

A selection is a non-collapsed `DocRange` with additional metadata.

```rust
pub struct Selection {
    pub range: DocRange,
    pub mode: SelectionMode,
}

pub enum SelectionMode {
    Character,
    Word,
    Line,
    Paragraph,
    Table,
}
```

### Selection Operations

| Input | Behavior |
|-------|----------|
| Shift+Arrow | Extend selection in arrow direction |
| Shift+Click | Extend selection to click position |
| Double-click | Select word at click position |
| Triple-click | Select paragraph at click position |
| Ctrl+A | Select all |
| Click+drag | Character selection from anchor to current position |

### Multi-Run Selection

Selections frequently span multiple runs and paragraphs. The text engine provides:

```rust
pub fn iter_runs_in_range(doc: &Document, range: &DocRange) -> impl Iterator<Item=RunSlice>;

pub struct RunSlice {
    pub run_id: NodeId,
    pub char_range: Range<CharIndex>,
    pub text: /* borrowed from rope */,
}
```

## Typing

Character insertion follows this flow:

1. UI captures key event with Unicode codepoint
2. If IME composition is active, route to composition handler (see below)
3. If selection is non-empty, delete selected range first
4. Determine target run and offset from cursor position
5. If cursor is at a run boundary with different formatting, split or merge runs as needed
6. `tw-text::insert(run_id, offset, char)` — O(log n)
7. `tw-edit` records inverse: `DeleteRange { run_id, range }`
8. Advance cursor by one grapheme cluster
9. Mark page dirty for re-layout

### Auto-Run-Splitting

When the user types with different formatting than the current run:

```
Before:  [Run A: "Hello" (normal)] [Run B: "World" (bold)]
Cursor at offset 5 in Run A, user presses Bold then types "X"

After:   [Run A: "Hello" (normal)] [Run C: "X" (bold)] [Run B: "World" (bold)]
         Run C is a new run with bold formatting
```

When formatting matches an adjacent run, merge instead of creating a new run:

```
Before:  [Run A: "Hello" (bold)] [Run B: "World" (bold)]
Cursor at offset 5 in Run A, user types " "

After:   [Run A: "Hello World" (bold)]
         Runs A and B merged (same formatting)
```

Run normalization (merging adjacent runs with identical formatting) runs after every edit operation.

## IME (Input Method Editor)

IME support is required for Chinese, Japanese, Korean, Hindi, Arabic, and other scripts that require composition.

```rust
pub struct CompositionState {
    pub active: bool,
    pub run_id: NodeId,
    pub start_offset: CharIndex,
    pub text: String,           // composed text so far
    pub cursor_in_composition: usize,
    pub underline: bool,        // show IME underline
}
```

### IME Flow

1. `compositionStart` → create `CompositionState`, record start position
2. `compositionUpdate(text)` → replace composition range with updated text, show underline
3. `compositionEnd(text)` → finalize: commit text as a normal insert, clear composition state
4. During composition, layout shows the composed text with an underline; the document model is not modified until `compositionEnd`

Flutter provides IME events via `TextInputConnection`. The FFI bridge translates these to `CompositionCommand` enums.

## Clipboard

### Copy

1. Get current selection range
2. Serialize selected content to multiple formats:
   - **Plain text** — concatenated run text
   - **HTML** — formatted HTML fragment
   - **Native** — full model subtree (preserves all formatting)
   - **DOCX fragment** — OOXML snippet (Phase 3+)
3. Write to system clipboard via Flutter

### Cut

Copy + delete selection.

### Paste

1. Read clipboard formats (priority: Native > DOCX > HTML > Plain text)
2. If Native format: deserialize model subtree, insert at cursor
3. If HTML: parse to model nodes via `tw-html`, insert at cursor
4. If Plain text: insert as single run with current formatting
5. If paste would split runs, perform run splitting/merging
6. Record undo inverse

### Internal Drag-and-Drop

Uses the same Native serialization format as clipboard. Drop position determined by hit-testing the layout engine's line map.

## Keyboard Shortcuts

Shortcuts are handled in Flutter (not Rust) because they involve UI actions beyond text editing.

### Text Editing (handled by tw-edit)

| Shortcut | Command |
|----------|---------|
| Ctrl+Z / Cmd+Z | Undo |
| Ctrl+Y / Cmd+Shift+Z | Redo |
| Ctrl+X / Cmd+X | Cut |
| Ctrl+C / Cmd+C | Copy |
| Ctrl+V / Cmd+V | Paste |
| Ctrl+A / Cmd+A | Select all |
| Backspace | Delete previous grapheme |
| Delete | Delete next grapheme |
| Ctrl+Backspace | Delete previous word |
| Ctrl+Delete | Delete next word |
| Enter | Insert paragraph break |
| Shift+Enter | Insert line break (soft return) |
| Tab | Insert tab / advance to next table cell |
| Shift+Tab | Move to previous table cell |

### Formatting (handled by tw-edit)

| Shortcut | Command |
|----------|---------|
| Ctrl+B / Cmd+B | Toggle bold |
| Ctrl+I / Cmd+I | Toggle italic |
| Ctrl+U / Cmd+U | Toggle underline |
| Ctrl+Shift+> | Increase font size |
| Ctrl+Shift+< | Decrease font size |

### UI Actions (handled by Flutter)

| Shortcut | Action |
|----------|--------|
| Ctrl+S / Cmd+S | Save |
| Ctrl+O / Cmd+O | Open |
| Ctrl+N / Cmd+N | New document |
| Ctrl+P / Cmd+P | Print |
| Ctrl+F / Cmd+F | Find |
| Ctrl+H / Cmd+H | Find and replace |
| Ctrl+Shift+8 | Toggle formatting marks |
| F11 | Full screen |

## RTL and Bidirectional Text

Bidirectional text support is handled at the layout/shaping layer, not the text engine. The text engine stores text in logical order (as typed). The layout engine applies the Unicode Bidirectional Algorithm (UAX #9) during line breaking.

Cursor movement in RTL text:
- Arrow left/right move in visual order (not logical order)
- Home/End go to visual start/end of line
- The text engine stores logical positions; the layout engine provides visual-to-logical mapping for cursor rendering

## Performance Considerations

| Operation | Target | Strategy |
|-----------|--------|----------|
| Single character insert | <1 ms | Rope insert + run normalization |
| Paste 10 KB text | <5 ms | Batch rope insert, defer normalization |
| Select all (500 pages) | <10 ms | Iterator over runs, no text copy |
| Copy selection | <5 ms | Serialize only selected subtree |
| Cursor movement | <1 ms | Layout line map lookup (cached) |

The text engine itself is not the bottleneck. Layout invalidation after edits is the critical path — see [layout-engine.md](layout-engine.md) and [ffi-bridge.md](ffi-bridge.md).
