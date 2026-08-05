# ADR-0003: Rust Owns Layout; Flutter Paints Display List

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 1

## Context

Text rendering requires a decision about where layout and shaping happen relative to the UI layer. Flutter provides `ui.Paragraph` for text rendering, but it performs its own shaping internally. If Rust shapes text for layout (cursor positioning, line breaking, selection) and Flutter re-shapes for rendering, the two pipelines may produce different glyph positions — breaking cursor alignment, selection highlighting, and hit testing.

Additionally, `dart:ui` exposes no API to draw individual glyphs at specified positions. There is no `canvas.drawGlyph(font, glyphId, x, y)`.

## Decision

**Rust owns the entire layout and shaping pipeline.** Flutter receives a pre-built display list and paints it via:
- `canvas.drawRawAtlas()` for text glyphs (from a Rust-built glyph atlas)
- `canvas.drawRect()` for borders, backgrounds, highlights
- `canvas.drawPath()` for shapes, lines, curves
- `canvas.drawImageRect()` for embedded images

Flutter's `ui.Paragraph`, `TextPainter`, and `RichText` widgets are **not used** for document content.

## Consequences

**Positive:**
- Single shaping pipeline — layout and rendering use identical glyph positions
- Cursor, selection, and hit testing are pixel-accurate
- Complex script support (Arabic, Indic, CJK) is consistent across platforms
- Rust controls font fallback chains and OpenType feature selection
- Display list is platform-agnostic — same data works on native and WASM

**Negative:**
- Must build and maintain a glyph atlas system
- `drawRawAtlas` requires pre-rasterized glyph bitmaps — memory overhead for large documents
- Flutter overlay elements (cursor, selection, IME underline) must query Rust for positions
- Cannot leverage Flutter's text selection or accessibility text APIs directly

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **Flutter shapes and renders (`ui.Paragraph`)** | Double shaping causes layout/render mismatch; no control over complex scripts; cursor positioning unreliable |
| **Rust shapes, Flutter renders via platform text APIs** | Platform-specific text APIs differ; no cross-platform consistency; still no glyph-level control in Flutter |
| **Rust owns layout AND rendering via Skia texture** | Maximum control but heaviest build setup; requires cmake and per-platform Skia builds; Flutter becomes a texture viewer |
| **HTML/CSS rendering (web only)** | Only works on web; different rendering on web vs desktop; CSS layout != Word layout |
