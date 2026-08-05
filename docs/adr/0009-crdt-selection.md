# ADR-0009: CRDT Selection (Yjs)

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 5

## Context

Real-time collaboration requires a Conflict-free Replicated Data Type (CRDT) so that concurrent edits from multiple users merge without conflicts. The CRDT choice affects offline support, binary encoding efficiency, ecosystem maturity, and integration complexity with our document model.

Requirements: offline editing with merge-on-reconnect, efficient binary encoding, cursor/presence awareness, Rust bindings, proven at scale.

## Decision

Use **Yjs** (via the `yrs` Rust crate) as the CRDT engine.

Integration approach:
- Yjs document mirrors the document model structure
- Local edits translate to Yjs operations via `tw-crdt::CrdtTranslator`
- Remote Yjs operations translate back to `Command` enums (ADR-0007)
- Yjs Awareness protocol handles live cursors and presence
- Sync via WebSocket (cloud) or WebRTC (P2P)

## Consequences

**Positive:**
- Battle-tested — used by Notion, Figma, JupyterLab, and hundreds of other products
- Efficient binary encoding via lib0 (typically <100 bytes per edit operation)
- Built-in Awareness protocol for cursors and presence — no custom implementation needed
- `yrs` crate provides native Rust bindings (no C FFI)
- Offline support with automatic merge on reconnect
- Large ecosystem of sync providers (y-websocket, y-webrtc, y-indexeddb)

**Negative:**
- Yjs is optimized for flat/shared text, not tree-structured documents — requires a mapping layer
- Document model → Yjs mapping adds complexity to `tw-crdt`
- Yjs document size grows with edit history (garbage collection needed for long sessions)
- Dependency on external project for a core feature

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **Automerge** | Document model mapping is more complex; smaller ecosystem; Rust bindings less mature |
| **Custom CRDT** | Multi-year effort to build and prove correct; unnecessary when mature libraries exist |
| **Operational Transform (OT)** | Requires central server for ordering; poor offline support; Google Docs uses OT but with massive infrastructure |
| **Diamond Types** | Optimized for text, not tree structures; less ecosystem support |
| **Loro** | Newer, less battle-tested; smaller community; Rust-native but unproven at scale |

## Mapping Strategy

The document model tree maps to Yjs shared types:

| Model Element | Yjs Type |
|---------------|----------|
| Document sections | `YArray` of section maps |
| Section blocks | `YArray` of block maps |
| Paragraph runs | `YArray` of run maps |
| Run text content | `YText` |
| Formatting properties | `YMap` on each node |
| Node IDs | `YMap` key (`tw:nodeId`) |

This mapping is maintained by `tw-crdt` and is transparent to `tw-edit` and the UI.
