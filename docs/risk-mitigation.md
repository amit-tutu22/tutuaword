# Word Compatibility Risk Mitigation

Single source of truth for **how we minimize Microsoft Word–class engineering risk**. Use this when roadmap, phase plan, long-tail gaps, or product MVP staging disagree — **this document wins until updated**.

Related: [roadmap.md](roadmap.md), [phase-plan.md](phase-plan.md), [feature-phases.md](feature-phases.md) (F01–F28 ↔ S0–S5 crosswalk), [architecture/docx-compatibility.md](architecture/docx-compatibility.md), [testing-strategy.md](testing-strategy.md), [long-tail-gaps.md](long-tail-gaps.md), ADR-0004 / 0007 / 0008.

---

## Nine-layer stack ↔ S0–S5 crosswalk

Canonical delivery order (see [roadmap.md](roadmap.md)):

| Layer | Delivery stages | Primary risk gates |
|-------|-----------------|-------------------|
| L1 DOCX compatibility | S1 continuous | Corpus import, Tier A round-trip, passthrough survival |
| L2 Layout | S1–S2 | Word screenshot baselines; HF/sections/floats |
| L3 Editing | S0–S2 | Command inverse; clipboard DOCX fragments |
| L4 Tables / images | S1–S2 | Nested tables, gridSpan/vMerge, PNG/JPEG/SVG |
| L5 Review | S2–S3 | TC ladder (c); spell suggestions; comment UI |
| L6 Automation API | After L5 UAT | F26.S4 schema semver; headless CLI |
| L7 Macro preservation | Tier C with L1 | VBA parts survive edit + save; never execute |
| L8 Enterprise policies | P6 / post-L6 | IRM passthrough; policy hooks on API/plugins/AI |
| L9 VBA execution | **Deferred** | Strategy D (preserve-only) until explicit business need |

**Rules:** S0–S5 describe *Word fidelity* staging (layers 1–5). Layers 6–9 are product surfaces that depend on S1–S2 stability but are not substitutes for DOCX/layout gates.

---

## Fidelity SLA (one bar)

| Metric | Target | How measured |
|--------|--------|--------------|
| **Open without crash** | 100% of corpus | Import returns `Ok` (or classified reject: password, corrupt) |
| **Tier A model identity** | 100% of Tier A fields after round-trip | Import → export → re-import model diff |
| **Visual vs Word** | Per-category pass rates below (not a single “95%/99%” slogan) | Word (fixed version) page screenshots vs our software render; ≤2% pixels *or* ≤2 px max displacement — pick one per suite and document it |
| **Passthrough survival** | 100% Tier B/C bytes when part unmodified | Package byte compare on untouched parts |

**External messaging:** Prefer “per-category Word visual pass rates + lossless Tier A round-trip.” Avoid competing “95%” vs “99%” claims until the corpus and Word baselines exist.

---

## Staging reconciliation

Two narratives existed: *engine-first* (`phase-plan.md`) and *Word-first MVP* (product). Reconciled delivery stages:

| Delivery stage | Goal | Maps to roadmap | Must include |
|----------------|------|-----------------|--------------|
| **S0 — Engine slice** | Type / format / undo / native save / paint | Phase 1 | Command path (ADR-0007), display list |
| **S1 — Word-open MVP** | Open real DOCX, edit text/styles/tables/common images, save without loss of unknown parts | Late Phase 2 → Phase 3 start | DOCX import + passthrough export; PNG/JPEG/SVG raster path; tables; lists; continuous corpus gate |
| **S2 — Word-edit maturity** | Pagination, floats (square), headers/footers, print preview, PDF *positions*, TC display + accept/reject | Phase 2–3 | Golden + Word baselines; TC ladder (a)–(c); PDF font embedding before “visual match” exit |
| **S3 — Product surface** | Comments UI, AI assistant, performance hardening | Phase 4 + comment model earlier | Comment/hyperlink model can land in S2 data layer |
| **S4 — Multiplayer** | CRDT, presence, offline sync | Phase 5 | Only after TC accept/reject is solid |
| **S5 — Long-tail** | EMF/WMF quality, OLE interaction, binary `.doc`, advanced DrawingML, accessibility tree | Phase 3+ / 6 / deferred | Explicit residual risk; not MVP blockers |

**Rules**

1. **DOCX is on the critical path from S1** — not “polish after layout is done.” Layout hardens *against* a seeded Word corpus.
2. **Self-goldens are regression only.** Word (or fixed Word-version) screenshots are the fidelity gate.
3. **Exit criteria that `long-tail-gaps.md` admits unmet must be downgraded or blocked** — never left as green gates.
4. **ODT / MD / HTML / RTF** do not block the DOCX exit for S1/S2.

---

## Risk → mitigation matrix

| Risk | Sev | Mitigation | Owner docs / ADRs | Delivery | Residual risk (honest) |
|------|-----|------------|-------------------|----------|------------------------|
| **DOCX / Open XML** | Critical | Tier A/B/C + OPC **package passthrough** (patch modified parts only) | ADR-0008, `docx-compatibility.md` | S1 continuous | Editing Tier B parts (styles/numbering) can desync model vs preserved XML if not gated |
| **Layout fidelity** | Critical | Rust layout engine; keep/widow/page rules; regression goldens **plus** Word baselines | `layout-engine.md`, `testing-strategy.md` | S1–S2 | 1 px drift can reflow pages; Knuth-Plass / full Word parity deferred |
| **Fonts & metrics** | Critical | Theme + doc defaults → `fontdb`; Office aliases; optional embedded fonts; fixed-font CI | ADR-0004, `layout-engine.md` | S1–S2 | Metric-incompatible fallbacks still reflow vs Word; OS font sets differ |
| **Complex scripts** | Critical | rustybuzz + bidi + segmenters; HarfBuzz feature-flag fallback | ADR-0004 | S2 gate | rustybuzz may lag HarfBuzz; fallback not default until corpus fails |
| **Tables** | Critical | Dedicated table layout; gridSpan/vMerge; borders/shading; nested later | `layout-engine.md`, model/table | S1–S2 | AutoFit / complex `tblStyle` bands / nested pagination incomplete |
| **Floating objects** | Critical | Square wrap MVP; preserve DrawingML/VML bytes; Tight/Through later | `layout-engine.md` | S2 square / S5 advanced | Text boxes, tight wrap, many anchors under-specified |
| **Numbering / lists** | Critical | Parse `numbering.xml` (`lvlText`, start, level `rPr`); promote **edit round-trip to Tier A gate** | `docx-compatibility.md` | S1–S2 | Multilevel restarts / overrides / legal numbering long-tail |
| **Styles / themes** | Critical | StyleSheet resolution + theme fonts; raw preserve for unknown | `docx-compatibility.md` | S1–S2 | Same Tier B desync risk as numbering when editing |
| **Headers / footers / sections** | High | Parse HF blocks; section format in model; even/odd/first later | layout + docx | S2 | Linked sections, columns, per-section page numbers incomplete |
| **Track Changes** | Critical | See [TC maturity ladder](#track-changes-maturity-ladder) | model revisions, roadmap, long-tail | (a)–(c) by S2; (d) S4 | CRDT + revisions interaction not designed yet |
| **Comments** | High | Model + passthrough early; threaded UI with collab | long-tail, collaboration.md | Data S2 / UI S3–S4 | Mentions / modern comment parts Tier C until parsed |
| **EMF / WMF** | Critical | **S1–S2:** preserve bytes + grey/placeholder or pre-raster if present; **S5:** native/convert quality path | `docx-compatibility.md`, rendering.md | S5 for quality | No reliable pure-Rust EMF decoder today |
| **SVG** | High | Rasterize at import (`resvg`) where possible; else placeholder | `tw-docx` image convert | S1 | Complex SVG / Word DrawingML hybrids |
| **OLE / embeddings** | Critical* | Tier C preserve-only; no edit/execute | docx-compatibility | Forever until S5+ | Users see placeholder; bytes survive save |
| **Legacy `.doc`** | Critical* | **Out of scope** — convert externally | vision, docx-compatibility | — | Explicit non-goal |
| **RTF** | High | Import-only, best-effort after DOCX exit | file-formats | After S2 | Not a compatibility pillar |
| **Printing** | Critical | Print preview + **font-embedded PDF** as print path; OS spooler later | roadmap, long-tail §PDF | Preview S2; WYSIWYG after PDF fonts | Preview ≠ OS print; DPI/duplex later |
| **PDF export** | Critical | Split gates: (1) glyph positions from layout (2) embedded fonts + images | long-tail §1, `tw-pdf` | (1) now / (2) blocks “visual match” | Helvetica-12 path must not be called Word-match |
| **Performance** | Critical | Page-granular invalidation; budgets; lazy/visible-first for large docs | `performance-budgets.md` | Continuous | 500-page &lt;2s is aspirational until benches gate CI |
| **Clipboard** | High | Glyph-mode cut/copy/paste; native &gt; DOCX fragment &gt; HTML &gt; plain | text-engine, UI audit | S1–S2 | DOCX clipboard interop incomplete |
| **Undo / redo** | Critical | Single `Command` mutation path | ADR-0007 | S0 | AI/plugin/remote ops must use Commands |
| **Collaboration** | High | Yjs + Command translator; after TC accept/reject | ADR-0007/0009 | S4 | Tree↔Yjs + TC semantics thin until prototype |
| **AI** | Medium | Provider abstraction; deterministic edits via Commands | ADR-0010 | S3 | Hallucination/privacy/cost — product controls |
| **Accessibility** | High | Parallel semantic tree + platform hooks (custom glyph paint cannot rely on Flutter text a11y) | ADR-0003 gap | S5 | Currently unplanned in roadmap — must be scheduled |
| **Cross-platform** | High | Pure-Rust shape/layout; Flutter UI; same display list | ADR-0002/0003/0004 | Continuous | Font availability and print/IME still diverge |
| **Testing underestimation** | Critical | Unit + fingerprint regression + **Word baselines** + corpus + fuzz + perf benches in CI | testing-strategy | Continuous | Docs historically overstated PNG/corpus readiness — track in long-tail |

\*Critical if claimed as supported; mitigated by **honest non-support** or preserve-only.

---

## Track Changes maturity ladder

One ladder — stop assigning accept/reject to multiple phases at once.

| Step | Capability | Delivery |
|------|------------|----------|
| **(a) Preserve** | Markup survives import/export (passthrough / model) | S1 |
| **(b) Display** | Insertions/deletions visible with author/time | S2 |
| **(c) Accept / reject** | Commands mutate model; UI enabled | **Before S4** (target end of S2) |
| **(d) Collab-safe review** | TC + CRDT semantics documented + tested | S4 |

---

## Testing gates (what actually reduces risk)

| Gate | Purpose | Status expectation |
|------|---------|-------------------|
| Unit / command inverse | Undo correctness | Required in CI |
| Layout fingerprints / self-goldens | Catch accidental layout regressions | Required in CI; **not** Word fidelity |
| Word screenshot baselines | Layout looks like Word | Required for S2 exit; seed from S1 |
| DOCX corpus import | Opens real files | Grow continuously; categories below |
| Round-trip Tier A / passthrough | No silent loss | Required for S1+ |
| Perf benches | Typing / open budgets | Blocking earlier than “P3+ future” |
| Fuzz malformed DOCX | Crash resistance | Nightly → then CI |

### Corpus categories (minimum emphasis)

| Category | Why |
|----------|-----|
| Styles + themes | Reflow and formatting identity |
| Multilevel numbering | Highest edit desync risk |
| Tables (merged, styled) | Mini layout engine |
| Images PNG/JPEG/SVG + EMF placeholders | Media path |
| Headers/footers + sections | Pagination |
| Floats (square first) | Anchor/wrap |
| Track changes | Revision ladder |
| Complex scripts | Shaping honesty |
| Real-world business/legal | Long-tail OOXML |
| Edge (empty, huge, corrupt) | Robustness |

---

## Phase doc sync checklist

When changing this file, update these to match (or link here):

- [ ] [roadmap.md](roadmap.md) Risk Register + PDF / TC exit wording  
- [ ] [phase-plan.md](phase-plan.md) — note S1 DOCX-on-critical-path  
- [ ] [architecture/docx-compatibility.md](architecture/docx-compatibility.md) — 95%/99% → link SLA above; EMF timing → S5 quality  
- [ ] [testing-strategy.md](testing-strategy.md) — distinguish fingerprint vs Word PNG  
- [ ] [long-tail-gaps.md](long-tail-gaps.md) — PDF/TC rows stay the honesty layer for *code* status  

---

## Top actions (living)

1. **Word baselines + DOCX corpus on the critical path** (S1) — F23.S1: ≥50 gate fixtures, ≥95% open, Tier A category + roundtrip-50 CI gates landed; Word PNG baselines still open (S2).  
2. **Honest exits:** PDF `VisualMatch` / `embed_fonts` embed layout faces (`/FontFile2`); structural Helvetica remains the default ship path. Numbering `lvlText`/`start`/level `rPr` Tier A round-trip gated.  
3. **TC ladder (c):** Accept/Reject all commands + Review ribbon wired; per-change nav still open.  
4. **Keep this matrix owned** — one staging story (S0–S5) for product and engineering.

---

## Strengths we rely on

- OPC passthrough (ADR-0008) for unknown OOXML survival  
- Tier model for render vs preserve vs skip  
- Single Command path (ADR-0007) for undo → AI → CRDT  
- Honest shaping tradeoff (ADR-0004) with HarfBuzz escape hatch  
- `long-tail-gaps.md` as contradiction detector for shipped code vs docs  
