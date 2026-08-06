# Architecture Documentation

Engineering reference for building a Microsoft Word-class document editor with modern technologies. This documentation is the contract that Phase 1 implementation will be built against.

## Reading Order

Read these documents in order if you are new to the project:

1. [Vision](vision.md) — product goals, principles, non-goals, open questions
2. [Roadmap](roadmap.md) — six development phases with measurable exit criteria
3. [Phase Plan](phase-plan.md) — detailed phase-wise implementation with ADRs and crates
4. [Feature Phases](feature-phases.md) — F01–F28 capability backlog: stages, tests, waves, baseline status
5. [Architecture Overview](architecture/overview.md) — layers, process model, data flow
6. [Crate Map](architecture/crate-map.md) — Rust crate boundaries and dependency rules
7. [Document Model](architecture/document-model.md) — node tree, IDs, styles, revisions
8. [Text Engine](architecture/text-engine.md) — rope buffer, cursor, selection, IME, clipboard
9. [Layout Engine](architecture/layout-engine.md) — shaping, line breaking, pagination
10. [Rendering](architecture/rendering.md) — display list, glyph atlas, Flutter painter
11. [FFI Bridge](architecture/ffi-bridge.md) — Rust/Dart boundary, threading, WASM
12. [File Formats](architecture/file-formats.md) — import/export matrix, native format
13. [DOCX Compatibility](architecture/docx-compatibility.md) — OOXML fidelity strategy
14. [Performance Budgets](performance-budgets.md) — targets and measurement methods
15. [Testing Strategy](testing-strategy.md) — golden images, round-trip corpus, benchmarks
16. [Risk Mitigation](risk-mitigation.md) — Word-compatibility risks, staging (S0–S5), residual risk (wins on conflicts)
17. [Long-Tail Gaps](long-tail-gaps.md) — partial/peripheral features not yet built (PDF fonts, Hunspell, parsers, plugins, AI)
18. [UI Functionality Audit](ui-functionality-audit.md) — ribbon/menu wiring vs engine; P0/P1/P2 control status

### Later Phases (interface specs only)

19. [AI Platform](architecture/ai-platform.md)
20. [Collaboration](architecture/collaboration.md)
21. [Plugins](architecture/plugins.md)
22. [Security](architecture/security.md)

### Architecture Decision Records

All significant technology choices are recorded in [adr/](adr/). Read ADRs when you need to understand *why* a decision was made, not just *what* was decided.

| ADR | Decision |
|-----|----------|
| [0001](adr/0001-rust-core-engine.md) | Rust as the core engine language |
| [0002](adr/0002-flutter-ui-layer.md) | Flutter as the UI layer |
| [0003](adr/0003-rust-layout-flutter-paints-display-list.md) | Rust owns layout; Flutter paints display list |
| [0004](adr/0004-text-shaping-stack.md) | Pure-Rust text shaping stack |
| [0005](adr/0005-ffi-bridge-and-threading.md) | FFI bridge and threading model |
| [0006](adr/0006-native-document-format.md) | Native on-disk document format |
| [0007](adr/0007-single-mutation-path-for-undo-and-crdt.md) | Single mutation path for undo and CRDT |
| [0008](adr/0008-docx-package-passthrough.md) | DOCX package passthrough strategy |
| [0009](adr/0009-crdt-selection.md) | CRDT selection (Yjs) |
| [0010](adr/0010-ai-provider-abstraction.md) | AI provider abstraction |

### Reference

- [Glossary](glossary.md) — terms and acronyms used throughout this documentation

## Conventions

- **Crate prefix:** `tw-` (from the repository name `tutuaword`)
- **Product name:** unresolved — see [vision.md](vision.md#open-questions)
- **ADRs:** one page each — context, decision, consequences, rejected alternatives
- **Fidelity tiers:** Tier A (round-trip lossless), Tier B (render + preserve), Tier C (preserve only)
