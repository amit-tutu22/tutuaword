# ADR-0002: Flutter as the UI Layer

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 1

## Context

The UI must run on Windows, macOS, Linux, Android, iOS, and Web from a single codebase. A Word-class editor requires custom rendering (pages, rulers, zoom, canvas, floating objects, custom cursor) — it is not a form-based application. The UI framework must support GPU-accelerated custom painting and provide a native feel on each platform.

## Decision

Use **Flutter** for the entire UI layer — toolbar, document view, sidebar, dialogs, settings, and all platform-specific UI chrome.

Flutter communicates with the Rust engine exclusively through the FFI bridge (`tw-ffi` / `tw-wasm`). Flutter does not perform text shaping, layout, or document model operations.

## Consequences

**Positive:**
- Single UI codebase for 6 platforms (Windows, macOS, Linux, Android, iOS, Web)
- Full control over every pixel via `CustomPainter` — essential for page rendering
- GPU-accelerated rendering with Skia (same engine as Chrome)
- Excellent animation support for cursor blink, page transitions, zoom
- Mature desktop support (Flutter 3.x stable on all desktop platforms)
- Hot reload accelerates UI development
- Large ecosystem of packages for non-editor UI (settings, dialogs, file pickers)

**Negative:**
- Flutter app size is larger than native (~15–25 MB minimum)
- Some platform integrations require platform channels (file associations, print dialog)
- Flutter web performance is acceptable but not native-speed for very large documents
- Team needs Dart/Flutter expertise in addition to Rust

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **React + Tauri** | Tauri's webview rendering cannot achieve 60 FPS custom page painting; webview text rendering conflicts with Rust-side shaping |
| **Electron** | Memory overhead (200+ MB baseline); Chromium text rendering same conflict as Tauri |
| **Native per platform (Swift/C#/Kotlin)** | 3–5 separate UI codebases; unsustainable for a 6–20 person team |
| **Qt (C++)** | Would require C++ UI code separate from Rust engine; Qt licensing complexity |
| **Slint** | Immature for complex desktop applications; small ecosystem |
