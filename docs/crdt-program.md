# CRDT / Collaboration Program

**Status:** Planned (separate from architecture remediation)  
**Authority:** Implements F20 (Collaboration) in [feature-phases.md](feature-phases.md). Not part of the R0–R3 remediation gates in [architecture-remediation.md](architecture-remediation.md).

Related: [ADR-0007](adr/0007-single-mutation-path-for-undo-and-crdt.md), [ADR-0009](adr/0009-crdt-selection.md), [collaboration.md](architecture/collaboration.md), [risk-mitigation.md](risk-mitigation.md) (S4 staging).

---

## Scope

Real-time multi-user editing (F20) is **out of scope for R3 remediation**. Platform work (WASM, CI, DOCX fidelity) proceeds independently. CRDT foundations, sync, presence, and collaboration UI are tracked here and implemented when F20 is scheduled (Wave W5, Stage S4).

**Do not build F20 on the current `tw-crdt` stub** until the foundations below exit.

---

## Prerequisites (from remediation + TC ladder)

| Prerequisite | Why |
|--------------|-----|
| R1 performance shape | Collab multiplies edit rate; O(whole-doc) relayout is unusable |
| R2.3 command-only path | Remote ops must translate to the same `Command` enum as local edits |
| R2.6 undo transactions | Local undo semantics must be documented before collab undo |
| Track-changes accept/reject solid (S2) | S4 collab per [risk-mitigation.md](risk-mitigation.md) |

---

## Program phases

### C1 — Operation metadata

| Task | Action |
|------|--------|
| Origin | `origin: Local \| Remote` on applied commands |
| Operation IDs | Stable `op_id` for correlation and replay |
| Transactions | Boundaries aligned with `tw-edit::Transaction` |

**Exit:** Every `ApplyEdit` path records origin; remote commands never generate outbound CRDT ops.

### C2 — Position model

| Task | Action |
|------|--------|
| CRDT positions | Prototype positions that survive concurrent inserts (not bare `(run_id, usize)`) |
| Mapping | `NodeId` ↔ Yjs anchors per [ADR-0009](adr/0009-crdt-selection.md) |
| Selection | Caret/selection expressed in CRDT-safe coordinates for merge |

**Exit:** Two simulated clients insert at the same offset; merged document has no torn runs.

### C3 — Vertical slice (`tw-crdt` + `yrs`)

| Task | Action |
|------|--------|
| Translator | `tw-crdt::local_to_crdt` / `crdt_to_local` per [collaboration.md](architecture/collaboration.md) |
| Demo | Two clients, paragraph text + bold, merge via `yrs` |
| Wire | Binary Yjs update encoding over test transport |

**Exit:** ADR-0009 marked **Partial** with a working 2-client demo (no production sync backend required).

### C4 — Undo under collaboration

| Task | Action |
|------|--------|
| Semantics | Document selective/local undo; inverse-stack alone is insufficient |
| Remote ops | Applied without polluting local undo stack (or with explicit policy) |
| Tests | Merge + undo/redo scenarios |

**Exit:** Documented undo policy in ADR-0007; integration tests for local undo after remote edit.

### C5 — Production collab (F20)

Deferred to [feature-phases.md](feature-phases.md) F20 sub-features: presence, comments sync, suggestions, offline, cloud backend.

---

## Crate boundaries

Per [crate-map.md](architecture/crate-map.md):

- `tw-crdt` — Yjs translation, conflict-free merge, **depends on `tw-edit`** (not `tw-core` directly for mutations)
- `tw-core` — orchestrates session; applies remote `Command`s through existing worker path
- No format crate (`tw-docx`, etc.) depends on `tw-crdt`

---

## CI gates (when program starts)

```yaml
# Example — add when C3 lands
- cargo test -p tw-crdt
- cargo test -p tw-core --test crdt_two_client_merge  # TBD
```

---

## Tracking

| Phase | Status | Exit date |
|-------|--------|-----------|
| C1 | Not started | |
| C2 | Not started | |
| C3 | Not started | |
| C4 | Not started | |
| C5 (F20) | Not started | |

Update this document when a phase exits. Link PRs to phase IDs (`C2-position-model`, etc.).
