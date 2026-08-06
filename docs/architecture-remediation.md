# Architecture Remediation Plan

**Status:** Active (2026-08-06)  
**Source:** Adversarial multi-model architecture review (Opus, Composer, Grok, Terra) + [Explore app architecture](architecture/overview.md) brief  
**Authority:** This document governs **implementation order** when it conflicts with feature velocity. See [ADR-0011](adr/0011-architecture-remediation-program.md).

Related: [ADR-0003](adr/0003-rust-layout-flutter-paints-display-list.md), [ADR-0005](adr/0005-ffi-bridge-and-threading.md), [ADR-0007](adr/0007-single-mutation-path-for-undo-and-crdt.md), [ADR-0008](adr/0008-docx-package-passthrough.md), [risk-mitigation.md](risk-mitigation.md), [feature-phases.md](feature-phases.md), [ui-functionality-audit.md](ui-functionality-audit.md), [performance-budgets.md](performance-budgets.md).

---

## Executive summary

The **layered Rust engine + Flutter UI** architecture is the right long-term bet. The **running code contradicts load-bearing ADRs** in ways that make F04–F28, web, mobile, and collaboration non-viable if unaddressed.

**Do not expand the feature surface (F04+) until Remediation Phase R1 exits.** Engine APIs added before R1 complete will be built on O(whole-document) relayout, UI-thread blocking FFI, and a document model that cannot represent most Word content.

### Top three structural fixes (consensus, all reviewers)

| # | Fix | Unblocks |
|---|-----|----------|
| 1 | **Incremental layout + per-page snapshots** | Performance, mobile, long documents |
| 2 | **Separate session atlas from page display lists** | Memory, FFI transfer, scroll |
| 3 | **Async FFI (no UI-thread wait)** | UX, ADR-0005 compliance, web path |

---

## Current state vs documented architecture

| ADR / spec claim | Documented | Implemented today | Severity |
|------------------|------------|-------------------|----------|
| Page-granular invalidation (ADR-0005) | Yes | `worker.rs` → `invalidate_all()` + full `layout_document()` | Critical |
| UI thread never blocks (ADR-0005) | Yes | `wait_for_events` busy-wait up to 100 ms (edit) / 30 s (open/save) on Dart isolate | Critical |
| Zero-copy display lists | Yes | Atlas embedded per page; multiple clone/copy layers Dart↔Rust | Critical |
| Single mutation path (ADR-0007) | Yes | `BridgeCommand` shortcuts + format import constructs `Document` | Warning |
| Flutter not used for document text (ADR-0003) | Yes | `EditorController` TextField fallback + mock pagination | Warning |
| Six-platform matrix | Yes | macOS app only; `tw-wasm` does not compile (`std::thread` in `tw-core`) | Critical |
| Strict downward crate deps | Yes | `tw-pdf` → layout/render/edit; `tw-plugin` ↔ `tw-core` cycle in docs | Warning |
| `cargo-deny` in CI | Yes | Not configured | Nit |

Full finding catalog with file references is in the review synthesis (2026-08-06); key code paths:

- `crates/tw-core/src/worker.rs` — full rebuild loop
- `crates/tw-render/src/display_list.rs` — atlas bytes in page payload
- `crates/tw-ffi/src/lib.rs` — sync wait, buffer ownership, global session
- `app/lib/editor/editor_controller.dart` — god-object + dual pipeline
- `crates/tw-model/src/nodes.rs` — minimal block/run vocabulary

---

## Remediation phases

Phases are **sequential gates**. Each phase has exit criteria that must pass in CI before the next phase starts.

```
R0 Safety & correctness ──► R1 Performance shape ──► R2 Architecture debt ──► R3 Platform & fidelity
     (1–2 weeks)              (3–6 weeks)              (4–8 weeks)              (ongoing)
```

Wave mapping: **R0–R1 complete before W2 (F04+) expansion.** R2 overlaps W1 hardening. R3 aligns with S1–S2 in [risk-mitigation.md](risk-mitigation.md).

---

## R0 — Safety and correctness

**Goal:** Stop silent data loss and undefined behavior at the FFI boundary.

### R0.1 FFI buffer ownership

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Fix deallocator | `tw-ffi` `tw_free_buffer` | `Box<[u8]>` transfer + `Box::from_raw` on slice | **Done** |
| Panic isolation | `extern "C"` exports | `guard_ffi` / `guard_ffi_void` on every `#[no_mangle]` export | **Done** |
| Reject bad IDs | `tw_apply_insert_text` | Return `-2` via `parse_node_id`; no random UUID fallback | **Done** |
| Remove `static mut` | `EVENT_CALLBACK` | `OnceLock<EventCallback>` | **Done** |

**Exit:** `cargo test -p tw-ffi --test buffer_lifetime` — all pass. Every `#[no_mangle]` export wrapped in `guard_ffi` / `guard_ffi_void`.

### R0.2 Command/event correlation

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Request IDs | `tw-core` worker/session | Monotonic `request_id` on every `BridgeCommand`; echo in `BridgeEvent`; `STARTUP_REQUEST_ID = 0` | **Done** |
| Event buffering | `session.rs` | `pending_events` buffer; `wait_for_response(request_id)` | **Done** |
| FFI correlated waits | `tw-ffi` | All edit/open/save/spell exports capture `request_id` and wait on matching event | **Done** |
| Rust integration test | `tw-core/tests/command_correlation.rs` | Concurrent save + insert; each `request_id` gets correct event type | **Done** |
| Dart routing | `NativeEngine` | Per-request `Completer` map; no global “any DisplayListReady” | **Done** |
| Autosave isolation | Dart + session | Autosave serializes behind user edits or uses dedicated channel | **Done** |

**Exit (Rust):** `cargo test -p tw-core command_correlation` passes. **Exit (full):** Integration test: type during autosave; both complete without cross-talk.

### R0.3 Command queue backpressure

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Stop silent drops | `session.rs` `send_command` | Blocking `send` on bounded command queue; `None` only when worker disconnected | **Done** |
| Bound events | Worker → UI event channel | Bounded channel + `EventPublisher`: coalesce `DisplayListReady` per page; drop-oldest under pressure | **Done** |
| Stress test | `tw-core/tests/command_backpressure.rs` | 1000 rapid inserts; zero lost characters | **Done** |

**Exit:** Stress test: 1000 rapid inserts; zero lost characters; document text matches input. Intermediate `DisplayListReady` events may be coalesced — the test polls document text, not per-request paint acks.

### R0 CI gates (add immediately)

Added to `.github/workflows/ci.yml` after workspace tests:

```yaml
- cargo test -p tw-ffi --test buffer_lifetime -- --test-threads=1
- cargo test -p tw-core --test command_correlation
- cargo test -p tw-core --test command_backpressure
```

Status: **Done**

---

## R1 — Performance shape

**Goal:** Match ADR-0005 performance story: O(dirty pages) per edit, bounded memory, non-blocking UI.

### R1.1 Incremental layout

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Dirty node set | `tw-edit` `EditResult` | Ensure every command reports `affected_nodes` | **Done** |
| Block-scoped reflow | `tw-layout` | `invalidate(node_id)` → reflow from first dirty block; stop when page boundaries stable | **Done** |
| Worker integration | `worker.rs` | Remove `invalidate_all()` from hot path; chain forward pages asynchronously | **Done** |
| Layout epochs | `tw-core` | Version layout; mark downstream pages `pending`; disable hit-test on stale pages | **Done** |

**Exit:**

- `U-R1-incremental-layout`: edit on page 1 of 50-page fixture relayouts ≤3 pages synchronously — **Done** (`crates/tw-core/tests/r1_incremental_layout.rs`)
- `I-R1-typing-latency-50p`: p50 &lt;15 ms, p99 &lt;20 ms via real `Session` + worker — **Done** (`crates/tw-core/tests/r1_incremental_layout.rs`)

### R1.2 Session atlas separation

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Remove atlas from page DL | `tw-render` `DisplayList` | Page payload = draw batches only | **Done** |
| Atlas resource API | `tw-ffi` | `tw_get_atlas(generation, out_ptr, …)`; versioned separately | **Done** |
| Dart cache | `document_view.dart` | Re-upload texture only when atlas generation changes | **Done** |
| Dirty rects | `tw-shape` atlas | Optional: sub-rect upload for small edits | Skipped |

**Exit:**

- 10-page doc keystroke allocates &lt;2 MB across FFI (measure in test) — **Done** (`crates/tw-core/tests/r1_atlas_separation.rs`)
- Display list v4 wire format updated + migration note in [rendering.md](architecture/rendering.md) — **Done**

### R1.3 Per-page snapshots

| Task | Location | Action |
|------|----------|--------|
| Page store | `tw-core/snapshot.rs` | `HashMap<(page, version), Arc<PageSnapshot>>` |
| Publish | Worker | Emit `DisplayListReady { page, version }` per dirty page only |
| Lazy fetch | Flutter | Off-screen pages fetched on scroll, not rebuilt every edit |
| Arc reads | `SnapshotBuffer` | Replace `clone()` on read with `Arc` handoff |

**Exit:** 500-page doc resident snapshot memory &lt;400 MB budget (see [performance-budgets.md](performance-budgets.md)).

### R1.4 Async FFI

| Task | Location | Action |
|------|----------|--------|
| Fire-and-forget edits | `tw-ffi` | Remove `wait_for_document_edit` from production edit exports |
| Wire callback | `tw-ffi` + Dart | Implement `EVENT_CALLBACK` via `NativeCallable.listener` / `Dart_PostCObject` |
| Isolate option | `native_engine.dart` | Long-lived helper isolate for blocking open/save if needed |
| Delete busy-wait | `tw-ffi` | `wait_for_events` test-only behind `#[cfg(test)]` |

**Exit:**

- Flutter integration test: 60 keystrokes in 1 s; no frame &gt;16 ms blocked on FFI
- ADR-0005 “UI thread never blocks” marked **Implemented** in ADR status table

---

## R2 — Architecture debt

**Goal:** One engine path, one mutation surface, decomposed UI, model ready for F04–F28.

### R2.1 Document model vocabulary

| Task | Action |
|------|--------|
| Expand enums | Add `RunContent` variants: `Hyperlink`, `Field`, `InlineImage`, `FootnoteRef`, `CommentRef`, `Bookmark` (placeholder render OK) |
| Expand blocks | Add `ShapeBlock`, header/footer typed maps per [document-model.md](architecture/document-model.md) |
| `#[non_exhaustive]` | Public enums in `tw-model` to allow forward-compatible extension |

**Exit:** DOCX import records element retention counts; failing test lists missing OOXML types.

### R2.2 Single text storage

| Task | Action |
|------|--------|
| Pick one store | **Recommendation:** `String` per run; delete per-run `Rope` in `TextBuffer` OR make rope sole store |
| Layout cache | `Arc<Document>` + version; no deep clone per edit |
| Read path | All queries (`format_at`, hit test) on versioned snapshot |

**Exit:** Property test: 10k random edits; run text == buffer text == export plaintext.

### R2.3 Command-only mutations

| Task | Action |
|------|--------|
| Collapse `BridgeCommand` | Worker handles only `ApplyEdit { command: Command }` + lifecycle commands |
| Remove shortcuts | Migrate `ApplyHeading1`, `InsertTable`, … to `Command` variants |
| Import purity | Format crates return `Document`; only `tw-edit` holds `&mut Document` in production |

**Exit:** `rg 'BridgeCommand::ApplyHeading'` returns zero; undo works for all migrated paths.

### R2.4 EditorController decomposition

| Split | Owns |
|-------|------|
| `DocumentSessionController` | Open/save/autosave/recents/properties |
| `SelectionController` | Caret, selection (`DocRange`), hit-test, drag |
| `FormattingController` | Ribbon state from `tw_get_caret_format` JSON only |
| `ViewController` | Zoom, page, ruler, navigation pane |
| `EditorController` | Thin composition + `ChangeNotifier` fan-out |

**Additional:**

- Delete TextField fallback in production; mock `NativeEngine` for tests
- Selection as `DocRange` in model; engine converts range → rects (never pixel → range)

**Exit:**

- `editor_controller.dart` &lt;800 lines
- CI builds `libtw_ffi` and runs Flutter tests against real engine
- P0 count in [ui-functionality-audit.md](ui-functionality-audit.md) = 0

### R2.5 FFI consolidation

| Task | Action |
|------|--------|
| Codegen | Adopt `flutter_rust_bridge` or equivalent for query/format APIs |
| Dispatch | `tw_dispatch(u32 request_id, command_bytes)` for edits |
| Stable C ABI | Keep narrow exports for display list + atlas bytes only |

**Exit:** New `SetCharFormat` field requires zero manual Dart typedef additions.

### R2.6 Undo transactions

| Task | Action |
|------|--------|
| `Transaction` type | Group commands; capture selection; support abort without stack pollution |
| Coalescing | Merge consecutive `InsertText` same run within 1 s / word boundary |
| Total inverse | `inverse()` returns `Result`, not `Option` |

**Exit:** 100-char word types as one undo step; batch failure rolls back atomically.

---

## R3 — Platform and fidelity

**Goal:** Honest multi-platform posture; Word fidelity gates; collaboration-ready foundations.

### R3.1 Executor abstraction (web)

| Task | Action |
|------|--------|
| `EngineExecutor` trait | `ThreadedExecutor` (native), `InlineExecutor` (WASM) |
| `tw-core` | No direct `std::thread::spawn` in session constructor |
| CI | `cargo check --target wasm32-unknown-unknown -p tw-wasm` |
| Fonts | Byte-slice font provider; no `fontdb/fs` on WASM |

**Exit:** `tw-wasm` compiles in CI; smoke test opens bytes in JS test harness.

### R3.2 Platform CI matrix

| Target | Minimum CI |
|--------|------------|
| Linux | Existing + `libtw_ffi.so` build |
| macOS | Build + Flutter test job |
| Windows | Cross-check or scheduled job |
| WASM | `wasm32` check |

### R3.3 DOCX fidelity hardening

| Task | Action |
|------|--------|
| Element retention | Corpus test counts OOXML element types before/after round-trip |
| Tier B gating | Editing styles/numbering forces dependent part re-serialize or dirty flag |
| Within-part preserve | Token-preserving transforms for modified `document.xml` (Terra finding) |
| Vector in model | Move `resvg` rasterization out of `tw-docx` into `tw-render` |

### R3.4 CRDT foundations (before F20)

| Task | Action |
|------|--------|
| Operation metadata | `origin: Local \| Remote`, `op_id`, transaction boundaries in `Command` |
| Position model | Prototype CRDT text positions; stop using bare `(run_id, usize)` for collab |
| Vertical slice | Two clients, paragraph text + bold, merge via `yrs` |
| Undo model | Document selective/local undo semantics; inverse-stack insufficient for collab |

**Exit:** ADR-0009 marked **Partial** with working 2-client demo.

### R3.5 Accessibility (F21 prep)

| Task | Action |
|------|--------|
| Parallel tree | Semantic tree from model (not display list) |
| Platform hooks | Flutter `Semantics` for paragraphs, tables, headings |

### R3.6 Governance

| Task | Action |
|------|--------|
| `cargo-deny` | Enforce layer rules from [crate-map.md](architecture/crate-map.md) |
| ADR contract tests | One test per ADR invariant (no sync wait, no full relayout in worker hot path) |
| Audit gate | CI fails if P0 &gt; 0 in ui-functionality-audit |
| Realistic perf gates | 50-page + 500-page fixtures through Session+FFI |

---

## What not to do during R0–R1

- Add new `tw_*` C exports for individual ribbon features
- Wire F04+ ribbon tabs before R2.4 decomposition
- Claim web/mobile support in marketing or README
- Expand `BridgeCommand` with new shortcut variants
- Build F20 collaboration on current `tw-crdt` stub

---

## Success metrics

| Metric | Target | Phase |
|--------|--------|-------|
| Keystroke p99 (50-page DOCX, Session+FFI) | &lt;15 ms | R1 |
| Snapshot memory (500 pages) | &lt;400 MB | R1 |
| FFI bytes per keystroke (10 pages) | &lt;2 MB | R1 |
| UI frame blocked on FFI | 0 ms | R1 |
| P0 UI audit items | 0 | R2 |
| `wasm32` compile | Green CI | R3 |
| OOXML element retention (Tier A corpus) | 100% known types | R3 |

---

## Tracking

Update this document when a phase exits. Link PRs to phase IDs (`R1.2-atlas-separation`, etc.).

| Phase | Status | Exit date |
|-------|--------|-----------|
| R0 | **Done** | 2026-08-06 |
| R1 | **In progress** (R1.1–R1.2 done) | |
| R2 | Not started | |
| R3 | Not started | |

---

## Review provenance

Findings synthesized from independent adversarial reviews (2026-08-06):

- Opus — code-level defects (atlas duplication, FFI UB, triple text store, model gap)
- Composer — CI blind spots, `BridgeCommand` duplication, memory math
- Grok — dual pipeline, CRDT stub, accessibility
- Terra — layout epochs, DOCX within-part loss, plugin cycle

Architecture brief: explore pass over repo layout and docs.
