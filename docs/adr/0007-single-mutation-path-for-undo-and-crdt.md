# ADR-0007: Single Mutation Path for Undo and CRDT

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 1 (with CRDT integration in Phase 5)

## Context

Document mutations come from multiple sources: user typing, paste, formatting commands, AI suggestions, plugin edits, and (in Phase 5) remote CRDT operations. Each source must support undo/redo. When CRDT collaboration arrives, remote operations must integrate without a rewrite of the editing system.

If different code paths mutate the document model directly (UI calling model methods, parsers constructing models, AI applying changes), undo/redo becomes inconsistent and CRDT translation requires mapping multiple mutation styles.

## Decision

**Every document mutation goes through a single entry point:**

```rust
pub fn apply(session: &mut EditSession, command: Command) -> Result<EditResult, EditError>;
```

- All sources (UI, paste, AI, plugins, CRDT) produce `Command` enums
- `apply()` executes the command, records the inverse on the undo stack, and returns affected node IDs
- Undo pops the inverse command and calls `apply()` with it
- CRDT operations translate to/from `Command` via `tw-crdt::CrdtTranslator`

No code outside `tw-edit` may mutate the document model directly.

## Consequences

**Positive:**
- Undo/redo works identically regardless of mutation source
- CRDT integration (Phase 5) requires only a translation layer, not a rewrite
- All mutations are auditable — every change has a typed `Command` record
- AI and plugin edits are undoable with the same Ctrl+Z as typing
- Testable — verify correctness by applying commands and checking invariants

**Negative:**
- Every possible edit must be expressible as a `Command` variant — requires upfront design
- Complex operations (e.g., paste formatted content) decompose into many commands or one large command
- Command enum grows over time (mitigated by grouping: `TableCommand`, `ImageCommand` sub-enums)
- Performance overhead of command serialization for CRDT (acceptable — commands are small)

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **Direct model mutation with change log** | Change log captures diffs but not intent; undo must reverse diffs which is error-prone for complex operations |
| **Separate undo systems per source** | User expects one Ctrl+Z for everything; multiple undo stacks are confusing |
| **Event sourcing (store all events, rebuild state)** | Memory overhead for long editing sessions; replay time grows linearly |
| **CRDT as primary model (no separate model)** | CRDT data structures are not suited for complex document formatting; would require CRDT-per-field |
