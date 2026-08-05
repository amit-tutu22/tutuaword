# Phase-Wise Implementation Plan

Consolidated phase plan mapping architecture documentation, ADRs, crates, and exit criteria. See also [roadmap.md](roadmap.md) and [README.md](README.md).

## Phase 0: Architecture (Complete)

All documentation under `docs/` — vision, roadmap, 13 module specs, 10 ADRs, performance budgets, testing strategy.

## Phase 1: Editor Foundation (Months 1–3)

**Crates:** `tw-model`, `tw-text`, `tw-edit`, `tw-shape`, `tw-layout`, `tw-render`, `tw-native`, `tw-core`, `tw-ffi`, Flutter app

**ADRs:** 0001, 0002, 0003, 0004, 0005, 0006, 0007

**Exit:** Type/format/undo/save/render vertical slice on single page.

## Phase 2: Word Processing (Months 4–6)

**Crates:** extend layout/model/edit/render; add `tw-pdf`

**Exit:** Pagination, tables, images, lists, PDF export, 60 FPS on 50 pages.

## Phase 3: Office Compatibility (Months 7–10)

**Crates:** `tw-docx`, `tw-odt`, `tw-markdown`, `tw-html`, `tw-rtf`

**ADR:** 0008 (DOCX package passthrough)

**Exit:** 95%+ DOCX corpus renders; 500-page open <2s.

## Phase 4: AI Integration (Months 11–14)

**Crates:** `tw-ai`

**ADR:** 0010

**Exit:** Hybrid AI layer (Automatic / Always Local / Always Cloud), llama.cpp desktop local + OpenAI/Gemini cloud adapters, ONNX/Apple FM mobile stubs, AI module marketplace.

## Phase 5: Collaboration (Months 15–18)

**Crates:** `tw-crdt`, sync backend

**ADR:** 0009

**Exit:** 2-user CRDT convergence, live cursors, offline sync.

## Phase 6: Enterprise (Months 19–24)

**Crates:** `tw-plugin`, security backend

**Exit:** SSO, WASM plugins, audit logs, on-prem Docker deploy.
