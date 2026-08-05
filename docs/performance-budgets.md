# Performance Budgets

Performance targets for the editor. Each budget includes the target, the measurement method, and the phase when it must be met.

## Budget Summary

| Metric | Target | Phase | Priority |
|--------|--------|-------|----------|
| Typing latency (p99) | <10 ms | 1 | Critical |
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

**Target:** p99 < 10 ms from key press to display list repaint.

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

```yaml
# .github/workflows/benchmarks.yml (future)
- name: Run Rust benchmarks
  run: cargo bench -- --save-baseline main
- name: Compare benchmarks
  run: cargo bench -- --baseline main
- name: Fail on regression >10%
  run: ./scripts/check_regression.sh --threshold 10
```

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
