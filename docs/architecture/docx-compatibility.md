# DOCX Compatibility

Microsoft Word compatibility is the highest-priority feature. This document specifies the strategy for achieving high-fidelity DOCX import, edit, and export.

## Fidelity Tiers

Not every OOXML element can be fully parsed into our model and regenerated identically. We classify compatibility into three tiers:

| Tier | Name | Import | Render | Edit | Export | Example |
|------|------|--------|--------|------|--------|---------|
| **A** | Full | Parse to model | Correct | Full | Lossless round-trip | Paragraphs, runs, bold/italic, tables, images |
| **B** | Render + Preserve | Parse to model | Correct | Partial | Preserved via passthrough | Styles, themes, numbering, headers/footers |
| **C** | Preserve Only | Store verbatim | Placeholder or skip | None | Preserved verbatim | Macros, ActiveX, ink, custom XML |

**Goal:** Per-category Word visual pass rates plus lossless Tier A round-trip and Tier B/C passthrough survival — see the fidelity SLA in [risk-mitigation.md](../risk-mitigation.md). Avoid a single “95%/99%” slogan until Word baselines and corpus gates exist.

## Package Passthrough Strategy

The key insight: **do not regenerate the entire DOCX from scratch.** Instead, retain the original OPC package and patch only the parts we modified.

### Why Passthrough

Parsing OOXML into our model and regenerating it loses:
- Unknown XML elements and attributes
- Namespace declarations we don't understand
- Custom XML parts
- Embedded objects (OLE, ActiveX)
- Proprietary Microsoft extensions
- Exact whitespace and element ordering in XML
- Relationship IDs and content type declarations

Word itself uses passthrough — it preserves unknown elements when saving. We must do the same.

### Architecture

```rust
pub struct DocxPackage {
    pub content_types: ContentTypes,
    pub relationships: Relationships,
    pub parts: HashMap<PartName, PartData>,
    pub modified_parts: HashSet<PartName>,
    pub original_bytes: Option<Vec<u8>>,
}

pub enum PartData {
    Parsed(ParsedPart),     // we understand this part
    Raw(Vec<u8>),             // preserved verbatim
}

pub enum ParsedPart {
    Document(DocumentPart),
    Styles(StylesPart),
    Numbering(NumberingPart),
    Theme(ThemePart),
    Settings(SettingsPart),
    Comments(CommentsPart),
    HeaderFooter(HeaderFooterPart),
    Footnotes(FootnotesPart),
    Endnotes(EndnotesPart),
    FontTable(FontTablePart),
}
```

### Import Flow

```
1. Open .docx as ZIP archive
2. Parse [Content_Types].xml → part inventory
3. Parse _rels/.rels → root relationships
4. For each part:
   a. If we have a parser → parse into model + store raw bytes
   b. If no parser → store raw bytes only (Tier C)
5. Build Document model from parsed parts
6. Store DocxPackage alongside Document for export
```

### Export Flow

```
1. Start with original DocxPackage (or create new if Save As)
2. For each modified part:
   a. Serialize model changes back to OOXML
   b. Replace part in package
3. For unmodified parts:
   a. Keep original bytes unchanged
4. Update [Content_Types].xml and relationships if parts added/removed
5. Write ZIP archive
```

### Node ID Embedding

To map between our model and OOXML elements, we embed `tw:nodeId` attributes in the XML:

```xml
<w:p w14:paraId="A1B2C3D4">
  <w:r>
    <w:t>Hello</w:t>
  </w:r>
</w:p>
```

We use Word's existing `w14:paraId` where available, and add our own `tw:nodeId` attribute for elements Word doesn't ID.

## Part-by-Part Coverage Matrix

### Core Document Parts

| OOXML Part | Path | Tier | Phase | Notes |
|------------|------|------|-------|-------|
| Document body | `word/document.xml` | A | 3 | Paragraphs, runs, tables, images, breaks |
| Styles | `word/styles.xml` | B | 3 | Parse to StyleSheet; preserve raw for unknown styles |
| Theme | `word/theme/theme1.xml` | B | 3 | Color/font theme; preserve raw |
| Numbering | `word/numbering.xml` | B | 3 | List definitions; parse common patterns |
| Settings | `word/settings.xml` | B | 3 | Document settings; preserve raw |
| Font table | `word/fontTable.xml` | B | 3 | Font declarations; parse + preserve |
| Relationships | `word/_rels/document.xml.rels` | A | 3 | Image/header/footer links |
| Content types | `[Content_Types].xml` | A | 3 | Part registry |

### Headers and Footers

| OOXML Part | Path | Tier | Phase | Notes |
|------------|------|------|-------|-------|
| Header 1..N | `word/header1.xml` | B | 2 | Parse content; preserve raw |
| Footer 1..N | `word/footer1.xml` | B | 2 | Parse content; preserve raw |

### Media and Embeddings

| OOXML Part | Path | Tier | Phase | Notes |
|------------|------|------|-------|-------|
| Images (PNG) | `word/media/imageN.png` | A | 2 | Direct binary passthrough |
| Images (JPEG) | `word/media/imageN.jpeg` | A | 2 | Direct binary passthrough |
| Images (EMF/WMF) | `word/media/imageN.emf` | B | 2 | Store; convert to PNG for rendering |
| OLE objects | `word/embeddings/oleObjectN.bin` | C | — | Preserve verbatim |
| VML shapes | `word/media/*.vml` | C | — | Preserve verbatim |

### Annotations

| OOXML Part | Path | Tier | Phase | Notes |
|------------|------|------|-------|-------|
| Comments | `word/comments.xml` | B | 5 | Parse for display; preserve raw |
| Comment authors | `word/commentsExtended.xml` | C | 5 | Preserve verbatim |
| Footnotes | `word/footnotes.xml` | B | 2 | Parse content; preserve raw |
| Endnotes | `word/endnotes.xml` | B | 3 | Parse content; preserve raw |

### Drawing and Graphics

| OOXML Part | Path | Tier | Phase | Notes |
|------------|------|------|-------|-------|
| DrawingML shapes | inline in `document.xml` (`w:drawing`) | B | 2 | Parse basic shapes; preserve complex |
| Chart data | `word/charts/chartN.xml` | C | — | Preserve verbatim; placeholder render |
| SmartArt | `word/diagrams/dataN.xml` | C | — | Preserve verbatim |
| Ink annotations | `word/inkN.xml` | C | — | Preserve verbatim |

### Metadata and Custom

| OOXML Part | Path | Tier | Phase | Notes |
|------------|------|------|-------|-------|
| Core properties | `docProps/core.xml` | A | 3 | Title, author, dates |
| App properties | `docProps/app.xml` | A | 3 | Page count, word count |
| Custom properties | `docProps/custom.xml` | C | — | Preserve verbatim |
| Custom XML | `customXml/*` | C | — | Preserve verbatim |
| VBA macros | `word/vbaProject.bin` | C | — | Preserve verbatim; never execute |

## Element-Level Mapping

### Paragraphs (`w:p`)

| OOXML Element | Model Mapping | Tier |
|---------------|---------------|------|
| `w:pPr/w:pStyle` | `Paragraph.style_id` | A |
| `w:pPr/w:jc` | `ParaFormat.alignment` | A |
| `w:pPr/w:spacing` | `ParaFormat.line_spacing`, `space_before`, `space_after` | A |
| `w:pPr/w:ind` | `ParaFormat.indent_*` | A |
| `w:pPr/w:numPr` | `ParaFormat.numbering` | B |
| `w:pPr/w:pBdr` | `ParaFormat.borders` | A |
| `w:pPr/w:shd` | `ParaFormat.shading` | A |
| `w:pPr/w:pageBreakBefore` | `ParaFormat.page_break_before` | A |
| `w:pPr/w:keepNext` | `ParaFormat.keep_with_next` | A |
| `w:pPr/w:keepLines` | `ParaFormat.keep_together` | A |
| `w:pPr/w:widowControl` | `ParaFormat.widow_orphan_control` | A |
| `w:pPr/w:tabs` | `ParaFormat.tab_stops` | B |
| `w:pPr/w:outlineLvl` | `ParaFormat.outline_level` | A |
| `w:pPr/w:rPr` (inline style) | Applied to all runs in paragraph | A |

### Runs (`w:r`)

| OOXML Element | Model Mapping | Tier |
|---------------|---------------|------|
| `w:t` | `RunContent::Text` | A |
| `w:tab` | `RunContent::Tab` | A |
| `w:br` | `RunContent::Break` | A |
| `w:drawing` | `RunContent::Image` or `ShapeBlock` | B |
| `w:rPr/w:b` | `CharFormat.bold` | A |
| `w:rPr/w:i` | `CharFormat.italic` | A |
| `w:rPr/w:u` | `CharFormat.underline` | A |
| `w:rPr/w:strike` | `CharFormat.strikethrough` | A |
| `w:rPr/w:vertAlign` | `CharFormat.superscript/subscript` | A |
| `w:rPr/w:sz` | `CharFormat.font_size` | A |
| `w:rPr/w:rFonts` | `CharFormat.font_family` | A |
| `w:rPr/w:color` | `CharFormat.color` | A |
| `w:rPr/w:highlight` | `CharFormat.highlight` | A |
| `w:rPr/w:lang` | `CharFormat.language` | A |
| `w:rPr/w:spacing` | `CharFormat.spacing` | B |
| `w:rPr/w:w` (scale) | `CharFormat.scale` | B |
| `w:rPr/w:position` | `CharFormat.position` | B |
| `w:rPr/w:rStyle` | Character style reference | B |
| `w:instrText` + `w:fldChar` | Field codes | B |

### Tables (`w:tbl`)

| OOXML Element | Model Mapping | Tier |
|---------------|---------------|------|
| `w:tblPr/w:tblStyle` | `Table.style_id` | B |
| `w:tblPr/w:tblW` | `TableFormat.width` | A |
| `w:tblGrid/w:gridCol` | Column widths | A |
| `w:tr/w:trPr/w:trHeight` | `RowFormat.height` | A |
| `w:tc/w:tcPr/w:gridSpan` | `CellFormat.colspan` | A |
| `w:tc/w:tcPr/w:vMerge` | `CellFormat.rowspan` | A |
| `w:tc/w:tcPr/w:tcBorders` | `CellFormat.borders` | A |
| `w:tc/w:tcPr/w:shd` | `CellFormat.shading` | A |
| `w:tc/w:tcPr/w:tcW` | `CellFormat.width` | A |
| `w:tc/w:tcPr/w:vAlign` | `CellFormat.vertical_align` | A |

## Test Corpus

A curated set of real-world DOCX files for automated compatibility testing:

### Corpus Categories

| Category | Count | Purpose |
|----------|-------|---------|
| Simple text | 10 | Paragraphs, runs, basic formatting |
| Styled documents | 15 | Style sheets, themes, templates |
| Tables | 15 | Simple, merged, nested, formatted |
| Images | 10 | Inline, wrapped, various formats |
| Lists | 10 | Bulleted, numbered, multi-level |
| Headers/footers | 5 | Default, first, even/odd |
| Complex layout | 10 | Columns, floats, text boxes |
| Track changes | 5 | Insertions, deletions, formatting |
| Comments | 5 | Comments with replies |
| Real-world samples | 15 | Legal, academic, business, government |
| Edge cases | 10 | Empty, single char, 500+ pages, corrupted |

### Test Procedures

**Import test:** Open each file, render all pages, compare against reference PNG (golden-image test). Pass if visual difference < 2% pixel tolerance.

**Round-trip test:** Import → save as DOCX → re-import → compare model. Tier A elements must be identical. Tier B elements must be preserved in raw bytes. Tier C elements must survive in the package.

**Performance test:** Open 500-page document in <2 seconds. Measure with `criterion` benchmarks.

## Compatibility Mode

When exporting, the user can select a compatibility mode:

| Mode | Behavior |
|------|----------|
| Strict OOXML | Only standard ECMA-376 elements |
| Word 2016+ | Include Microsoft extensions (`w14:`, `w15:` namespaces) |
| Word 2010 | Legacy compatibility for older Word versions |
| Max fidelity | Preserve everything from original (default for Save, not Save As) |

Default: **Max fidelity** when saving an imported DOCX. **Word 2016+** when saving as new DOCX.

## Known Limitations

These are accepted limitations, documented for user expectations:

1. **VBA macros** — preserved but never executed
2. **ActiveX controls** — preserved but not rendered or interactive
3. **SmartArt** — preserved; rendered as placeholder bounding box
4. **Embedded Excel charts** — preserved; rendered as static image if available
5. **Form fields** — preserved; interactive form filling is Phase 6+
6. **Legacy `.doc` format** — not supported; user must convert via Word or LibreOffice
7. **Password-protected files** — detected and rejected with clear error message (decryption is Phase 6)
