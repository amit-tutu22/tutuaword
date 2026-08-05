# File Formats

This document specifies the file format support matrix, the native on-disk format, and the common import/export architecture shared by all format crates.

## Format Matrix

| Format | Extension | Import | Export | Phase | Crate |
|--------|-----------|--------|--------|-------|-------|
| Native | `.twdoc` | Yes | Yes | 1 | `tw-native` |
| Microsoft Word | `.docx` | Yes | Yes | 3 | `tw-docx` |
| OpenDocument | `.odt` | Yes | Yes | 3 | `tw-odt` |
| Markdown | `.md` | Yes | Yes | 3 | `tw-markdown` |
| HTML | `.html`, `.htm` | Yes | Yes | 3 | `tw-html` |
| Rich Text | `.rtf` | Yes | — | 3 | `tw-rtf` |
| PDF | `.pdf` | — | Yes | 2 | `tw-pdf` |
| EPUB | `.epub` | — | Future | — | — |
| Plain Text | `.txt` | Yes | Yes | 1 | Built into `tw-native` |
| Legacy Word | `.doc` | — | — | — | Out of scope |

## Format Handler Trait

All format crates implement a common trait:

```rust
pub trait FormatHandler {
    fn import(&self, source: &[u8]) -> Result<ImportResult, ImportError>;
    fn export(&self, doc: &Document, options: &ExportOptions) -> Result<Vec<u8>, ExportError>;
    fn extensions(&self) -> &[&str];
    fn mime_types(&self) -> &[&str];
}

pub struct ImportResult {
    pub document: Document,
    pub warnings: Vec<ImportWarning>,
    pub source_format: String,
    pub passthrough: Option<PassthroughPackage>,
}

pub struct ExportOptions {
    pub target_format: String,
    pub compatibility_mode: CompatibilityMode,
    pub embed_fonts: bool,
    pub image_quality: ImageQuality,
    pub passthrough: Option<PassthroughPackage>,
}
```

The `passthrough` field is critical for DOCX fidelity — see [docx-compatibility.md](docx-compatibility.md).

## Native Format (`.twdoc`)

The native format is the primary save format during editing. It is optimized for fidelity, diffability, and fast load/save — not for interchange with other applications.

### Design Goals

1. **Lossless** — every field in the document model is preserved
2. **Diffable** — JSON content diffs well in Git
3. **Human-readable** — developers can inspect and debug documents
4. **Fast** — load/save a 500-page document in <500 ms
5. **Extensible** — new fields can be added without breaking old readers

### File Structure

A `.twdoc` file is a ZIP archive:

```
document.twdoc (ZIP)
├── manifest.json          format version, creation date, app version
├── content.json           full document model (sections, blocks, runs)
├── styles.json            style sheet
├── settings.json          document settings
├── comments.json          comments (if any)
├── revisions.json         track-changes data (if any)
├── media/
│   ├── {uuid}.png         embedded images
│   ├── {uuid}.jpg
│   └── {uuid}.emf         metafiles
└── fonts/
    └── {uuid}.ttf         embedded fonts (optional)
```

### manifest.json

```json
{
  "format_version": "1.0",
  "created": "2026-08-04T22:00:00Z",
  "modified": "2026-08-04T22:30:00Z",
  "app_version": "0.1.0",
  "generator": "tutuaword"
}
```

### content.json

The full document tree serialized as JSON. Key conventions:

- `NodeId` values serialize as UUID strings: `"a1b2c3d4-e5f6-7890-abcd-ef1234567890"`
- Format structs serialize sparsely — only `Some` fields are included
- Text content is inline in runs (not in a separate buffer)
- Section/page structure is explicit (not computed)

Example fragment:

```json
{
  "sections": [{
    "id": "sec-001",
    "format": {
      "page_width": 612.0,
      "page_height": 792.0,
      "margin_top": 72.0,
      "margin_bottom": 72.0,
      "margin_left": 72.0,
      "margin_right": 72.0
    },
    "blocks": [{
      "type": "paragraph",
      "id": "para-001",
      "style_id": "Heading1",
      "runs": [{
        "id": "run-001",
        "format": { "bold": true, "font_size": 16.0 },
        "content": { "type": "text", "value": "Chapter 1" }
      }]
    }]
  }]
}
```

### Versioning

The `format_version` field follows semver:
- **Major** — breaking changes to the schema (migration required)
- **Minor** — new optional fields (old readers ignore them)
- **Patch** — bug fixes in serialization

Migration functions handle major version upgrades:

```rust
pub fn migrate(content: &serde_json::Value, from_version: &str) -> Result<Document, MigrationError>;
```

## Plain Text (`.txt`)

Plain text import/export is built into `tw-native` (no separate crate):

- **Import:** Each line becomes a paragraph. Empty lines become empty paragraphs. No formatting.
- **Export:** Each paragraph becomes a line. Runs concatenated without formatting. Line breaks within paragraphs become spaces.

## PDF Export (Phase 2)

PDF export uses the layout engine's output directly — no re-layout needed.

### Pipeline

```
Document Model
      │
      ▼
  Layout Engine (all pages)
      │
      ▼
  Display List (per page)
      │
      ▼
  PDF Writer (printpdf or PDFium)
  ├── Embed fonts (subset)
  ├── Draw text from atlas glyph positions
  ├── Draw rects, paths, images
  └── Write PDF structure (bookmarks, metadata, hyperlinks)
```

Library choice (decide in Phase 2):
- **printpdf** — pure Rust, good for basic PDFs, limited font support
- **PDFium** — C library, full PDF spec support, requires native build

Recommendation: start with `printpdf` for MVP PDF export; evaluate PDFium if fidelity requirements exceed printpdf's capabilities.

## Markdown (Phase 3)

### Import Mapping

| Markdown | Document Model |
|----------|---------------|
| `# Heading` | Paragraph with Heading 1 style |
| `**bold**` | Run with bold formatting |
| `*italic*` | Run with italic formatting |
| `- item` | Paragraph with bullet list numbering |
| `1. item` | Paragraph with numbered list numbering |
| `[text](url)` | Run with hyperlink |
| `![alt](url)` | ImageBlock |
| `\| cell \|` | Table |
| `---` | Section break or horizontal rule |

### Export Mapping

Reverse of import. Unsupported features (page breaks, headers/footers, complex tables) are exported as HTML comments or omitted with a warning.

## HTML (Phase 3)

### Import

Parse HTML DOM into document model:
- `<p>`, `<h1>`–`<h6>` → Paragraph with appropriate style
- `<strong>`, `<b>`, `<em>`, `<i>`, `<u>` → Run formatting
- `<table>`, `<tr>`, `<td>` → Table structure
- `<img>` → ImageBlock
- CSS styles → CharFormat/ParaFormat (subset)

Uses the `html5ever` + `markup5ever_rcdom` crates for parsing.

### Export

Generate semantic HTML with inline CSS. Embedded images as base64 data URIs or relative file references.

## ODT (Phase 3)

OpenDocument Text format. Similar structure to DOCX (ZIP + XML) but using ODF schemas.

- Import: parse `content.xml`, `styles.xml`, `meta.xml` into document model
- Export: generate ODF XML from document model
- Lower priority than DOCX — implement after DOCX compatibility is solid

## RTF (Phase 3)

Rich Text Format — import only (legacy interchange).

- Parse RTF control words into document model
- No export (RTF is a legacy format; export to DOCX instead)
- Uses a custom RTF parser (no mature Rust RTF library exists)

## Format Detection

```rust
pub fn detect_format(source: &[u8], filename: Option<&str>) -> Format {
    // 1. Check file extension if filename provided
    // 2. Check magic bytes:
    //    PK\x03\x04 → ZIP (DOCX, ODT, TWDOC)
    //    {\rtf → RTF
    //    %PDF → PDF (export-only, reject on import)
    //    <!DOCTYPE / <html → HTML
    //    # / ## / - → Markdown (heuristic)
    // 3. For ZIP: inspect internal structure
    //    [Content_Types].xml + word/document.xml → DOCX
    //    content.xml + META-INF/manifest.xml → ODT
    //    manifest.json + content.json → TWDOC
}
```

## Error Handling

```rust
pub enum ImportError {
    UnsupportedFormat { detected: String },
    CorruptedFile { detail: String },
    PasswordProtected,
    ParseError { part: String, line: u32, detail: String },
    OutOfMemory,
}

pub enum ExportError {
    UnsupportedFeature { feature: String },
    FontEmbeddingFailed { font: String },
    ImageEncodingFailed { image_id: NodeId },
    IoError { detail: String },
}
```

Import warnings (non-fatal) are collected and shown to the user:

```rust
pub enum ImportWarning {
    UnsupportedElement { element: String, preserved: bool },
    FontSubstituted { requested: String, fallback: String },
    ImageFormatUnsupported { format: String },
    StyleNotFound { style_id: String },
    DataTruncated { reason: String },
}
```
