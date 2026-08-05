# tutuaword

A Microsoft Word-class document editor built with modern technologies — Rust, Flutter, CRDTs, AI, and WebAssembly.

## Vision

A professional, AI-native document editor that opens and edits `.docx` files with high fidelity, works across all platforms, and differentiates through built-in AI, real-time collaboration, and modern architecture.

**Microsoft Word + Notion AI + Grammarly + Git versioning + Local AI**

## Architecture Documentation

Full engineering reference: **[docs/README.md](docs/README.md)**

| Document | Description |
|----------|-------------|
| [Vision](docs/vision.md) | Product goals, principles, non-goals |
| [Roadmap](docs/roadmap.md) | 6-phase development plan with exit criteria |
| [Architecture Overview](docs/architecture/overview.md) | Layers, process model, data flow |
| [Crate Map](docs/architecture/crate-map.md) | Rust crate boundaries |
| [ADRs](docs/adr/) | Architecture Decision Records |

## Technology Stack

| Layer | Technology |
|-------|-----------|
| UI | Flutter (Windows, macOS, Linux, Android, iOS, Web) |
| Core Engine | Rust (native + WASM) |
| Text Shaping | rustybuzz, swash, fontdb |
| Layout | Custom (UAX #14 line breaking, pagination) |
| Rendering | Display list → Flutter CustomPainter |
| Collaboration | Yjs (CRDT) |
| AI | Multi-provider abstraction (OpenAI, Gemini, Claude, llama.cpp, ONNX, Apple FM) |

## Status

**Phase 0: Architecture** — Complete (docs + ADRs)

**Phase 1: Editor Foundation** — Complete

**Phase 2: Word Processing** — In progress

- Pagination, tables, images, lists, styles, PDF export
- Extended model: `Table`, `ImageBlock`, `NumberingRef`, style resolution
- Multi-page layout engine with table grid rendering
- Display list v2: rect, path, image batches
- Flutter: rulers, page navigator, print preview, table/image/PDF toolbar

See [docs/phase-plan.md](docs/phase-plan.md) for the full phase-wise plan.

## License

See [LICENSE](LICENSE).
