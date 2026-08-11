# Performance Budgets

Performance targets for the editor. Each budget includes the target, the measurement method, and the phase when it must be met.

## Budget Summary

| Metric | Target | Phase | Priority |
|--------|--------|-------|----------|
| Typing latency (p99) | <15 ms (Phase 1 gate) → <10 ms (Phase 2 target) | 1–2 | Critical |
| Open 500-page document | <2 s | 3 | Critical |
| Scroll frame time | <16 ms (60 FPS) | 1 | Critical |
| Zoom frame time | <16 ms (60 FPS) | 1 | Critical |
| Memory (500-page document) | <400 MB | 3 | High |
| Search 1000 pages | <100 ms | 2 | High |
| Save document | <1 s | 1 | Medium |
| PDF export (100 pages) | <10 s | 2 | Medium |
| AI inline suggestion | <500 ms (local), <2 s (cloud) | 4 | Medium |
| Collaboration sync latency | <200 ms | 5 | Medium |
| App startup (cold) | <3 s | 1 | Medium |
| WASM module load | <2 s | 1 (web) | Low |

## Detailed Budgets

### Typing Latency

**Target:** p99 < 10 ms from key press to display list repaint (Phase 2). The enforced
Phase 1 gate is < 15 ms; see "Enforced gates" below for what CI actually measures.

**Breakdown:**

| Step | Budget |
|------|--------|
| Flutter input capture → FFI enqueue | <0.1 ms |
| Worker: apply command to model | <1 ms |
| Worker: find affected page | <0.1 ms |
| Worker: re-layout affected page | <5 ms |
| Worker: build display list | <1 ms |
| Worker: publish snapshot | <0.1 ms |
| Flutter: deserialize + repaint | <3 ms |

**Measurement:**
- Rust benchmark: `criterion` bench for `apply(InsertText)` + `layout_page()` pipeline
- End-to-end: instrumented test that records timestamps at each step
- Run on reference hardware (see below) with a 10-page document
- Report p50, p95, p99 over 1000 keystrokes

**Phase 1 gate:** p99 < 15 ms (relaxed). Tightened to <10 ms by Phase 2.

#### Enforced gates

| Gate | Where | Measures | Threshold |
|------|-------|----------|-----------|
| `i_r1_typing_latency_50p` | `crates/tw-core/tests/r1_incremental_layout.rs` | `Session::apply` → correlated `DisplayListReady`, 50-page doc | p50 <10 ms, p99 <15 ms |
| `i_f02_s1_typing_latency` | `crates/tw-core/tests/f02_s1_typing_latency.rs` | Edit + layout pipeline | p99 <15 ms |
| `typing_incremental_relayout` | `crates/tw-core/benches/typing_latency.rs` | Warm engine, one keystroke + incremental reflow | Tracked, not gated |
| `cold_full_document_layout` | `crates/tw-core/benches/typing_latency.rs` | Cold `LayoutEngine` over the whole corpus (open path) | Tracked, not gated |

Both latency tests early-return under `cfg!(debug_assertions)`, so they only enforce
anything in `--release`. CI runs them explicitly in release; a debug-only test run
will report success without measuring latency.

`cold_full_document_layout` is the document-open path, not typing: it is expected to
be two orders of magnitude slower than `typing_incremental_relayout` and must not be
read as a keystroke cost. The benchmark that preceded them, `insert_text_and_layout_page`,
rebuilt the engine every iteration and so measured cold layout while claiming to
measure typing; it also grew the document on every iteration, so its number drifted
with the sample count.

**Not yet gated:** end-to-end key-press → painted-frame latency in Flutter. `r14_async_ffi_test.dart`
covers FFI enqueue cost only (<16 ms per call), not repaint.

### Document Open

**Target:** <2 seconds for a 500-page DOCX with mixed content (text, tables, images).

**Breakdown:**

| Step | Budget |
|------|--------|
| Read file from disk | <200 ms |
| Parse ZIP / OPC package | <100 ms |
| Parse document.xml into model | <500 ms |
| Parse styles, numbering, themes | <200 ms |
| Load images into cache | <300 ms |
| Layout all pages (parallel, rayon) | <700 ms |
| Build initial display lists | <200 ms |

**Measurement:**
- Benchmark crate: `tw-core/benches/open_document.rs`
- Test corpus: generated 500-page DOCX with 80% text, 10% tables, 10% images
- Run on reference hardware
- Report wall-clock time (includes I/O)

**Phase 1 gate:** Open 50-page native format in <500 ms.

### Scroll and Zoom Frame Time

**Target:** <16 ms per frame (60 FPS) during scroll and zoom.

**Measurement:**
- Flutter DevTools frame timeline
- Instrument `CustomPainter.paint()` duration
- Scroll test: fling gesture on 100-page document, measure frame times for 3 seconds
- Zoom test: pinch zoom from 50% to 200%, measure frame times
- Report: percentage of frames under 16 ms (target: >95%)

**Phase 1 gate:** >90% of frames under 16 ms on 10-page document.

### Memory Usage

**Target:** <400 MB resident memory for a 500-page document actively being edited.

**Breakdown (estimated):**

| Component | Budget |
|-----------|--------|
| Document model | <50 MB |
| Text buffers (ropes) | <30 MB |
| Layout cache (all pages) | <80 MB |
| Display list snapshots (visible + overscan) | <10 MB |
| Glyph atlas | <20 MB |
| Image cache | <100 MB |
| Font data | <30 MB |
| Flutter framework + UI | <80 MB |

**Measurement:**
- `/proc/self/status` (Linux), `task_info` (macOS), `GetProcessMemoryInfo` (Windows)
- Measure after document fully loaded and one page scrolled
- Report peak resident set size (RSS)
- Run on reference hardware

**Phase 1 gate:** <100 MB for 10-page document.

### Search

**Target:** <100 ms to search a 1000-page document for a single query.

**Measurement:**
- Rust benchmark: build text index, run 100 queries, report p99
- Index structure: suffix array or trigram index built on document open
- Incremental index update on edit (<5 ms per keystroke)

**Phase 2 gate:** Search 100-page document in <20 ms.

### Save

**Target:** <1 second to save a 500-page document in native format.

**Measurement:**
- Rust benchmark: serialize model to JSON, write ZIP
- DOCX save (Phase 3): patch modified parts, write ZIP — same target

**Phase 1 gate:** Save 10-page document in <100 ms.

## Reference Hardware

All benchmarks run on this configuration:

| Component | Specification |
|-----------|--------------|
| CPU | Apple M1 / Intel i5-12400 / AMD Ryzen 5 5600 |
| RAM | 16 GB |
| Storage | SSD |
| Display | 1920×1080, 1× DPI |
| OS | macOS 14+, Windows 11, Ubuntu 22.04 |

Mobile benchmarks use:
- Android: Pixel 7 or equivalent
- iOS: iPhone 13 or equivalent

## Regression Prevention

Performance benchmarks run in CI on every PR:

`.github/workflows/benchmarks.yml` runs criterion on every PR, restores the previous
baseline from cache, and calls `scripts/check-bench-regression.sh 10` to report any
benchmark more than 10% slower.

Per Phase 1 policy this job **reports without failing** — criterion on shared CI runners
is too noisy to gate on. Hard enforcement comes from the release latency tests in
`ci.yml` (see "Enforced gates" above), which assert absolute p50/p99 thresholds.

Benchmark results are stored as JSON baselines. PRs that regress any critical metric by >10% are flagged (not blocked in Phase 1; blocked in Phase 3+).

## Profiling Tools

| Tool | Use |
|------|-----|
| `criterion` | Rust micro-benchmarks |
| `cargo-flamegraph` | CPU profiling |
| `dhat` / `heaptrack` | Memory profiling |
| Flutter DevTools | UI frame timeline |
| `perf` (Linux) | System-level profiling |
| Instruments (macOS) | Time Profiler, Allocations |

## Optimization Priority

When a budget is exceeded, optimize in this order:

1. **Avoid unnecessary work** — page-granular invalidation, skip clean pages
2. **Cache aggressively** — layout cache, shaped run cache, line maps
3. **Parallelize** — rayon for multi-page layout, background thread for I/O
4. **Reduce allocations** — reuse display list buffers, arena allocation for layout
5. **Algorithm improvement** — Knuth-Plass, better indexing, lazy layout
6. **SIMD** — only after profiling confirms bottleneck in hot loop

Never optimize before measuring. Profile first, optimize the proven bottleneck.
