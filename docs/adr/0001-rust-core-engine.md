# ADR-0001: Rust as the Core Engine Language

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 1

## Context

The document engine — model, layout, shaping, rendering, file parsing, undo, search — is the performance-critical core of the application. It must run on desktop (Windows, macOS, Linux), mobile (Android, iOS), and web (WASM). The language choice for this core determines performance, safety, cross-platform reach, and team hiring for the entire project.

## Decision

Implement the core engine in **Rust**.

All engine crates (`tw-model`, `tw-text`, `tw-shape`, `tw-layout`, `tw-render`, `tw-edit`, format parsers) are Rust. The engine compiles to native libraries for desktop/mobile and to WASM for web.

## Consequences

**Positive:**
- Memory safety without garbage collection — critical for a long-running editor process
- Native performance for layout and shaping (target: <10 ms typing latency)
- Single codebase compiles to native + WASM
- Strong type system catches errors at compile time across crate boundaries
- Excellent ecosystem for text processing (rustybuzz, swash, ropey, icu4x)
- Growing hiring pool for Rust developers

**Negative:**
- Steeper learning curve for team members new to Rust
- Compile times are longer than C++ or Go
- Some C library bindings (Skia, PDFium) require `unsafe` FFI
- WASM binary size needs management (target: <5 MB compressed)

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **C++** | Memory safety risks in a complex layout engine; no WASM support; harder to hire |
| **C** | Same safety concerns as C++; no modern tooling |
| **Go** | GC pauses unacceptable for <10 ms typing latency; poor WASM support |
| **Zig** | Immature ecosystem for text processing; smaller hiring pool |
| **TypeScript (Node)** | Cannot meet performance targets for layout/shaping; no native mobile |
