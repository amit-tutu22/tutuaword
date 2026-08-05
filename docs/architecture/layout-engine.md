# Layout Engine

The layout engine (`tw-layout`) converts the document model into positioned layout boxes ready for rendering. It is the hardest subsystem in the editor — responsible for text shaping, line breaking, pagination, float placement, and table layout.

## Pipeline Overview

```
Document Model
      │
      ▼
  Block Iterator (sections → blocks → paragraphs/tables/images)
      │
      ▼
  Text Shaping (tw-shape: rustybuzz + swash)
      │
      ▼
  Line Breaking (unicode-linebreak / UAX #14)
      │
      ▼
  Line Assembly (fit shaped glyphs into lines)
      │
      ▼
  Float Placement (images, shapes, text wrapping)
      │
      ▼
  Pagination (page breaks, headers, footers, columns)
      │
      ▼
  PageLayout (positioned boxes per page)
```

## Layout Engine State

```rust
pub struct LayoutEngine {
    doc: DocumentRef,
    font_db: FontDatabase,
    page_cache: HashMap<PageIndex, PageLayout>,
    dirty_pages: HashSet<PageIndex>,
    line_maps: HashMap<PageIndex, LineMap>,
}

impl LayoutEngine {
    pub fn layout_page(&mut self, page_index: PageIndex) -> &PageLayout;
    pub fn invalidate(&mut self, node_id: NodeId);
    pub fn invalidate_page(&mut self, page_index: PageIndex);
    pub fn hit_test(&self, page: PageIndex, x: f32, y: f32) -> HitTestResult;
    pub fn line_map(&self, page: PageIndex) -> &LineMap;
}
```

## Text Shaping

Before line breaking, all text must be shaped — converting character sequences into positioned glyphs with correct advances, offsets, and font selections.

### Shaping Pipeline

```
Run text (UTF-8)
      │
      ▼
  Script detection (unicode-script)
      │
      ▼
  Font selection (fontdb + fallback chain)
      │
      ▼
  rustybuzz::shape(text, font, features)
      │
      ▼
  ShapedRun { glyphs, advances, offsets }
      │
      ▼
  GlyphAtlas::rasterize(glyphs)  (for rendering)
```

### Font Selection and Fallback

```rust
pub struct FontDatabase {
    faces: HashMap<FontId, FontFace>,
    fallback_chains: HashMap<Script, Vec<FontId>>,
}

impl FontDatabase {
    pub fn resolve(&self, char_format: &ResolvedCharFormat, ch: char) -> FontId;
    pub fn fallback_chain(&self, script: Script) -> &[FontId];
}
```

Fallback chain example for a document using "Calibri":
1. Calibri (requested font)
2. Arial (same script, similar metrics)
3. Noto Sans (universal fallback)
4. Noto Sans Symbols 2 (symbols, emoji)
5. Last resort: tofu glyph (□)

Font enumeration uses `fontdb` to scan system fonts at startup. Embedded document fonts (from DOCX `fontTable.xml`) are loaded into the database on document open.

### OpenType Features

| Feature | Tag | Usage |
|---------|-----|-------|
| Kerning | `kern` | Default on |
| Ligatures | `liga`, `clig` | Default on |
| Small caps | `smcp` | When `all_caps` or `small_caps` formatting |
| Superscript | `sups` | When superscript formatting |
| Subscript | `subs` | When subscript formatting |
| Tabular nums | `tnum` | Table cells, numbered lists |

### Complex Scripts

| Script | Shaping Requirements |
|--------|---------------------|
| Latin/Cyrillic/Greek | Standard left-to-right |
| Arabic | RTL, contextual forms, ligatures (lam-alef) |
| Hebrew | RTL, mark positioning |
| Devanagari/Tamil/Telugu/Kannada | Matra reordering, conjuncts |
| Thai | No spaces — dictionary-based word breaking |
| CJK | No spaces — dictionary or ML-based word breaking |
| Emoji | Multi-codepoint sequences, color glyphs |

## Line Breaking

Line breaking follows Unicode Standard Annex #14 (Line Breaking Rules).

### Algorithm

```
For each paragraph:
  1. Collect all shaped runs into a linear glyph stream
  2. Apply UAX #9 (bidirectional reordering) if mixed direction
  3. Find break opportunities using UAX #14 rules
  4. Apply paragraph-specific rules:
     - Hyphenation (if enabled, using hyphenation dictionary)
     - Tab stops (advance to next tab position)
     - Non-breaking spaces (U+00A0 — no break before or after)
     - Keep-together spans (no break within)
  5. Pack glyphs into lines using available width:
     - Available width = page width - margins - indent - float intrusion
     - Greedy first-fit for MVP; Knuth-Plass for Phase 2+
  6. Apply alignment (left, center, right, justify)
  7. Compute line metrics (ascent, descent, line height)
```

### Line Breaking Rules (UAX #14 Summary)

| Rule | Description | Example |
|------|-------------|---------|
| Direct break | Space, most punctuation | "word \| word" |
| Indirect break | Hyphens, soft hyphens | "hyphen-\|ation" |
| Prohibited break | Non-breaking space, zero-width joiner | "100\u{00A0}km" |
| Break after | Closing punctuation (in some locales) | "word)\| " |
| Break before | Opening punctuation (in some locales) | "\|(word" |

Implemented via the `unicode-linebreak` crate with locale-aware rule sets.

### Knuth-Plass (Phase 2+)

For justified text, the Knuth-Plass algorithm minimizes the total "badness" of all lines by considering break opportunities globally rather than greedily. This produces more even spacing in justified paragraphs.

```
Badness(line) = penalty(break_opportunity) + stretch_ratio³

Total badness = sum of badness across all lines in paragraph
Optimal breaks = argmin(Total badness)
```

Deferred to Phase 2 because greedy first-fit produces acceptable results for left/center/right alignment, which covers the majority of documents.

## Line Assembly

```rust
pub struct TextLine {
    pub y: f32,                    // baseline y-position on page
    pub x: f32,                    // x-origin (after indent)
    pub width: f32,                // total line width
    pub ascent: f32,
    pub descent: f32,
    pub line_height: f32,
    pub shaped_runs: Vec<ShapedLineRun>,
    pub alignment: Alignment,
    pub paragraph_id: NodeId,
    pub line_index_in_paragraph: u32,
}

pub struct ShapedLineRun {
    pub run_id: NodeId,
    pub char_range: Range<CharIndex>,
    pub glyphs: Vec<PositionedGlyph>,
    pub font_id: FontId,
}

pub struct PositionedGlyph {
    pub glyph_id: u32,
    pub x: f32,
    pub y: f32,
    pub atlas_key: AtlasKey,
}
```

## Pagination

Pagination determines where content flows across pages.

### Page Break Rules

Content breaks to a new page when:
1. **Explicit break** — `\page` break character or `page_break_before` paragraph format
2. **Section break** — new section with potentially different page size/margins
3. **Natural overflow** — content exceeds available page height
4. **Keep constraints** — `keep_together` (paragraph must not split), `keep_with_next` (paragraph must stay with next paragraph)
5. **Widow/orphan control** — minimum 2 lines at top/bottom of page

### Page Layout

```rust
pub struct PageLayout {
    pub page_index: PageIndex,
    pub width: f32,
    pub height: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub header: Option<HeaderFooterLayout>,
    pub footer: Option<HeaderFooterLayout>,
    pub content_area: Rect,
    pub boxes: Vec<LayoutBox>,
    pub page_number: u32,
}

pub enum LayoutBox {
    TextLine(TextLine),
    Image(ImageLayout),
    Table(TableLayout),
    Shape(ShapeLayout),
    Spacer { height: f32 },
}
```

### Column Layout

Multi-column sections divide the content area:

```
┌──────────────────────────────┐
│  Column 1    │   Column 2    │
│  text text   │   text text   │
│  text text   │   text text   │
│  text text   │               │
│              │               │
│  (overflow→) │               │
└──────────────────────────────┘
```

Content fills column 1 top-to-bottom, then column 2, then overflows to the next page.

## Float Placement

Images and shapes can float alongside text.

### Text Wrap Modes

| Mode | Behavior |
|------|----------|
| Inline | Image on its own line, no text beside it |
| Square | Text wraps around the bounding box |
| Tight | Text wraps around the contour (Phase 2+) |
| Through | Text flows through transparent areas (Phase 2+) |
| Top and bottom | Text above and below, none beside |
| Behind text | Image rendered behind text |
| In front of text | Image rendered over text |

### Float Algorithm (MVP — Square Wrap)

```
1. Place float at its anchor position (vertical: after preceding line)
2. Compute bounding box (including margins)
3. Reduce available line width on affected lines:
   - Lines that vertically overlap the float lose width on the float's side
4. Re-break affected lines with reduced width
5. Text flows around the float's bounding box
```

## Table Layout

Tables are the most complex layout element.

### Table Layout Algorithm

```
1. Compute column widths:
   a. Fixed-width columns: use specified width
   b. Auto-width columns: measure content width (max of all cells in column)
   c. Percentage columns: fraction of available width
   d. Distribute remaining space proportionally
2. For each row:
   a. Compute row height: max of all cell content heights
   b. Handle merged cells (rowspan/colspan)
   c. Layout cell content (recursive — cells contain blocks)
3. Apply table borders and cell padding
4. Handle nested tables (recursive layout)
```

```rust
pub struct TableLayout {
    pub table_id: NodeId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rows: Vec<RowLayout>,
    pub column_widths: Vec<f32>,
    pub borders: BorderSet,
}

pub struct RowLayout {
    pub height: f32,
    pub cells: Vec<CellLayout>,
}

pub struct CellLayout {
    pub x: f32,
    pub width: f32,
    pub content: Vec<LayoutBox>,
    pub rowspan: u32,
    pub colspan: u32,
    pub borders: BorderSet,
    pub padding: Padding,
}
```

### Cell Content

Table cells contain `Vec<Block>` — the same block types as section content (paragraphs, nested tables, images). Cell layout is recursive: the layout engine calls itself for each cell's content area (cell width minus padding, cell height unbounded for auto-height).

## Headers and Footers

```rust
pub enum HeaderFooterType {
    Default,
    First,
    Even,
    Odd,
}

pub struct HeaderFooterLayout {
    pub hf_type: HeaderFooterType,
    pub height: f32,
    pub content: Vec<LayoutBox>,
}
```

Header/footer content is laid out in the margin area above/below the content area. Different header/footer variants apply based on page number (first page, even/odd pages).

## Footnotes and Endnotes

### Footnotes (Phase 2)

```
Page content area
├── Body text
├── Body text (continued)
├────────────────────── separator line
└── Footnote area
    ├── Footnote 1 text
    └── Footnote 2 text
```

Footnotes reduce the available body text area on the page where they appear. If footnotes exceed a configurable fraction of the page, they overflow to the next page.

### Endnotes (Phase 3)

Endnotes are collected at the end of the section or document, laid out as a continuous block.

## Hit Testing

The layout engine provides hit testing for cursor placement and selection:

```rust
pub struct HitTestResult {
    pub page: PageIndex,
    pub position: DocPosition,
    pub affinity: CursorAffinity,
    pub target: HitTarget,
}

pub enum HitTarget {
    Text,
    Image { image_id: NodeId },
    Table { table_id: NodeId, cell: Option<(row, col)> },
    Shape { shape_id: NodeId },
    Margin,
}
```

Hit testing uses the `LineMap` — a cached mapping from (page, x, y) to (run_id, char_offset):

```rust
pub struct LineMap {
    pub lines: Vec<LineEntry>,
}

pub struct LineEntry {
    pub y_range: Range<f32>,
    pub x_range: Range<f32>,
    pub run_map: Vec<(Range<f32>, NodeId, CharIndex)>,
}
```

## Invalidation Strategy

Layout is expensive. Only re-layout what changed.

```
Edit operation
  → tw-edit returns affected_nodes
  → layout engine finds pages containing affected_nodes
  → mark those pages dirty
  → if paragraph grew/shrank, may need to re-layout subsequent pages (chain invalidation)
  → re-layout dirty pages in background thread
  → publish new display list snapshots
```

### Chain Invalidation

When a paragraph grows (text inserted), it may push subsequent content to the next page. The layout engine re-layouts forward from the dirty page until page breaks stabilize (content no longer overflows or underflows).

Worst case: inserting text at page 1 of a 500-page document could theoretically require re-layouting all 500 pages. In practice, page break changes propagate only a few pages forward. Benchmark and optimize if this becomes a problem.

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Layout single page (pure text) | <5 ms | Typical page: ~40 lines, ~2000 chars |
| Layout single page (with tables/images) | <15 ms | |
| Full document layout (500 pages) | <2 s | Parallel page layout via rayon |
| Hit test | <0.1 ms | LineMap lookup |
| Invalidate + re-layout after keystroke | <8 ms | Single page, no chain |
| Chain invalidation (5 pages) | <40 ms | Background thread, UI not blocked |

See [performance-budgets.md](../performance-budgets.md) for measurement methodology.
