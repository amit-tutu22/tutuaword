# ADR-0011: Architecture Remediation Program

**Status:** Accepted  
**Date:** 2026-08-06  
**Phase:** 1 (blocks Phase 2 feature expansion)

## Context

A four-model adversarial architecture review (2026-08-06) found that the **documented architecture is sound** but the **implementation violates load-bearing ADRs** in performance-critical areas:

- ADR-0005 page-granular invalidation and non-blocking UI are not implemented
- Display lists embed a full glyph atlas per page (~16 MB × page count per keystroke)
- FFI uses synchronous busy-wait on the Dart UI isolate
- The document model supports only a fraction of Word content types required by F04–F28
- Web (`tw-wasm`) cannot compile because `tw-core` hard-codes OS threads

Continuing to add features (F04+) on this foundation increases refactor cost superlinearly and produces a product that looks feature-rich in the engine but cannot perform at Word scale.

## Decision

Adopt a **sequential remediation program** (R0–R3) documented in [architecture-remediation.md](../architecture-remediation.md).

**Rules:**

1. **R0 (safety) and R1 (performance shape) must exit before F04+ feature work begins.** F01–F03 may continue only as bug fixes and UI wiring that do not add FFI exports or expand `BridgeCommand`.
2. **No new typed `tw_*` C exports** after this ADR except lifecycle/debug. New edit operations go through `Command` serialization (R2.5).
3. **No marketing or docs claim** of web, mobile, or collaboration parity until the corresponding exit criteria pass (collaboration: [crdt-program.md](../crdt-program.md), not R3).
4. **ADR invariants become CI gates** where mechanically verifiable (wasm compile, perf fixtures, P0 audit count, dependency lint).
5. **Remediation wins over feature-phases waves** when they conflict until R1 exits; then [feature-phases.md](../feature-phases.md) wave order resumes.

### Phase summary

| Phase | Focus | Gate |
|-------|-------|------|
| **R0** | FFI safety, request correlation, no silent command drops | Fuzz + stress tests green |
| **R1** | Incremental layout, atlas separation, per-page snapshots, async FFI | 50-page p99 latency + memory budget |
| **R2** | Model vocabulary, single text store, Command-only path, UI decomposition | P0 audit = 0; engine-backed Flutter CI |
| **R3** | WASM executor, platform CI, DOCX retention tests | `wasm32` CI green; platform matrix + fidelity gates |

CRDT / F20 collaboration is **not** an R3 gate. See [crdt-program.md](../crdt-program.md).

Full task breakdown: [architecture-remediation.md](../architecture-remediation.md).

## Implementation status

| Phase | Status | Date |
|-------|--------|------|
| R0 | **Done** | 2026-08-06 |
| R1 | **Done** | 2026-08-06 |
| R2 | **Done** | 2026-08-07 |
| R3 | Pending | — |

**F04+ feature work is unblocked** as of R1 exit; **R2 architecture debt is cleared** (2026-08-07). Wave order in [feature-phases.md](../feature-phases.md) resumes; R3 covers platform and DOCX fidelity; collaboration is [crdt-program.md](../crdt-program.md).

## Consequences

**Positive:**

- Prevents compounding technical debt during F04–F28
- Restores trust between ADRs and code
- Unblocks web, mobile, and collaboration on a proven performance base
- Gives reviewers and contributors a single ordered backlog

**Negative:**

- Feature velocity pauses 4–10 weeks for R0–R1 (estimated)
- Display list wire format v3 changes when atlas is separated (Flutter + Rust must migrate together)
- Some in-flight engine APIs may need rework when `BridgeCommand` collapses to `Command`

## Rejected alternatives

1. **Continue feature-first, fix later** — rejected; review consensus: cost grows superlinearly (model enums, FFI surface, dual UI pipeline).
2. **Big-bang rewrite** — rejected; incremental R0→R3 preserves working DOCX slice and test corpus.
3. **Drop Rust layout, use Flutter Paragraph** — rejected; contradicts ADR-0003 and DOCX fidelity strategy.

## References

- [architecture-remediation.md](../architecture-remediation.md) — full remediation plan
- [crdt-program.md](../crdt-program.md) — CRDT / F20 program (separate from R3)
- ADR-0003, ADR-0005, ADR-0007, ADR-0008, ADR-0009
- [risk-mitigation.md](../risk-mitigation.md) — S0–S5 staging
- [ui-functionality-audit.md](../ui-functionality-audit.md) — P0 gate for R2
