# Document Model

> **Implementation status:** The tree below includes types not yet built in code (`Comment`, `Bookmark`, `RunContent::Field`, `Block::ShapeBlock`, typed header/footer maps). See [Long-Tail Gaps](../long-tail-gaps.md) §4 for current vs planned model.

The document model (`tw-model`) is the single source of truth for all document content and structure. Every other subsystem reads from it; only `tw-edit` mutates it.

## Design Principles

1. **Everything is a tree.** The document is a rooted tree of typed nodes. There are no parallel data structures for content.
2. **Stable IDs.** Every node has a UUID that never changes, even across undo/redo, copy/paste, and CRDT merges. Layout, selection, and comments reference nodes by ID.
3. **Formatting is layered.** Character formatting applies to runs, paragraph formatting to paragraphs, section formatting to sections. Document-level defaults exist but are overridden by styles.
4. **Runs are the atomic text unit.** A paragraph contains one or more runs. Each run has uniform character formatting. When formatting changes mid-paragraph, the run splits.
5. **Revisions are metadata, not copies.** Track-changes information is attached to nodes as metadata, not stored as parallel document versions.

## Node Tree

```
Document
├── metadata: DocumentMetadata
├── styles: StyleSheet
├── settings: DocumentSettings
└── sections: Vec<Section>
    └── Section
        ├── id: NodeId
        ├── format: SectionFormat
        ├── headers: HashMap<HeaderFooterType, HeaderFooter>
        ├── footers: HashMap<HeaderFooterType, HeaderFooter>
        └── blocks: Vec<Block>
            ├── Paragraph
            │   ├── id: NodeId
            │   ├── format: ParaFormat
            │   ├── style_id: Option<StyleId>
            │   ├── revision: Option<Revision>
            │   └── runs: Vec<Run>
            │       └── Run
            │           ├── id: NodeId
            │           ├── format: CharFormat
            │           ├── revision: Option<Revision>
            │           └── content: RunContent
            │               ├── Text(String)
            │               ├── Tab
            │               ├── Break(BreakType)
            │               ├── Image(ImageRef)
            │               ├── Field(FieldType)
            │               └── Symbol { font, char_code }
            ├── Table
            │   ├── id: NodeId
            │   ├── format: TableFormat
            │   └── rows: Vec<TableRow>
            │       └── TableRow
            │           ├── id: NodeId
            │           ├── format: RowFormat
            │           └── cells: Vec<TableCell>
            │               └── TableCell
            │                   ├── id: NodeId
            │                   ├── format: CellFormat
            │                   └── blocks: Vec<Block>  (recursive)
            ├── ImageBlock
            │   ├── id: NodeId
            │   ├── image: ImageData
            │   └── wrap: TextWrap
            └── ShapeBlock
                ├── id: NodeId
                ├── shape: ShapeData
                └── wrap: TextWrap
```

## Node IDs

```rust
pub struct NodeId(Uuid);
```

- Generated with `Uuid::new_v4()` at node creation time
- Never reassigned, even when a node is moved, copied, or restored by undo
- Used by: layout cache, selection, comments, bookmarks, CRDT operations, revision tracking
- Serialized in the native format and embedded in DOCX as custom XML (`tw:nodeId` attribute)

### ID Stability Across Operations

| Operation | ID Behavior |
|-----------|-------------|
| Insert paragraph | New UUID for paragraph and its initial run |
| Delete paragraph | ID removed from tree; undo restores with same ID |
| Split run (format change) | Original run keeps ID; new run gets new UUID |
| Merge runs (same format) | Surviving run keeps its ID; merged run's ID is discarded |
| Copy/paste | Pasted nodes get new UUIDs |
| Undo/redo | Restored nodes retain their original IDs |
| CRDT merge | Remote nodes get their CRDT-assigned IDs mapped to NodeIds |

## Formatting Types

### CharFormat

```rust
pub struct CharFormat {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,         // points
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<UnderlineStyle>,
    pub strikethrough: Option<bool>,
    pub superscript: Option<bool>,
    pub subscript: Option<bool>,
    pub color: Option<Color>,
    pub highlight: Option<Color>,
    pub language: Option<String>,       // BCP 47
    pub spacing: Option<f32>,           // character spacing in points
    pub scale: Option<f32>,             // horizontal scale percentage
    pub position: Option<f32>,          // baseline offset in points
    pub hidden: Option<bool>,
    pub all_caps: Option<bool>,
    pub small_caps: Option<bool>,
}
```

All fields are `Option<T>`. `None` means "inherit from paragraph style or document default." This matches OOXML's `<w:rPr>` semantics.

### ParaFormat

```rust
pub struct ParaFormat {
    pub alignment: Option<Alignment>,
    pub line_spacing: Option<LineSpacing>,
    pub space_before: Option<f32>,
    pub space_after: Option<f32>,
    pub indent_left: Option<f32>,
    pub indent_right: Option<f32>,
    pub indent_first_line: Option<f32>,
    pub indent_hanging: Option<f32>,
    pub keep_together: Option<bool>,
    pub keep_with_next: Option<bool>,
    pub page_break_before: Option<bool>,
    pub widow_orphan_control: Option<bool>,
    pub borders: Option<BorderSet>,
    pub shading: Option<Color>,
    pub tab_stops: Option<Vec<TabStop>>,
    pub numbering: Option<NumberingRef>,
    pub outline_level: Option<u8>,
}
```

### SectionFormat

```rust
pub struct SectionFormat {
    pub page_width: f32,
    pub page_height: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub margin_header: f32,
    pub margin_footer: f32,
    pub gutter: f32,
    pub columns: ColumnLayout,
    pub page_number_start: Option<u32>,
    pub text_direction: TextDirection,
    pub paper_source: Option<PaperSource>,
}
```

## Style System

Styles are named, reusable format templates with inheritance.

```rust
pub struct StyleSheet {
    pub defaults: DocumentDefaults,
    pub character_styles: HashMap<StyleId, CharacterStyle>,
    pub paragraph_styles: HashMap<StyleId, ParagraphStyle>,
    pub table_styles: HashMap<StyleId, TableStyle>,
}

pub struct ParagraphStyle {
    pub id: StyleId,
    pub name: String,
    pub based_on: Option<StyleId>,
    pub next_style: Option<StyleId>,
    pub para_format: ParaFormat,
    pub char_format: CharFormat,
    pub is_quick_style: bool,
    pub priority: i32,
}
```

### Style Resolution

When rendering a run, effective formatting is computed by merging layers (lowest priority first, highest wins):

1. Document defaults (`docDefaults` in OOXML)
2. Paragraph style (walk `based_on` chain to root)
3. Character style (if applied)
4. Direct formatting on the run (`CharFormat` with `Some` fields)
5. Revision formatting (if track changes is active)

```rust
pub fn resolve_char_format(
    doc: &Document,
    run: &Run,
    paragraph: &Paragraph,
) -> ResolvedCharFormat {
    let mut resolved = doc.styles.defaults.char_format.clone();
    if let Some(style_id) = &paragraph.style_id {
        resolved.merge(&doc.styles.resolve_paragraph_style(style_id).char_format);
    }
    resolved.merge(&run.format);
    resolved
}
```

## Revision Tracking

Track-changes metadata is attached to nodes, not stored as parallel versions.

```rust
pub struct Revision {
    pub revision_type: RevisionType,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub id: i64,
}

pub enum RevisionType {
    Insert,
    Delete,
    FormatChange { old_format: Box<dyn Format> },
    MoveFrom,
    MoveTo,
}
```

When track changes is enabled:
- Inserted text gets `RevisionType::Insert`
- Deleted text is not removed; it gets `RevisionType::Delete` and is hidden in "final" view
- Format changes record the old format in `RevisionType::FormatChange`

## Bookmarks and Cross-References

```rust
pub struct Bookmark {
    pub id: NodeId,
    pub name: String,
    pub start_run: NodeId,
    pub start_offset: usize,
    pub end_run: NodeId,
    pub end_offset: usize,
}

pub struct CrossReference {
    pub field_type: FieldType,
    pub bookmark_name: Option<String>,
    pub display_text: Option<String>,
}
```

## Comments

```rust
pub struct Comment {
    pub id: NodeId,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub anchor_run: NodeId,
    pub anchor_offset: usize,
    pub anchor_end_run: Option<NodeId>,
    pub anchor_end_offset: Option<usize>,
    pub body: Vec<Block>,  // comment content is a mini-document
    pub replies: Vec<Comment>,
    pub resolved: bool,
}
```

## Document Settings

```rust
pub struct DocumentSettings {
    pub track_changes_enabled: bool,
    pub default_tab_stop: f32,
    pub compatibility_mode: CompatibilityMode,
    pub zoom_level: f32,
    pub view_mode: ViewMode,
    pub measurement_unit: MeasurementUnit,
    pub hyphenation: bool,
    pub even_odd_headers: bool,
}
```

## Serialization

The native format (`.twdoc`) serializes the entire model as JSON with binary assets (images) in a ZIP container. See [file-formats.md](file-formats.md).

Key serialization rules:
- `NodeId` serializes as UUID string
- `CharFormat`/`ParaFormat` serialize only `Some` fields (sparse)
- Text content in runs is stored inline (not in the rope; the rope is a runtime optimization in `tw-text`)
- Images stored as `{uuid}.{ext}` in the ZIP `media/` directory, referenced by `ImageData.id`

## Invariants

These invariants are enforced by `tw-edit` and tested:

1. Every `Run` belongs to exactly one `Paragraph`
2. Every `Paragraph`/`Table`/`ImageBlock` belongs to exactly one `Section` (or `TableCell`)
3. A `Paragraph` has at least one `Run` (even if empty text)
4. Adjacent runs with identical `CharFormat` are merged (normalization pass after every edit)
5. `NodeId` values are unique across the entire document
6. Section breaks appear only as the last block in a section (except the final section)
