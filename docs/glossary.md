# Glossary

Terms and acronyms used throughout the architecture documentation.

## A

**ADR (Architecture Decision Record)**
A short document recording a significant architectural decision, its context, and consequences. Stored in `docs/adr/`.

**Atlas (Glyph Atlas)**
A shared texture containing pre-rasterized glyph bitmaps. Text rendering references glyphs by position in the atlas rather than rasterizing on each frame.

**Awareness (Yjs Awareness)**
Ephemeral protocol for sharing cursor positions, selections, and user presence in real-time collaboration. Not persisted in the document.

## B

**Block**
A top-level content element within a section: paragraph, table, image, or shape.

## C

**CharFormat**
Character-level formatting properties (bold, italic, font, size, color, etc.) applied to a run. See [document-model.md](architecture/document-model.md).

**Command**
A typed edit operation (InsertText, DeleteRange, SetCharFormat, etc.) that is the sole entry point for document mutations. See ADR-0007.

**CRDT (Conflict-free Replicated Data Type)**
A data structure that allows concurrent edits from multiple users to merge automatically without conflicts. Used for real-time collaboration (Yjs).

**Cursor Affinity**
When a cursor is at a line boundary, affinity determines whether it sticks to the end of the previous line (upstream) or the start of the next line (downstream).

## D

**Display List**
A flat, ordered sequence of draw commands (glyph batches, rects, paths, images) produced by the rendering engine. Consumed by Flutter's CustomPainter.

**DocPosition**
A document-wide position referencing a specific run and character offset within that run.

**DocRange**
A document-wide range with start and end DocPositions.

**DOCX**
Microsoft Word's Office Open XML format. A ZIP archive containing XML parts. Primary interchange format for Word compatibility.

**DocxPackage**
The internal representation of a DOCX file retaining all original OPC parts for passthrough on save.

## F

**FFI (Foreign Function Interface)**
The boundary between the Rust engine and the Flutter UI. Uses C ABI on native platforms and WASM bindings on web.

**Fidelity Tier**
Classification of DOCX compatibility: Tier A (full round-trip), Tier B (render + preserve), Tier C (preserve only).

## G

**Grapheme Cluster**
The smallest unit of text that a user perceives as a character. May consist of multiple Unicode codepoints (e.g., emoji with skin tone modifier, Hindi conjuncts).

## K

**Knuth-Plass**
An optimal line breaking algorithm that minimizes total "badness" across all lines in a paragraph. Used for justified text (Phase 2+).

## L

**LayoutBox**
A positioned element on a page: text line, image, table, shape, or spacer. Output of the layout engine.

**LineMap**
A cached mapping from (page, x, y) screen coordinates to (run_id, char_offset) document positions. Used for hit testing and cursor placement.

## N

**NodeId**
A stable UUID assigned to every node in the document tree at creation time. Never changes across undo/redo, copy/paste, or CRDT merges.

**Native Format (.twdoc)**
The editor's primary on-disk format. A ZIP archive containing JSON files for the document model and binary assets.

## O

**OOXML (Office Open XML)**
The ECMA-376 standard underlying DOCX, XLSX, and PPTX file formats. XML parts in a ZIP (OPC) container.

**OPC (Open Packaging Convention)**
The ZIP-based container format used by OOXML files. Defines `[Content_Types].xml`, relationships, and part naming.

## P

**Package Passthrough**
Strategy for DOCX export that retains the original OPC package and patches only modified parts, preserving unknown elements verbatim.

**Page-Granular Invalidation**
Performance optimization where a keystroke re-layouts only the affected page(s), not the entire document.

**ParaFormat**
Paragraph-level formatting properties (alignment, line spacing, indentation, borders, etc.).

## R

**Run**
The atomic text unit within a paragraph. A run has uniform character formatting. A paragraph contains one or more runs.

**Run Normalization**
Post-edit pass that merges adjacent runs with identical formatting and removes empty runs.

## S

**Section**
A document division with its own page settings (size, margins, columns, headers/footers). Contains blocks (paragraphs, tables, images).

**ShapedRun**
Output of text shaping: a sequence of glyphs with positions, advances, and font references.

**Snapshot (Display List Snapshot)**
An immutable, versioned display list for a page. The UI thread reads snapshots; the worker thread builds and publishes them via double buffering.

**StyleSheet**
Collection of named character, paragraph, and table styles with inheritance chains.

## T

**Tier (Fidelity Tier)**
See Fidelity Tier.

**tw-**
Crate name prefix derived from the repository name `tutuaword`. All Rust crates use this prefix (e.g., `tw-model`, `tw-layout`).

## U

**UAX (Unicode Annex)**
Unicode Standard Annex — supplementary specifications for text processing:
- UAX #9: Bidirectional Algorithm
- UAX #14: Line Breaking Properties
- UAX #29: Text Segmentation (word boundaries)

**Undo/Redo**
Edit history managed via the command pattern. Every `apply(command)` records the inverse command on the undo stack.

## Y

**Yjs**
A CRDT library for shared editing. Selected for real-time collaboration (ADR-0009). Rust bindings via the `yrs` crate.

**yrs**
The Rust port of Yjs. Provides CRDT data types (YText, YMap, YArray) and binary encoding.
