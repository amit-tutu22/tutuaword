# ADR-0006: Native On-Disk Document Format

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 1

## Context

The editor needs a primary save format for editing. Options include using DOCX as the native format, inventing a custom binary format, or using a structured text format. The native format is used during editing; DOCX is the interchange format for compatibility.

Requirements: lossless, diffable (Git-friendly), human-readable (debuggable), fast load/save, extensible.

## Decision

The native format is **`.twdoc`** — a ZIP archive containing JSON files for the document model and a `media/` directory for binary assets (images, fonts).

Key files: `manifest.json`, `content.json`, `styles.json`, `settings.json`, `media/`.

Format version follows semver. Migration functions handle major version upgrades.

## Consequences

**Positive:**
- JSON is human-readable — developers can inspect and debug documents with any text editor
- Git diffs work on JSON content (meaningful diffs for text changes)
- ZIP provides efficient storage and familiar structure (same as DOCX/ODT)
- Extensible — new JSON fields added without breaking old readers (sparse serialization)
- Fast parse — `serde_json` deserializes a 500-page document in <500 ms
- No license encumbrance — fully open format

**Negative:**
- Larger file size than binary format (~2–3× vs optimized binary)
- JSON parsing overhead vs binary deserialization (acceptable: <500 ms target)
- Not interoperable with other applications (by design — DOCX is the interchange format)
- Media files in ZIP cannot be memory-mapped individually (acceptable for document sizes)

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **DOCX as native format** | OOXML is complex to round-trip; editing requires constant XML regeneration; poor Git diffs; passthrough strategy works better with DOCX as interchange only |
| **Custom binary format** | Not human-readable; not Git-diffable; harder to debug; serialization/deserialization code is error-prone |
| **SQLite** | Overkill for document storage; poor Git diffs; adds dependency; complicates media storage |
| **Protocol Buffers / FlatBuffers** | Not human-readable; poor Git diffs; harder to debug; extensibility requires schema evolution discipline |
| **Markdown as native format** | Cannot represent Word-level formatting (styles, tables, images, sections); not lossless |
