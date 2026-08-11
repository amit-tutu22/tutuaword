# ADR-0012: VBA Execution Strategy (Layer 9)

**Status:** Accepted — Strategy D (preserve-only)  
**Date:** 2026-08-10  
**Context:** [Layered Word parity roadmap](../roadmap.md), [docx-compatibility.md](../architecture/docx-compatibility.md)

## Decision

**Do not execute VBA macros.** Preserve `vbaProject.bin` and related OPC parts on DOCX import/export (Tier C passthrough per ADR-0008). Layer 9 (VBA execution/compatibility) remains **deferred** until an explicit business requirement overrides this ADR.

## Options considered

| Strategy | Description | Verdict |
|----------|-------------|---------|
| **A. Sandboxed VBA runtime** | Office-compatible subset in isolated VM | Rejected — very high cost; incompatible with WASM/web |
| **B. Static translation** | VBA → Commands / Rust plugins | Deferred — revisit only after F26.S4 automation API for named patterns |
| **C. External bridge** | Launch Word/LibreOffice for macro docs | Rejected — out of product scope |
| **D. Preserve-only** | Keep bytes; never run | **Accepted** through enterprise rollout |

## Consequences

- Users can open macro-enabled `.docm` files, edit body content, and save without stripping macros.
- UI must message: macros preserved; not executed.
- Enterprise policy (`tw-policy`) can deny opening macro documents via `open_macro_document`.
- Automation API checks VBA parts before `OpenDocx` when policy restricts macros.

## Revisit criteria

Re-open Layer 9 only if:

1. Customer contract requires macro execution parity, **and**
2. Security review approves chosen strategy (A or B), **and**
3. Layers 1–8 are stable (DOCX, layout, automation API, macro preservation, enterprise policies).

Until then, **Strategy D** is the canonical answer to “include VBA.”
