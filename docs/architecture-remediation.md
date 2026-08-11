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
| UI thread never blocks (ADR-0005) | Yes | Edits fire-and-forget; Dart awaits events via `NativeEventRouter` | **Implemented** (R1.4) |
| Zero-copy display lists | Yes | Atlas embedded per page; multiple clone/copy layers Dart↔Rust | Critical |
| Single mutation path (ADR-0007) | Yes | All edits via `ApplyEdit { command }`; format import returns `Document` | **Implemented** (R2.3) |
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
| Bound events | Worker → UI event channel | Bounded channel + `EventPublisher`: coalesce `DisplayListReady` per page | **Done** |
| Keep correlations | `EventPublisher` | Coalescing drops the event, never the `request_id`: superseded ids queue in a compact `VecDeque<u64>` and re-emit when the channel drains | **Done** |
| Stress test | `tw-core/tests/command_backpressure.rs` | 1000 rapid inserts; zero lost characters | **Done** |
| Correlation stress test | `tw-core/tests/r1_event_correlation_backpressure.rs` | 1024 same-page edits with no consumer; every `request_id` still completes | **Done** |

**Exit:** Stress test: 1000 rapid inserts; zero lost characters; document text matches input. Intermediate `DisplayListReady` events are coalesced into a single repaint per page, but each edit's `request_id` still receives a completion, so a caller awaiting an edit never times out.

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
- `U-R1-cache-retention`: an incremental pass re-caches only its dirty pages and drops caches for pages the edit removed — **Done** (`crates/tw-layout/tests/r1_incremental_cache_retention.rs`)
- `I-R1-forward-relayout`: after an edit on page 0 of a ~50-page flowing fixture, background reflow clears every `pending` page and hit-test resolves on the last page — **Done** (`crates/tw-core/tests/r1_forward_relayout.rs`)
- `I-R1-page-count-reuse`: growing the page count reuses the display-list `Arc` of every page whose layout is unchanged — **Done** (`crates/tw-core/tests/r1_page_count_change_reuse.rs`)
- `U-R1-stale-window`: a converged pass during background catch-up does not re-mark pages an earlier chunk already rebuilt — **Done** (`crates/tw-layout/tests/r1_stale_window.rs`)

An incremental pass stops at the first block boundary whose flow state (page index, pen `y`, section geometry, list counters, pending boxes) matches the previous pass — the tail is then provably identical and reused verbatim. If the 3-page cap is hit first, the block is recorded as a resume point and the worker finishes the reflow in chunks whenever the command queue is empty, emitting repaints under `BACKGROUND_REQUEST_ID`. Pages after the reflow window stay `pending` (hit-test disabled) until that background pass reaches them.

The `pending` window is `[LayoutEngine::pending_reflow_page(), page_count)`. That floor is the first page no pass has rebuilt since the edit that opened the window; it advances as background chunks land and survives a later converged pass, which by definition proves the tail is unchanged. Without that, every keystroke during catch-up would re-mark pages the background pass had already fixed. On a 48-page flowing fixture, one large front-of-document insert marks 44 pages `pending`, the midpoint clears at ~215 ms and the last page at ~460 ms in release (`crates/tw-core/tests/r1_stale_window_probe.rs`, `#[ignore]`d as a measurement). Edits whose reflow converges inside the synchronous window mark nothing `pending` at all.

Dart distinguishes the two hit-test failure modes through `tw_is_page_stale`; `tw_hit_test` also returns `-4` for a stale page versus `-2` for a genuine miss. Conflating them makes a click on a not-yet-reflowed page look like a click on an empty page, which drops the caret at an unrelated tail position.

Staleness gates geometry queries only. The document model stays current throughout a pending reflow, so `tw_document_tail_hit` — which resolves the document's last run and only echoes `page` back for caret display — answers regardless of staleness, and select-all keeps working during catch-up (`crates/tw-ffi/tests/r1_stale_page_hit_tests.rs`).

### R1.2 Session atlas separation

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Remove atlas from page DL | `tw-render` `DisplayList` | Page payload = draw batches only | **Done** |
| Atlas resource API | `tw-ffi` | `tw_get_atlas(generation, out_ptr, …)`; versioned separately | **Done** |
| Generation-only query | `tw-ffi` | `tw_get_atlas_generation(out_generation)` so a change check costs no pixel copy | **Done** |
| Dart cache | `document_view.dart` | Re-upload texture only when atlas generation changes | **Done** |
| Dirty rects | `tw-shape` atlas | Optional: sub-rect upload for small edits | Skipped |

**Exit:**

- 10-page doc keystroke allocates &lt;2 MB across FFI (measure in test) — **Done** (`crates/tw-core/tests/r1_atlas_separation.rs`)
- Display list v4 wire format updated + migration note in [rendering.md](architecture/rendering.md) — **Done**

### R1.3 Per-page snapshots

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Page store | `tw-core/snapshot.rs` | `HashMap<(page, version), Arc<PageSnapshot>>` | **Done** |
| Publish | Worker | Emit `DisplayListReady { page, version }` per dirty page only | **Done** |
| Lazy fetch | Flutter | Off-screen pages fetched on scroll, not rebuilt every edit | **Done** |
| Arc reads | `SnapshotBuffer` | Replace `clone()` on read with `Arc` handoff | **Done** |

**Exit:** 500-page doc resident snapshot memory &lt;400 MB budget (see [performance-budgets.md](performance-budgets.md)) — **Done** (`crates/tw-core/tests/r1_per_page_snapshots.rs`).

### R1.4 Async FFI

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Fire-and-forget edits | `tw-ffi` | Remove `wait_for_document_edit` from production edit exports | **Done** |
| Wire callback | `tw-ffi` + Dart | Implement `EVENT_CALLBACK` via `NativeCallable.listener` / `Dart_PostCObject` | **Done** |
| Async open/save/spell | `tw-ffi` | Enqueue returns `request_id`; result collected later by id (`tw_*_async` + `tw_take_*`), blocking exports kept as wrappers | **Done** |
| Event pump | `tw-ffi` | `tw_pump_events` so a host that never blocks still receives worker events; **required**, not optional | **Done** |
| Isolate option | `native_engine.dart` | Long-lived helper isolate for blocking open/save if needed | Not needed (async exports supersede) |
| Delete busy-wait | `tw-ffi` | `wait_for_events` test-only behind `#[cfg(test)]` | **Done** |

**Exit:**

- Flutter integration test: 60 keystrokes in 1 s; no frame &gt;16 ms blocked on FFI — **Done** (`app/test/r14_async_ffi_test.dart`)
- ADR-0005 “UI thread never blocks” marked **Implemented** in ADR status table — **Done**
- `I-R1-async-document-exports`: enqueue returns an id, the result survives until collected, and abandoned slots are evicted rather than leaked — **Done** (`crates/tw-ffi/tests/r1_async_document_exports.rs`)

Async results live in a bounded 16-slot, oldest-first-evicted table keyed by `request_id`. Getters return `1` while in flight, `0` with the payload, `-2` on failure, `-3` for an unknown or aged-out id. Dart remains free to use the blocking wrappers during migration, but must not pump from another isolate while one is parked.

`Session::drain_events` is the only thing that invokes the event observer, and nothing calls it spontaneously — enqueuing a command does not. Before `tw_pump_events` existed, the Dart callback fired only when some unrelated blocking export happened to drain the channel as a side effect. Dart now runs an adaptive pump in `NativeEventRouter` (2 ms while a correlated request is outstanding, 16 ms idle). The requirement is stated at the top of [ffi-bridge.md](architecture/ffi-bridge.md) and on the `tw_init` and `Session::set_event_observer` doc comments, and a correlated wait that times out having never been pumped prints a one-time warning to stderr naming the cause.

Chasing the missing pump turned up the actual cause of the 30 s `TimeoutException` on select-all: the callback passed the correlation id only through a pointer to a stack buffer, and Dart's `NativeCallable.listener` read it a turn of the event loop later, after the frame was gone. No correlation id was ever delivered correctly, and `nativeFfiEventsAvailable()` was consequently always false, silently skipping the whole real-FFI integration suite. Two hardening changes followed, both ABI-visible:

| Task | Location | Action | Status |
|------|----------|--------|--------|
| Correlation by value | `tw-ffi` | `tw_event_callback` takes `event_type` and `request_id` as arguments; the payload is documented as call-scoped and no longer load-bearing | **Done** |
| No host code under lock | `tw-ffi` | Events are collected under `SESSION` and forwarded after release, so a host may re-enter any export from its callback | **Done** (`crates/tw-ffi/tests/r1_callback_reentrancy.rs`) |

---

## R2 — Architecture debt

**Goal:** One engine path, one mutation surface, decomposed UI, model ready for F04–F28.

### R2.1 Document model vocabulary

| Task | Action | Status |
|------|--------|--------|
| Expand enums | Add `RunContent` variants: `Hyperlink`, `Field`, `InlineImage`, `FootnoteRef`, `CommentRef`, `Bookmark` (placeholder render OK) | **Done** |
| Expand blocks | Add `ShapeBlock`, header/footer typed maps per [document-model.md](architecture/document-model.md) | **Done** |
| `#[non_exhaustive]` | Public enums in `tw-model` to allow forward-compatible extension | **Done** |

**Exit:** DOCX import records element retention counts; failing test lists missing OOXML types — **Done** (`crates/tw-docx/tests/r2_document_model_vocabulary.rs`).

### R2.2 Single text storage

| Task | Action | Status |
|------|--------|--------|
| Pick one store | **Recommendation:** `String` per run; delete per-run `Rope` in `TextBuffer` OR make rope sole store | **Done** |
| Layout cache | `Arc<Document>` + version; no deep clone per edit | **Done** |
| Read path | All queries (`format_at`, hit test) on versioned snapshot | **Done** |

**Exit:** Property test: 10k random edits; run text == buffer text == export plaintext — **Done** (`crates/tw-edit/tests/r2_single_text_storage.rs`).

### R2.2 CI gates

Added to `.github/workflows/ci.yml`:

```yaml
- cargo test -p tw-edit --test r2_single_text_storage
- cargo test -p tw-core --test r2_layout_cache_arc
```

Status: **Done**

### R2.3 Command-only mutations

| Task | Action | Status |
|------|--------|--------|
| Collapse `BridgeCommand` | Worker handles only `ApplyEdit { command: Command }` + lifecycle commands | **Done** |
| Remove shortcuts | Migrate `ApplyHeading1`, `InsertTable`, … to `Command` variants | **Done** |
| Import purity | Format crates return `Document`; only `tw-edit` holds `&mut Document` in production | **Done** |

**Exit:** `rg 'BridgeCommand::ApplyHeading'` returns zero; undo works for all migrated paths — **Done** (`crates/tw-core/tests/r2_command_only_mutations.rs`).

### R2.3 CI gates

Added to `.github/workflows/ci.yml`:

```yaml
- cargo test -p tw-core --test r2_command_only_mutations
- |
  if rg 'BridgeCommand::ApplyHeading|BridgeCommand::InsertTable|BridgeCommand::ApplyBullet|BridgeCommand::ApplyNormal|BridgeCommand::InsertImage|BridgeCommand::InsertPageBreak|BridgeCommand::AcceptAll|BridgeCommand::RejectAll' crates/*/src app/lib --glob '!**/tests/**'; then
    echo "BridgeCommand edit shortcuts must not appear in production code"
    exit 1
  fi
```

Status: **Done**

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

- `editor_controller.dart` &lt;800 lines — **Done** (558 lines)
- CI builds `libtw_ffi` and runs Flutter tests against real engine — **Done**
- P0 count in [ui-functionality-audit.md](ui-functionality-audit.md) = 0 — **Done**

**Status:** **Done** (2026-08-07)

### R2.5 FFI consolidation

| Task | Action | Status |
|------|--------|--------|
| Codegen | JSON command dispatch (`tw_dispatch`) as FRB-equivalent for edit/format APIs | **Done** |
| Dispatch | `tw_dispatch(command_bytes)` for all `Command` edits | **Done** |
| Stable C ABI | Narrow primary surface: lifecycle, dispatch, display list, atlas, query JSON | **Done** |

**Exit:** New `SetCharFormat` field requires zero manual Dart typedef additions — **Done** (`format_codec.dart` dynamic JSON).

**Status:** **Done** (2026-08-07)

### R2.6 Undo transactions

| Task | Action | Status |
|------|--------|--------|
| `Transaction` type | Group commands; capture selection; support abort without stack pollution | **Done** |
| Coalescing | Merge consecutive `InsertText` same run within 1 s / word boundary | **Done** |
| Total inverse | `inverse()` returns `Result`, not `Option` | **Done** |

**Exit:** 100-char word types as one undo step; batch failure rolls back atomically — **Done** (`crates/tw-edit/tests/r2_6_undo_transactions.rs`).

### R2.6 CI gates

Added to `.github/workflows/ci.yml`:

```yaml
- cargo test -p tw-edit --test r2_6_undo_transactions
```

Status: **Done**

---

## R3 — Platform and fidelity

**Goal:** Honest multi-platform posture; Word fidelity gates. *(CRDT / F20 collaboration is tracked separately in [crdt-program.md](crdt-program.md).)*

### R3.1 Executor abstraction (web)

| Task | Action | Status |
|------|--------|--------|
| `EngineExecutor` trait | `ThreadedExecutor` (native), `InlineExecutor` (WASM) | **Done** |
| `tw-core` | No direct `std::thread::spawn` in session constructor | **Done** |
| CI | `cargo check --target wasm32-unknown-unknown -p tw-wasm` | Pending (`tw-core` and `tw-shape` check clean) |
| Fonts | Byte-slice font provider; no `fontdb/fs` on WASM | **Done** |

**Exit:** `tw-wasm` compiles in CI; smoke test opens bytes in JS test harness.

#### Execution model

`Session` owns the protocol — commands in, correlated events out — but no longer
owns the thread the engine runs on. That is a policy behind `EngineExecutor`:

| | `ThreadedExecutor` | `InlineExecutor` |
|---|---|---|
| Constructor | `Session::new_threaded()` (native default) | `Session::new_inline()` (default on `wasm32`) |
| Engine location | dedicated OS thread, blocking `recv` when idle | the caller's thread; no thread at all |
| `submit` backpressure | blocking `send` on the 512-deep queue | queues, and drives the engine once the queue reaches 512 |
| `drive()` | `0` — the worker thread does the work | runs every queued command, flushes the event backlog, then one background reflow chunk |
| Correlated waits | sleep until the deadline | drive until the response lands or the engine reports itself idle |

`ThreadedExecutor` is `#[cfg]`-compiled out on `wasm32`, so a web build contains
no reachable `std::thread::spawn` or `std::thread::sleep` — verified against the
emitted `wasm32` rlib, which carries neither instantiation while the native one
carries both.

**Driving an inline session.** Nothing runs until the host drives it, and
`Session::pump_events` is the driver: it executes queued work and then delivers
events to the observer, which is exactly what the FFI/Dart timer pump already
does. `poll_event`, `wait_for_response` and `wait_for_event` drive as well, so a
correlated wait cannot deadlock. Those waits are bounded by *work* rather than by
a clock, because `Instant::now` is unusable on `wasm32-unknown-unknown`: an
inline `wait_for_response` ignores its `timeout` and returns `Timeout` as soon as
the engine goes idle without having produced the response.

**Background reflow without an idle thread.** The worker thread ran forward
relayout chunks whenever the command channel was empty. Inline, "idle" is
redefined as *the end of a drive*: once a drive has emptied the command queue it
runs exactly one chunk. Preemption is preserved — a command queued before the
chunk is always served first — and one chunk per drive keeps a frame-timer host
responsive while successive pumps walk the pending window to zero. A host that
stops pumping freezes catch-up instead of getting it in the background;
`is_page_stale` keeps reporting the truth throughout.

Regression coverage is `crates/tw-core/tests/r3_inline_executor.rs` — 12 tests on
the native host, each behind a watchdog so a re-introduced block fails CI rather
than hanging it.

**Known gap:** `tw-edit`'s undo coalescing clock (`UndoStack::now` →
`Instant::now`) sits on the `ApplyEdit` path and panics on
`wasm32-unknown-unknown`. Compilation is unaffected; the first edit at runtime is
not. This needs a platform clock abstraction before a JS smoke test can type.

#### Fonts

`FontDatabase::empty()` / `TextShaper::with_injected_fonts()` /
`LayoutEngine::with_injected_fonts()` build an engine whose only faces come from
`register_face(&FontFaceSpec, bytes)` or `register_font_data(bytes)`. The
system-font constructors (`new()`) are unchanged, so native behaviour and every
existing caller are untouched.

`fontdb`'s `fs` feature is dropped on `wasm32` through a target-specific
dependency in `crates/tw-shape/Cargo.toml`, which deletes `load_system_fonts`,
`Source::File`, and `Source::SharedFile` from the build. A filesystem font path
on the web is a compile error rather than a silent runtime miss.

An unregistered family resolves through family → Office aliases → theme minor
font → any registered face in the nearest style. With no faces at all, layout
emits glyph-free lines instead of panicking. Details and the minimum font set a
web host must supply are in [layout-engine.md](architecture/layout-engine.md).

Covered by `crates/tw-shape/tests/injected_fonts.rs` and
`crates/tw-layout/tests/injected_font_layout.rs`; the latter lays out a document
from injected bytes alone and asserts real advances and glyph ids. CI still needs
`cargo check -p tw-shape --target wasm32-unknown-unknown`.

### R3.2 Platform CI matrix

| Target | Minimum CI |
|--------|------------|
| Linux | Existing + `libtw_ffi.so` build |
| macOS | Build + Flutter test job |
| Windows | Cross-check or scheduled job |
| Android | `scripts/build-ffi.sh android` + jniLibs verify |
| iOS | `scripts/build-ffi.sh ios` + XCFramework verify |
| WASM | `wasm32` check |

### R3.3 DOCX fidelity hardening

| Task | Action |
|------|--------|
| Element retention | Corpus test counts OOXML element types before/after round-trip |
| Tier B gating | Editing styles/numbering forces dependent part re-serialize or dirty flag |
| Within-part preserve | Token-preserving transforms for modified `document.xml` (Terra finding) |
| Vector in model | Move `resvg` rasterization out of `tw-docx` into `tw-render` |

### R3.4 CRDT / collaboration

**Moved to [crdt-program.md](crdt-program.md).** Not part of R3 remediation exit criteria. Implement with F20 (Wave W5, Stage S4).

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
- Build F20 collaboration on the current `tw-crdt` stub — see [crdt-program.md](crdt-program.md)

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
| R1 | **Done** | 2026-08-06 |
| R2 | **Done** | 2026-08-07 |
| R3 | Not started | |

---

## Review provenance

Findings synthesized from independent adversarial reviews (2026-08-06):

- Opus — code-level defects (atlas duplication, FFI UB, triple text store, model gap)
- Composer — CI blind spots, `BridgeCommand` duplication, memory math
- Grok — dual pipeline, CRDT stub, accessibility
- Terra — layout epochs, DOCX within-part loss, plugin cycle

Architecture brief: explore pass over repo layout and docs.
