# ADR-0008: DOCX Package Passthrough Strategy

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 3

## Context

Microsoft Word compatibility requires opening, editing, and saving `.docx` files without losing formatting. DOCX is an OPC (Open Packaging Convention) ZIP archive containing XML parts. A typical document has 10–50 parts; complex documents may have hundreds.

The naive approach — parse all XML into our model, then regenerate the entire DOCX from scratch — fails because:
- Unknown XML elements and attributes are lost
- Element ordering and whitespace may differ
- Custom XML parts, embedded objects, and proprietary extensions are discarded
- Relationship IDs and namespace declarations change
- Round-trip fidelity is typically 60–70% for real-world documents

Word itself preserves unknown elements when saving. We must do the same.

## Decision

**Retain the original OPC package and patch only modified parts.**

On import:
1. Store the entire ZIP archive as `DocxPackage`
2. Parse known parts into the document model
3. Store raw bytes for unknown/unparsed parts

On export:
1. Start with the original `DocxPackage`
2. Re-serialize only parts that were modified during editing
3. Leave all unmodified parts as original bytes
4. Update `[Content_Types].xml` and relationships only if parts were added/removed

Fidelity is classified into three tiers:
- **Tier A:** Full parse, edit, lossless round-trip
- **Tier B:** Parse for rendering, preserve raw bytes on save
- **Tier C:** Preserve raw bytes only, no rendering

## Consequences

**Positive:**
- Unknown OOXML elements survive save cycles — critical for 99% compatibility target
- Custom XML, macros, ActiveX, ink annotations are never lost on unmodified passthrough export
- Save is fast — only modified parts are re-serialized
- Incremental compatibility — new part parsers can be added without affecting existing passthrough
- Matches Word's own behavior — users expect this

**Negative:**
- Memory overhead — original ZIP bytes stored alongside parsed model (~2× for unmodified documents)
- Complexity — must track which parts are modified vs pristine
- Cannot fully edit Tier C elements (macros, SmartArt) — they are preserved but not interactive
- Testing requires verifying both model correctness AND byte-level preservation

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **Full regeneration from model** | 60–70% fidelity on real-world documents; loses unknown elements; unacceptable for Word compatibility |
| **Model-only (no DOCX export)** | Users require saving as DOCX; export-only-to-PDF limits interoperability |
| **LibreOffice as conversion backend** | External dependency; licensing concerns; no control over fidelity; cannot embed in mobile/WASM |
| **Template-based generation** | Cannot handle arbitrary document structures; only works for documents matching templates |
