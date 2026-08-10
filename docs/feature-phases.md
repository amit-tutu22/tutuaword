# Feature-Wise Phase Implementation Plan (F01–F28)

Product feature backlog contract: **one phase per capability area**, each split into **stages** with named **unit** and **integration** tests, crate ownership, and exit criteria.

**Related docs (do not duplicate):**

| Doc | Role |
|-----|------|
| [roadmap.md](roadmap.md) / [phase-plan.md](phase-plan.md) | Calendar phases P1–P6, team scale |
| [risk-mitigation.md](risk-mitigation.md) | Word fidelity staging S0–S5 (wins on conflicts) |
| [architecture-remediation.md](architecture-remediation.md) | R0–R3 engineering gates (R0–R1 **Done** 2026-08-06; F04+ unblocked; [ADR-0011](adr/0011-architecture-remediation-program.md)) |
| [testing-strategy.md](testing-strategy.md) | Test pyramid, CI, golden vs Word baselines |
| [long-tail-gaps.md](long-tail-gaps.md) | Honest *code* status for deferred work |
| [ui-functionality-audit.md](ui-functionality-audit.md) | Ribbon/menu wiring vs engine |

---

## Conventions

| Item | Rule |
|------|------|
| Phase IDs | `F01` … `F28` (fixed order below) |
| Stage IDs | `Fnn.S1`, `Fnn.S2`, … (MVP → polish → advanced) |
| Unit tests | `U-Fnn-Sx-short-name` in `crates/*/tests/` or `#[cfg(test)]` |
| Integration tests | `I-Fnn-Sx-short-name` in `app/test/` or `crates/tw-core/tests/` |
| Status | **Implemented** / **Partial** / **Stub** / **Missing** (baseline snapshot) |
| Edits | All mutations via `Command` in `tw-edit` ([ADR-0007](adr/0007-single-mutation-path-for-undo-and-crdt.md)) |
| Remediation | F04+ stages **unblocked** — [R1 exited](architecture-remediation.md#r1--performance-shape) 2026-08-06 ([ADR-0011](adr/0011-architecture-remediation-program.md)) |
| DOCX | Package passthrough + Tier A/B/C ([ADR-0008](adr/0008-docx-package-passthrough.md)) |

### Test spec template (used in each stage)

```text
U-Fnn-Sx-name
  crate: tw-edit | tw-docx | tw-layout | app
  arrange: …
  act: Command::… or import/export
  assert: model invariant; undo inverse; round-trip

I-Fnn-Sx-name
  path: user action in Flutter or tw-core worker
  assert: display list / save / reopen / status text
```

---

## Implementation waves (order ≠ F01→F28)

Build in **waves** so drawing/cloud features do not block the edit loop.

| Wave | Phases | Goal |
|------|--------|------|
| **W0** | F02, F03 (core), F23 | Trustworthy typing, format, DOCX/TXT |
| **W1** | F01, F04, F05, F06, F18 | Document shell, para/lists/styles, find |
| **W2** | F07, F08, F09, F10, F17, F19 | Layout, HF, tables/images, review, nav |
| **W3** | F15, F16, F25, F24 | Symbols, references, print, templates |
| **W4** | F11, F12, F13, F14 | Shapes/charts/equations/SmartArt (preserve-first) |
| **W5** | F28, F20, F21, F22, F26, F27 | AI, collab, a11y, security, plugins, cloud |

**Explicit non-goals:** binary `.doc` import (convert externally); VBA macro execution (preserve only).

---

## Cross-reference matrix

| Feature | Primary wave | Roadmap P | Risk S | Main crates / app |
|---------|--------------|-----------|--------|-------------------|
| F01 Document Management | W1 | P1–P2 | S1 | `tw-core`, `tw-ffi`, `editor_controller` |
| F02 Text Editing | W0 | P1 | S0 | `tw-edit`, `tw-text`, `glyph_editor_surface` |
| F03 Character Formatting | W0–W1 | P1–P2 | S1 | `tw-model`, `tw-edit`, `home_tab` |
| F04 Paragraph Formatting | W1 | P2 | S1–S2 | `tw-model`, `tw-layout`, `tw-edit` |
| F05 Lists | W1 | P2 | S1 | `tw-model/list`, `tw-docx`, `tw-layout` |
| F06 Styles | W1 | P2 | S1 | `tw-model/styles`, `tw-docx` |
| F07 Page Layout | W2 | P2 | S2 | `tw-model`, `tw-layout`, `layout_tab` |
| F08 Headers & Footers | W2 | P2–P3 | S2 | `tw-docx`, `tw-layout/engine` |
| F09 Tables | W2 | P2 | S1–S2 | `tw-model/table`, `tw-layout/tables`, `tw-edit` |
| F10 Images | W2 | P2–P3 | S1 | `tw-model/image`, `tw-docx`, `tw-layout` |
| F11 Shapes | W4 | P3+ | S5 | new `ShapeBlock`, `tw-docx`, `tw-render` |
| F12 Smart Objects | W4 | P3+ | S5 | passthrough Tier C |
| F13 Charts | W4 | P3+ | S5 | passthrough Tier C |
| F14 Equations | W4 | P3+ | S5 | OMML preserve → edit |
| F15 Symbols | W3 | P2–P3 | S1 | Flutter dialog, `InsertText` |
| F16 References | W3 | P3–P5 | S2–S3 | `tw-model`, `tw-docx`, `tw-layout` |
| F17 Review | W2 | P2–P5 | S2–S4 | `tw-edit`, `tw-spell`, `tw-docx` |
| F18 Search | W1 | P2 | S1 | `tw-edit`, find UI |
| F19 Navigation | W2 | P2 | S1 | `editor_controller`, model links |
| F20 Collaboration | W5 | P5 | S4 | `tw-crdt`, sync backend |
| F21 Accessibility | W5 | P6+ | S5 | semantic tree, Flutter Semantics |
| F22 Security | W5 | P6 | — | `security.md`, crypto |
| F23 File Formats | W0 | P1–P3 | S1–S2 | `tw-docx`, `tw-odt`, `tw-html`, `tw-pdf` |
| F24 Templates | W3 | P3 | S1 | `tw-model/theme`, template gallery |
| F25 Printing | W3 | P2 | S2 | `tw-pdf`, platform print |
| F26 Macros & Automation | W5 | P6 | — | `tw-plugin`, mail merge |
| F27 Cloud | W5 | P5–P6 | S4 | sync service |
| F28 AI | W5 | P4 | — | `tw-ai`, ADR-0010 |

### Phase completion rollup (2026-08-10)

| Phase | Baseline | Notes |
|-------|----------|-------|
| F01 Document Management | **Complete** | Lifecycle, export, protection |
| F02 Text Editing | **Complete** | S1–S4 |
| F03 Character Formatting | **Complete** | S1–S4 |
| F04 Paragraph Formatting | **Complete** | S1–S4 |
| F05 Lists | **Complete** | S1–S4 (custom bullet glyphs limited) |
| F06 Styles | **Complete** | S1–S4 |
| F07 Page Layout | **Complete** | S1–S4 |
| F08 Headers & Footers | **Complete** | S1–S4 |
| F09 Tables | **Complete** | S1–S5 |
| F10 Images | **Complete** | S1–S4 |
| F11 Shapes | **Complete** | S1–S3 (freeform pen deferred) |
| F12 SmartArt | **Complete** | S1–S3 preserve/insert; edit out of scope |
| F13 Charts | **Complete** | S1–S3 |
| F14 Equations | **Complete** | S1–S4 |
| F15 Symbols | **Complete** | S1–S3 |
| F16 References | **Complete** | S1–S4 |
| F17 Review | **Complete** | S1–S4 |
| F18 Search | **Complete** | S1–S4 |
| F19 Navigation | **Complete** | S1–S4 |
| F20 Collaboration | **Stub** | `tw-crdt` not implemented |
| F21 Accessibility | **Complete** | S1–S5 |
| F22 Security | **Complete** | S1–S4; S5 IRM deferred |
| F23 File Formats | **Complete** | S1–S4; binary `.doc` / EPUB deferred |
| F24 Templates | **Complete** | S1–S4 |
| F25 Printing | **Complete** | S1–S4 |
| F26 Macros & Automation | **Complete** | S1–S3 |
| F27 Cloud | **Missing** | S1–S4 not started |
| F28 AI | **Complete** | S1–S6 |

---

## F01 — Document Management

**Scope:** New, Open, Save, Save As, Auto Save, Recent documents, Password protection, Read-only mode, Print, Print Preview, Export PDF/HTML/ODT, Document properties.

### Baseline status: **Complete** (F01.S1–S4).

| Capability | Status | Evidence |
|------------|--------|----------|
| New | Implemented | File→New + `tw_new_document` FFI |
| Open / Save / Save As | Implemented | File menu; `.twdoc` + `.docx` Save As |
| Auto Save / Recent | Implemented | Timer autosave + File menu recent (max 10) |
| Password / Read-only | Implemented | Encrypted DOCX open/save (F22); DOCX protection flag |
| Print / Preview | Implemented | View toggle; paginated read-only preview |
| Export PDF | Implemented | structural PDF from File/Review menu |
| Export HTML / ODT | Implemented | Save As |
| Document properties | Implemented | File→Properties (view-only) |

**Dependencies:** F23 (formats), F25 (print).

### F01.S1 — Core file lifecycle

**Deliverables:** File→New; confirm Save/Save As paths for DOCX + native `.twdoc`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F01-S1-new-document-empty` | Unit | `EditSession::new()` → one section, one empty para |
| `U-F01-S1-save-roundtrip-twdoc` | Unit | `tw-native` serialize/deserialize identity |
| `I-F01-S1-open-docx` | Integration | Open sample DOCX → non-empty display list |
| `I-F01-S1-save-as-docx` | Integration | Edit → Save As → re-open text matches |

**Exit:** New/Open/Save/Save As work from File menu; round-trip native format lossless.

### F01.S2 — Auto-save and recent files

**Deliverables:** Timer-based autosave to temp path; recent list in File menu (max 10).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F01-S2-autosave-serializes` | Unit | Mock clock → autosave writes bytes |
| `I-F01-S2-crash-recover` | Integration | Kill after edit → reopen recovers draft |

**Exit:** Autosave interval configurable; recent opens last paths.

### F01.S3 — Export and print preview

**Deliverables:** Export PDF (structural); print preview uses paginated layout.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F01-S3-pdf-starts-with-header` | Unit | `tw-pdf` output `%PDF` |
| `I-F01-S3-export-pdf-menu` | Integration | Review→Export PDF saves file |
| `I-F01-S3-print-preview-toggle` | Integration | View toggles preview layout |

**Exit:** PDF export from menu; preview shows page breaks (not WYSIWYG print until F25).

### F01.S4 — Protection and properties

**Deliverables:** Detect password DOCX → clear error; read-only flag; properties dialog (title, author, page count).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F01-S4-password-import-error` | Unit | Encrypted DOCX → `ImportError::PasswordProtected` |
| `I-F01-S4-properties-dialog` | Integration | Shows fields from `docProps/core.xml` |

**Exit:** Password-protected files rejected with message; properties view-only.

**Out of scope:** Encrypt-on-save (F22), full OS print pipeline (F25).

---

## F02 — Text Editing

**Scope:** Typing, Delete, Backspace, Insert, Select, Copy, Cut, Paste, Paste Special, Undo, Redo, Drag and Drop; Multi-cursor (future).

### Baseline status: **Complete** (F02.S1–S4).

| Capability | Status |
|------------|--------|
| Typing / Backspace / Select | Implemented |
| Forward Delete | Implemented |
| Shift+arrow extend / double-click word | Implemented |
| Copy / Cut / Paste | Implemented (glyph path) |
| Paste Special | Implemented (plain + HTML; DOCX when available) |
| Undo / Redo | Implemented |
| Drag Drop | Implemented |

**Dependencies:** F03 (format at caret), F23 (paste HTML/DOCX fragment).

### F02.S1 — Core input (complete)

**Deliverables:** Maintain glyph keyboard path; Tab insert (done).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F02-S1-insert-undo-redo` | Unit | `EditSession` 100 ops |
| `I-F02-S1-typing-latency` | Integration | p99 &lt; 15 ms benchmark gate |

**Exit:** Glyph insert + Tab + undo/redo regression suite green; release p99 &lt; 15 ms.

### F02.S2 — Forward delete and selection extend (complete)

**Deliverables:** `Delete` key; double-click word select; shift+arrow extend.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F02-S2-delete-forward` | Unit | `DeleteRange` at caret |
| `I-F02-S2-delete-key` | Integration | Flutter `LogicalKeyboardKey.delete` |

**Exit:** Delete key, shift+arrow extend, and double-click word select covered by regression tests; forward delete undo restores.

### F02.S3 — Clipboard and paste special (complete)

**Deliverables:** Cut/copy/paste via system clipboard; Paste Special: plain, HTML (sanitized), keep source formatting when DOCX fragment available.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F02-S3-paste-plain-merges-runs` | Unit | Paste inserts text at caret |
| `I-F02-S3-paste-special-dialog` | Integration | Plain vs formatted choice |

**Exit:** Paste from Word/HTML preserves basic bold/lists where parser allows.

### F02.S4 — Drag-drop text (complete)

**Deliverables:** Drag selected text within document.

| Test ID | Type | Spec |
|---------|------|------|
| `I-F02-S4-drag-reorder` | Integration | Drag selection → new position |

**Out of scope:** Multi-cursor (`F02.S5-Future` — document only, no schedule).

---

## F03 — Character Formatting

**Scope:** Font family/size, bold, italic, underline, strikethrough, double underline, super/subscript, font color, highlight, character spacing, small caps, all caps, hidden text, text effects, typography (ligatures).

### Baseline status: **Complete** (F03.S1–S4).

| Capability | Status |
|------------|--------|
| Font, B/I/U, strike, super/sub | Implemented |
| Color, highlight | Implemented (ribbon pickers; DOCX round-trip) |
| Double underline, spacing, caps, hidden, ligatures | Implemented |

**Dependencies:** F06 (character styles), `tw-shape` for OpenType.

### F03.S1 — Core ribbon (complete)

**Deliverables:** Home ribbon wired for font family/size, bold, italic, underline, strikethrough, super/subscript; caret→ribbon read sync; run normalization merges adjacent equivalent formats after edit.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F03-S1-bold-merge-runs` | Unit | Adjacent same-format runs merge |
| `I-F03-S1-font-size-caret-end` | Integration | Ribbon font size at run end applies |

**Tests:** `crates/tw-edit/tests/f03_s1_core_formatting.rs`, `app/test/f03_s1_core_formatting_test.dart`, `app/test/f03_s1_underline_test.dart`

**Exit:** Met for B/I/U/font/size/strike/super/sub.

### F03.S2 — Color and highlight UI (complete)

**Deliverables:** Wire color pickers on Home tab; export `w:color` / `w:highlight`; yellow highlight visible in display list.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F03-S2-color-roundtrip-docx` | Unit | Import/export ARGB |
| `I-F03-S2-highlight-visible` | Integration | Yellow highlight in display list |

**Tests:** `crates/tw-docx/tests/f03_s2_color_roundtrip.rs`, `app/test/f03_s2_color_highlight_test.dart`

**Exit:** User can set font color and highlight from ribbon.

### F03.S3 — Extended underline and spacing (complete)

**Deliverables:** Double underline enum; `w:spacing` character spacing.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F03-S3-double-underline-layout` | Unit | Decoration kind in `TextLine` |

**Tests:** `crates/tw-layout/tests/f03_s3_double_underline_layout.rs`, `crates/tw-docx/tests/f03_s3_spacing_roundtrip.rs`, `crates/tw-edit/tests/f03_s3_character_spacing.rs`

**Exit:** Double underline renders as `DoubleUnderline` decorations; character spacing round-trips via `w:spacing` and widens layout.

### F03.S4 — Caps, hidden, OpenType (advanced) (complete)

**Deliverables:** Small caps / all caps / hidden flags; optional ligature feature in `tw-shape`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F03-S4-hidden-not-in-plaintext` | Unit | Hidden runs excluded from export plaintext |

**Tests:** `crates/tw-core/tests/f03_s4_hidden_plaintext.rs`, `crates/tw-docx/tests/f03_s4_caps_hidden_roundtrip.rs`, `crates/tw-layout/tests/f03_s4_hidden_layout.rs`, `crates/tw-shape/src/shaper.rs` (unit)

**Exit:** Hidden text omitted from `document_plain_text` and layout; caps/hidden round-trip via DOCX; `liga`/`smcp` OpenType features in shaper; Home ribbon toggles for All Caps, Small Caps, Hidden, and Ligatures.

---

## F04 — Paragraph Formatting

**Scope:** Alignment, line/paragraph spacing, indentation, hanging indent, tabs, borders, shading, keep with next, widow/orphan control.

### Baseline status: **Complete** (F04.S1–S4).

| Capability | Status |
|------------|--------|
| Align L/C/R/J, indent | Implemented |
| Line/para spacing, tabs | Implemented |
| Borders, shading | Implemented |
| keepNext, widow/orphan | Implemented (layout + UI) |

**Dependencies:** F07 (sections), F05 (list indents).

### F04.S1 — Alignment and indent (complete)

| Test ID | Type | Spec |
|---------|------|------|
| `I-F04-S1-alignment-buttons` | Integration | Home alignment updates layout |

### F04.S2 — Spacing UI (complete)

**Deliverables:** Line spacing single/1.5/double/exact; space before/after.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F04-S2-line-rule-exact` | Unit | `w:lineRule` exact twips in layout |
| `I-F04-S2-spacing-dialog` | Integration | Paragraph dialog applies spacing |

**Tests:** `crates/tw-layout/tests/f04_s2_line_rule_exact.rs`, `app/test/f04_s2_spacing_dialog_test.dart`

**Exit:** Exact line rule fixes laid-out `line_height` in points; Home Paragraph Spacing dialog applies Single / 1.5 / Double / Exactly plus space before/after via `SetParaFormatRange`. Rust tests assert ±0.01 pt exact height and a 36 pt combined space delta; Flutter covers Double, Exactly, and 1.5× encoding.

### F04.S3 — Tab stops editor (complete)

**Deliverables:** UI to add/remove tab stops; Tab key uses stops (F02).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F04-S3-custom-tab-stop` | Unit | `explicit_tab_stops.rs` layout test |

**Tests:** `crates/tw-layout/tests/explicit_tab_stops.rs`, `app/test/f04_s3_tab_stops_dialog_test.dart`

**Exit:** Explicit `ParaFormat.tab_stops` override the default tab grid in layout; Layout → Tabs dialog add/remove/clear applies via `SetParaFormatRange` (`Some([])` clears). Rust tests pin tab x within ±2 pt of the stop; Flutter asserts alignment round-trip.

### F04.S4 — Borders, shading, pagination flags (complete)

**Deliverables:** Para borders/shading on model + layout; keep/widow UI toggles.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F04-S4-keep-together-no-split` | Unit | Layout keeps para on one page |
| `U-F04-S4-widow-orphan` | Unit | Two-line para split respects widow |

**Tests:** `crates/tw-layout/tests/f04_s4_pagination_flags.rs`, `app/test/f04_s4_borders_pagination_test.dart`

**Exit:** Paragraph shading/borders emit `LayoutBox::Rect` before text lines; keep-together and widow/orphan pagination enforced with positive/negative controls; Home dialogs toggle pagination flags and borders/shading. Rust tests assert page-fill remainder, rect paint order, and four border strokes.

---

## F05 — Lists

**Scope:** Bullets, numbering, multi-level, custom bullets, restart/continue, outline numbering.

### Baseline status: **Complete** (F05.S1–S4).

| Capability | Status |
|------------|--------|
| Bullet / numbered | Implemented |
| Multi-level, restart | Implemented |
| Outline numbering | Implemented |
| Custom | Implemented (level format; custom glyph pickers limited) |
| Outline (F19 nav) | Pass (F19.S2) |

**Exit:** List level indents use ±0.01 pt position and ±2 pt hanging tolerances; restart markers reset counters with double-restart controls; numbered lists sync `outline_level` while bullets stay out of the outline; DOCX round-trips `numRestart` and `outlineLvl`; Flutter tests assert engine para format, non-list Tab fallback, max-level no-op, and outline navigation.

### F05.S1 — Basic lists (complete)

| Test ID | Type | Spec |
|---------|------|------|
| `U-F05-S1-lvltext-roundtrip` | Unit | `tier_a_numbering_styles.rs` |
| `I-F05-S1-bullet-toggle` | Integration | `f05_s1_bullet_toggle_test.dart` |

### F05.S2 — Multi-level and Tab promote/demote (complete)

**Deliverables:** Increase/decrease list level; Tab/Shift+Tab on list items.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F05-S2-level-indent` | Unit | `f05_s2_level_indent.rs` |
| `I-F05-S2-promote-demote` | Integration | `f05_s2_promote_demote_test.dart` |

### F05.S3 — Restart and continue numbering (complete)

**Deliverables:** Commands `RestartNumbering`, `ContinueNumbering`; DOCX `w:numRestart`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F05-S3-restart-counter` | Unit | `f05_s3_restart_counter.rs` |
| `I-F05-S3-restart-continue` | Integration | `f05_s3_restart_continue_test.dart` |
| `U-F05-S3-list-restart-cmd` | Unit | `f05_s3_list_restart.rs` |
| `U-F05-DOCX-list-props` | Unit | `f05_list_properties.rs` |

### F05.S4 — Outline numbering (complete)

**Deliverables:** Link to `outline_level` on para format; nav outline (F19).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F05-S4-outline-level` | Unit | `f05_s4_outline_level.rs` |
| `I-F05-S4-outline-nav` | Integration | `f05_s4_outline_nav_test.dart` |

---

## F06 — Styles

**Scope:** Heading 1–9, Normal, Quote, Caption, custom styles, inheritance, themes, style inspector.

### Baseline status: **Complete** (F06.S1–S4).

| Capability | Status |
|------------|--------|
| Normal, Heading 1–9, Quote, Caption | Implemented |
| Custom styles (create/rename/delete, DOCX) | Implemented |
| Themes (Design gallery) | Implemented |
| Style inspector | Implemented |

**Exit (S1):** Built-in paragraph styles ship in `StyleSheet::with_defaults` with Heading 2–9 chained via `based_on`; styles gallery applies resolved format through `applyParagraphStyle`; unit test verifies H3 inherits the H2→H1 chain.

**Exit (S2):** User paragraph styles can be created, renamed, and deleted via edit commands; built-ins are protected; export regenerates `word/styles.xml` when the style catalog changes.

**Exit (S3):** Built-in Office/Facet/Ion themes apply via Design tab; `resolve_theme_fonts` substitutes `+major*`/`+minor*` references; `resolve_theme_colors` re-resolves theme-linked run colors from accent/text/background slots; undo restores the prior theme name.

**Exit (S4):** Styles Pane toggles a right-side inspector showing paragraph style plus direct character overrides at the caret (`Heading 1 + Bold direct`).

### F06.S1 — Built-in paragraph styles (complete)

**Deliverables:** Add H2–H9, Quote, Caption to `StyleSheet::with_defaults`; gallery entries.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F06-S1-resolve-based-on` | Unit | H3 inherits H2 chain — `f06_s1_builtin_styles.rs` |
| `I-F06-S1-apply-heading-style` | Integration | Styles gallery applies size — `f06_s1_apply_heading_style_test.dart` |

### F06.S2 — Custom styles (complete)

**Deliverables:** Create/rename/delete user styles; save in DOCX `styles.xml`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F06-S2-custom-style-roundtrip` | Unit | Export/import custom style id — `f06_s2_custom_style_roundtrip.rs`, `f06_s2_custom_styles.rs` |

### F06.S3 — Themes (complete)

**Deliverables:** Apply document theme colors/fonts from Design tab.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F06-S3-theme-font-resolve` | Unit | `resolve_theme_fonts` — `f06_s3_theme_font_resolve.rs`, `f06_s3_document_theme.rs` |
| `U-F06-S3-theme-color-resolve` | Unit | Theme slot recolor on theme change — `f06_s3_theme_color_resolve.rs` |
| `I-F06-S3-design-tab` | Integration | Design gallery applies theme — `f06_s3_design_theme_test.dart` |
| `I-F06-S3-theme-color` | Integration | Theme accent color follows theme switch — `f06_s3_theme_color_test.dart` |

### F06.S4 — Style inspector (complete)

**Deliverables:** Pane showing resolved format at caret (direct + style + defaults).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F06-S4-inspector-shows-source` | Unit | Style vs direct labels — `f06_s4_style_inspector.rs` |
| `I-F06-S4-inspector-shows-source` | Integration | Toggle shows "Heading 1 + Bold direct" — `f06_s4_style_inspector_test.dart` |

---

## F07 — Page Layout

**Scope:** Margins, orientation, page size, columns, section breaks, page breaks, line numbering, watermark, page color, borders.

### Baseline status: **Complete** (F07.S1–S4).

| Capability | Status |
|------------|--------|
| Page break | Done (Insert tab) |
| Section break (next page) | Done (F07.S2) |
| Margins, size, orientation | Done (F07.S1) |
| Columns | Done (F07.S3) |
| Watermark, line numbers | Done (F07.S4) |

### F07.S1 — Margins, size, orientation UI (complete)

**Deliverables:** Layout tab controls → `SectionFormat`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F07-S1-landscape-swaps-dimensions` | Unit | Layout page width/height swap — `f07_s1_page_setup.rs` |
| `U-F07-S1-set-section-format` | Unit | Margins undo — `f07_s1_section_format.rs` |
| `I-F07-S1-margin-preset` | Integration | Narrow margin reflows text — `f07_s1_page_setup_test.dart` |

### F07.S2 — Section and page breaks (complete)

**Deliverables:** Section break (next page); page break (done).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F07-S2-section-format-per-section` | Unit | Multi-section document model — `f07_s2_section_break.rs`, `f07_s2_section_format_per_section.rs` |
| `I-F07-S2-section-break` | Integration | Layout tab inserts section break — `f07_s2_section_break_test.dart` |

### F07.S3 — Columns (complete)

**Deliverables:** 1–3 column layout in engine.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F07-S3-two-columns-balanced-flow` | Unit | Text flows column 1 then column 2 — `f07_s3_columns.rs` |
| `U-F07-S3-three-columns-narrower-width` | Unit | Line width fits column — `f07_s3_columns.rs` |
| `I-F07-S3-columns` | Integration | Layout tab applies Two columns — `f07_s3_columns_test.dart` |

### F07.S4 — Watermark, page color, line numbers (complete)

**Deliverables:** Background watermark rect; line number gutter.

**Out of scope:** Full Word art watermark behind text (use F11/F10).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F07-S4-page-color-rect` | Unit | Full-page color rect — `f07_s4_page_decorations.rs` |
| `U-F07-S4-watermark-background` | Unit | Watermark rect + text — `f07_s4_page_decorations.rs` |
| `U-F07-S4-line-number-gutter` | Unit | One gutter label per body line — `f07_s4_page_decorations.rs` |
| `U-F07-S4-line-numbers-continue` | Unit | Numbering continues on page 2 — `f07_s4_page_decorations.rs` |
| `U-F07-S4-set-page-color-and-watermark` | Unit | Section format patch — `f07_s4_section_decorations.rs` |
| `U-F07-S4-decorations-preserve-header` | Unit | Header text survives decoration patch — `f07_s4_section_decorations.rs` |
| `I-F07-S4-watermark` | Integration | Design tab watermark apply/remove — `f07_s4_page_decorations_test.dart` |
| `I-F07-S4-page-color` | Integration | Page color apply/clear — `f07_s4_page_decorations_test.dart` |
| `I-F07-S4-line-numbers` | Integration | Layout tab line numbers toggle — `f07_s4_page_decorations_test.dart` |

---

## F08 — Headers and Footers

**Scope:** Header, footer, page numbers, date/time, different first page, odd/even, section-specific.

### Baseline status: **Complete** (F08.S1–S4).

| Capability | Status |
|------------|--------|
| Import/layout HF blocks | Implemented |
| Edit header/footer body | Done (F08.S1) |
| Insert UI, page fields | Done (F08.S2) |
| First page / odd-even | Done (F08.S3) |
| Section-specific HF | Done (F08.S4) |

**Exit (S1):** Insert→Header/Footer ensures a default band with an editable empty paragraph; `InsertText` targets header/footer runs via `RunLocation`; layout places header lines in the top margin band; `.twdoc` round-trip preserves typed header text.

**Exit (S2):** `InsertField` stores PAGE/DATE field runs; layout evaluates them per page via `FieldEvalContext`; Insert→Page Number inserts a PAGE field at the caret.

**Dependencies:** F07 (sections), F16 (PAGE field).

### F08.S1 — Edit header/footer body (complete)

**Deliverables:** Insert→Header/Footer opens editable region; layout reserves margin band.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F08-S1-header-blocks-layout` | Unit | Header lines in `PageLayout` |
| `I-F08-S1-edit-header` | Integration | Type in header survives save |

### F08.S2 — Page number and date fields (complete)

**Deliverables:** Insert PAGE, DATE field codes; evaluate on layout.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F08-S2-page-field-increments` | Unit | Page 2 shows "2" |

### F08.S3 — First page / odd-even (complete)

**Deliverables:** Model flags; export `w:titlePg`, `w:evenAndOddHeaders`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F08-S3-first-page-header` | Unit | Page 1 uses First variant when `different_first_page` |
| `U-F08-S3-odd-even-headers` | Unit | Odd/even pages use distinct header bands |
| `U-F08-S3-export-flags` | Unit | DOCX round-trip preserves titlePg and evenAndOddHeaders |

**Exit (S3):** `SectionFormat.different_first_page` and `DocumentSettings.even_and_odd_headers` drive layout variant selection; Insert tab toggles First Page / Odd & Even; export emits OOXML flags.

### F08.S4 — Section-specific HF (complete)

**Deliverables:** Per-section HF blocks linked/unlinked.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F08-S4-linked-inherits-previous` | Unit | Linked section 2 shows section 1 header |
| `U-F08-S4-unlinked-distinct-header` | Unit | Unlinked section shows its own header |
| `U-F08-S4-linked-export` | Unit | `f08_s4_linked_export.rs` |
| `U-F08-S4-unlinked-export` | Unit | `f08_s4_linked_export.rs` |
| `U-F08-S4-footer-link` | Unit | `f08_s4_section_hf_links.rs` |
| `U-F08-S4-sync-session` | Unit | `f08_s4_section_hf_session.rs` |
| `I-F08-S4-linked-inherits` | Integration | `f08_s4_section_hf_test.dart` |
| `I-F08-S4-unlink-distinct` | Integration | `f08_s4_section_hf_test.dart` |
| `I-F08-S4-link-toggle` | Integration | `f08_header_footer_test.dart` |
| `S-F08-S4-link-churn` | Stress | `stress/f08_s4_section_hf_churn.rs` |

**Exit (S4):** New sections default to link-to-previous; `SetHeaderFooterLink` toggles per variant; layout resolves headers through the link chain; opening a linked band auto-unlinks and copies content; export skips header refs for linked sections.

---

## F09 — Tables

**Scope:** Insert, delete rows/columns, merge/split cells, AutoFit, borders, shading, sorting, formulas, nested tables.

### Baseline status: **Complete** (F09.S1–S5).

| Capability | Status |
|------------|--------|
| Insert | Done (F09.S1) |
| Delete row/column | Done (F09.S2) |
| Merge / split cells | Done (F09.S3) |
| Borders, shading, AutoFit | Done (F09.S4) |
| Sort, formulas, nested | Done (F09.S5) |

### F09.S1 — Insert (complete)

**Deliverables:** Insert tab → Table inserts a 3×3 table after the current block via `InsertTable` command.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F09-S1-insert-3x3` | Unit | 3×3 table block after paragraph; undo removes it |
| `U-F09-S1-table-layout` | Unit | Layout produces 9 cells and grid lines |
| `I-F09-S1-insert-3x3` | Integration | Insert tab → Table in doc |

**Tests:** `crates/tw-edit/tests/f09_s1_insert_table.rs`, `crates/tw-layout/tests/f09_s1_table_layout.rs`, `app/test/f09_s1_insert_table_test.dart`

**Exit:** Insert→Table enqueues `InsertTable { rows: 3, cols: 3 }`; document gains a table block; layout renders a 3×3 grid; mock/native round-trip preserves table dimensions.

### F09.S2 — Row/column delete (complete)

**Deliverables:** `DeleteTableRow`, `DeleteTableColumn` commands + FFI + Layout tab.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F09-S2-delete-row` | Unit | Row count decreases; undo |
| `I-F09-S2-delete-row-ui` | Integration | Layout→Delete Row |

**Tests:** `crates/tw-edit/tests/f09_s2_delete_row.rs`, `app/test/f09_s2_delete_row_test.dart`

**Exit:** Caret in table cell → Layout→Delete Row/Column removes one row or column; undo restores; last row/column cannot be deleted.

### F09.S3 — Merge and split (complete)

**Deliverables:** Wire existing `MergeTableCells`; add `SplitTableCell` command + Layout tab.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F09-S3-merge-cells-span` | Unit | colspan/rowspan on model and layout |
| `I-F09-S3-merge-cells` | Integration | Layout→Merge Cells / Split Cell |

**Tests:** `crates/tw-edit/tests/f09_s3_merge_split.rs`, `crates/tw-layout/tests/f09_s3_merge_cells_layout.rs`, `app/test/f09_s3_merge_split_test.dart`

**Exit:** Caret in table cell → Merge Cells combines with the cell to the right; Split Cell resets colspan/rowspan to 1; layout renders merged span width; undo restores prior span.

### F09.S4 — Borders, shading, AutoFit (complete)

**Deliverables:** Table design dialog; `SetTableBorder`, `SetTableCellShading`, `AutoFitTable`, `ResizeTableColumn` wired through FFI + Layout tab.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F09-S4-set-table-border` | Unit | Table border on model; undo |
| `U-F09-S4-cell-shading` | Unit | Cell background on model; undo |
| `U-F09-S4-resize-column` | Unit | Column width change; undo |
| `U-F09-S4-autofit-to-window` | Unit | Column widths scale to content width; undo |
| `U-F09-S4-table-border-layout` | Unit | Layout grid lines reflect border |
| `U-F09-S4-cell-shading-layout` | Unit | Layout cell background ARGB |
| `I-F09-S4-table-design-ui` | Integration | Layout→Table Design dialog |

**Tests:** `crates/tw-edit/tests/f09_s4_table_design.rs`, `crates/tw-layout/tests/f09_s4_table_design_layout.rs`, `app/test/f09_s4_table_design_test.dart`

**Exit:** Caret in table cell → Layout→Table Design sets border/shading/column width; AutoFit scales columns to page content width; layout renders borders and cell shading; undo restores prior format.

### F09.S5 — Sort, formulas, nested (complete)

**Deliverables:** Sort table by column; `=SUM(ABOVE)` field; nested table in cell layout.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F09-S5-sort-rows-asc` | Unit | Rows reorder by column; header preserved; undo |
| `U-F09-S5-insert-sum-field` | Unit | `TableSumAbove` field run at caret |
| `U-F09-S5-insert-nested-table` | Unit | Nested `Block::Table` in cell; undo |
| `U-F09-S5-nested-table-layout` | Unit | Layout cell has `nested_tables` slice |
| `U-F09-S5-sum-field-layout` | Unit | SUM field renders total above |
| `I-F09-S5-sort-table` | Integration | Layout→Sort A→Z |

**Tests:** `crates/tw-edit/tests/f09_s5_sort_formula_nested.rs`, `crates/tw-layout/tests/f09_s5_nested_sum_layout.rs`, `app/test/f09_s5_sort_formula_nested_test.dart`

**Exit:** Caret in table → Sort A→Z reorders data rows; Sum Formula inserts `=SUM(ABOVE)`; Nested Table adds 2×2 inside cell; layout renders nested grid and computed sum.

---

## F10 — Images

**Scope:** Insert, crop, rotate, resize, compression, wrap, position, caption, transparency, replace.

### Baseline status: **Complete** (F10.S1–S4).

| Capability | Status |
|------------|--------|
| Insert | **Done** (F10.S1 — file picker, PNG/JPEG/SVG bytes) |
| Wrap/anchor layout | **Done** (F10.S3) |
| Resize/replace | **Done** (F10.S2) |
| Crop, rotate, caption, compress | **Done** (F10.S4) |

### F10.S1 — Real image insert ✅

**Deliverables:** File picker → PNG/JPEG/SVG bytes in model; display list image batch.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F10-S1-import-png-bytes` | Unit | DOCX image round-trip bytes | ✅ `f10_s1_import_png_bytes.rs` |
| `I-F10-S1-insert-picture` | Integration | Insert→Picture from file | ✅ `f10_s1_insert_picture_test.dart` |

### F10.S2 — Resize and replace ✅

**Deliverables:** Drag handles; replace image keeps wrap.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F10-S2-set-image-size` | Unit | `display_width/height` change + undo | ✅ `f10_s2_resize_replace.rs` |
| `U-F10-S2-replace-bytes-keeps-wrap` | Unit | Replace bytes; wrap/anchor preserved | ✅ `f10_s2_resize_replace.rs` |
| `U-F10-S2-layout-reflow` | Unit | Layout box reflects new size | ✅ `f10_s2_image_resize_layout.rs` |
| `I-F10-S2-resize-drag` | Integration | Drag handle → size command | ✅ `f10_s2_resize_replace_test.dart` |
| `I-F10-S2-replace-picture` | Integration | Replace via engine | ✅ `f10_s2_resize_replace_test.dart` |

### F10.S3 — Wrap and position ✅

**Deliverables:** Square/inline/behind; anchor UI.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F10-S3-square-wrap-reflow` | Unit | `image_placement.rs` | ✅ `image_placement.rs` |
| `U-F10-S3-set-image-wrap` | Unit | Wrap/anchor commands + undo | ✅ `f10_s3_wrap_anchor.rs` |
| `I-F10-S3-wrap-square` | Integration | Layout tab → square wrap | ✅ `f10_s3_wrap_position_test.dart` |
| `I-F10-S3-move-image` | Integration | Drag image → anchor command | ✅ `f10_s3_wrap_position_test.dart` |

### F10.S4 — Crop, rotate, compress, caption, transparency ✅

**Deliverables:** Model transform; caption paragraph linked; optional re-encode.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F10-S4-set-image-transform` | Unit | Transform + crop + undo | ✅ `f10_s4_transform_caption.rs` |
| `U-F10-S4-insert-image-caption` | Unit | Caption paragraph linked | ✅ `f10_s4_transform_caption.rs` |
| `U-F10-S4-compress-image` | Unit | JPEG re-encode + undo | ✅ `f10_s4_transform_caption.rs` |
| `U-F10-S4-display-list-transform` | Unit | Rotation/opacity/crop in v6 batch | ✅ `f10_s4_image_transform_display_list.rs` |
| `I-F10-S4-rotate-picture` | Integration | Rotate → transform command | ✅ `f10_s4_transform_caption_test.dart` |
| `I-F10-S4-insert-caption` | Integration | Insert caption via engine | ✅ `f10_s4_transform_caption_test.dart` |
| `I-F10-S4-compress-picture` | Integration | Compress via engine | ✅ `f10_s4_transform_caption_test.dart` |

---

## F11 — Shapes

**Scope:** Rectangle, circle, arrow, lines, callouts, freeform, text boxes, icons, WordArt.

### Baseline status: **Complete** (F11.S1–S3 — preserve, insert basic shapes, text boxes/WordArt; freeform pen deferred).

**Dependencies:** F10 (drawing layer), `tw-render` paths.

### F11.S1 — Preserved DrawingML (read-only) ✅

**Deliverables:** Import `w:drawing` shapes as bounds + passthrough; placeholder render.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F11-S1-drawing-preserves-bytes` | Unit | Unmodified shape part in package | ✅ `f11_s1_drawing_preserves_bytes.rs` |
| `U-F11-S1-shape-placeholder-display-list` | Unit | Placeholder rect in display list | ✅ `f11_s1_shape_placeholder_display_list.rs` |

### F11.S2 — Insert basic shapes ✅

**Deliverables:** `ShapeBlock` model; rect, line, ellipse; stroke/fill.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F11-S2-shape-display-list` | Unit | Path batch contains shape | ✅ `f11_s2_shape_display_list.rs` |
| `U-F11-S2-insert-shape` | Unit | Insert rect/line/ellipse + undo | ✅ `f11_s2_insert_shape.rs` |
| `I-F11-S2-insert-rectangle` | Integration | Insert tab → rectangle | ✅ `f11_s2_insert_shape_test.dart` |

### F11.S3 — Text boxes and WordArt ✅

**Deliverables:** Shapes with embedded paragraph; simple WordArt text path.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F11-S3-text-box-paragraph` | Unit | Text box embeds paragraph + undo | ✅ `f11_s3_text_box_paragraph.rs` |
| `U-F11-S3-word-art-display-list` | Unit | WordArt arc path + inner glyphs | ✅ `f11_s3_word_art_display_list.rs` |
| `I-F11-S3-insert-text-box` | Integration | Insert tab → text box / WordArt | ✅ `f11_s3_insert_text_box_test.dart` |

**Out of scope:** SmartArt (F12), freeform pen (later).

---

## F12 — Smart Objects (SmartArt)

**Scope:** SmartArt, flowcharts, process diagrams, hierarchy, org charts.

### Baseline status: **Complete** (F12.S1–S3 — preserve, static preview, insert placeholder; SmartArt editing out of scope).

### F12.S1 — Preserve and placeholder ✅

**Deliverables:** `word/diagrams/*` passthrough; bounding box placeholder.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F12-S1-diagram-part-survives-save` | Unit | Package bytes unchanged | ✅ `f12_s1_diagram_part_survives_save.rs` |
| `U-F12-S1-diagram-placeholder-display-list` | Unit | Bounding box placeholder rect | ✅ `f12_s1_diagram_placeholder_display_list.rs` |

### F12.S2 — Render static diagram (optional) ✅

**Deliverables:** Raster fallback if EMF/PNG preview part exists.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F12-S2-diagram-preview-import` | Unit | PNG preview resolved from drawing part | ✅ `f12_s2_diagram_preview_import.rs` |
| `U-F12-S2-diagram-preview-display-list` | Unit | Preview PNG in image batch | ✅ `f12_s2_diagram_preview_display_list.rs` |

### F12.S3 — Insert placeholder and read-only selection ✅

**Deliverables:** Insert SmartArt placeholder; centered label; read-only click selection.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F12-S3-insert-diagram` | Unit | Insert diagram block + undo | ✅ `f12_s3_insert_diagram.rs` |
| `U-F12-S3-diagram-label-display-list` | Unit | SmartArt label + selection batch | ✅ `f12_s3_diagram_label_display_list.rs` |
| `I-F12-S3-insert-smart-art` | Integration | Insert tab → SmartArt | ✅ `f12_s3_insert_smart_art_test.dart` |

**Out of scope:** SmartArt editing (Tier C indefinitely).

---

## F13 — Charts

**Scope:** Bar, line, pie, area, scatter, radar, bubble, editable datasets.

### Baseline status: **Complete** (F13.S1–S3 — preserve, static preview, editable chart data).

### F13.S1 — Preserve chart parts ✅

**Deliverables:** `word/charts/*` passthrough; bounding box placeholder.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F13-S1-chart-xml-passthrough` | Unit | `word/charts/chart1.xml` preserved | ✅ `f13_s1_chart_xml_passthrough.rs` |

### F13.S2 — Static chart image ✅

**Deliverables:** Show embedded chart PNG if relationship exists.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F13-S2-chart-preview-import` | Unit | PNG preview resolved from chart part | ✅ `f13_s2_chart_preview_import.rs` |
| `U-F13-S2-chart-preview-display-list` | Unit | Preview PNG in image batch | ✅ `f13_s2_chart_preview_display_list.rs` |

### F13.S3 — Editable chart data ✅

**Deliverables:** Minimal data table model; insert chart; edit dataset; export round-trip.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F13-S3-insert-chart` | Unit | Insert chart block + undo | ✅ `f13_s3_insert_chart.rs` |
| `U-F13-S3-set-chart-data` | Unit | SetChartData command + undo | ✅ `f13_s3_set_chart_data.rs` |
| `U-F13-S3-chart-data-import` | Unit | Parse chart cache from chart part | ✅ `f13_s3_chart_data_import.rs` |
| `U-F13-S3-chart-data-export` | Unit | Edited dataset in chart1.xml | ✅ `f13_s3_chart_data_export.rs` |
| `U-F13-S3-chart-label-display-list` | Unit | Chart label + selection batch | ✅ `f13_s3_chart_label_display_list.rs` |
| `I-F13-S3-insert-chart` | Integration | Insert tab → Chart | ✅ `f13_s3_insert_chart_test.dart` |

### F12/F13 — Bugbot hardening ✅

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F12-S2-diagram-preview-selection-batch` | Unit | Preview diagram in shape selection batch | ✅ `f12_s2_diagram_preview_selection_batch.rs` |
| `U-F13-S2-chart-preview-selection-batch` | Unit | Preview chart in shape selection batch | ✅ `f13_s2_chart_preview_selection_batch.rs` |
| `U-F12-S3-inserted-diagram-export` | Unit | Inserted SmartArt OPC round-trip | ✅ `f12_s3_inserted_diagram_export.rs` |
| `U-F13-S3-multi-chart-content-types` | Unit | Per-chart content-type overrides | ✅ `f13_s3_multi_chart_content_types.rs` |
| `I-F12-S2-diagram-preview-selection` | Integration | Preview tap → diagram selection | ✅ `f12_s2_diagram_preview_selection_test.dart` |
| `I-F13-mixed-hit-test` | Integration | Shape vs image hit priority | ✅ `f13_mixed_hit_test_test.dart` |
| `stress_multi_chart_export_ten` | Stress (`#[ignore]`) | Ten charts → OPC metadata | ✅ `stress/f13_multi_chart_export.rs` |
| `stress_chart_passthrough_fifty_parts` | Stress (`#[ignore]`) | 50 chart parts byte-stable | ✅ `stress/f13_chart_passthrough_scale.rs` |
| `stress_malformed_chart_*` | Stress (`#[ignore]`) | Malformed chart OPC resilience | ✅ `stress/f13_malformed_chart.rs` |
| `stress_chart_data_churn` | Stress (`#[ignore]`) | 500× SetChartData → export round-trip | ✅ `tw-edit/tests/stress/f13_chart_data_churn.rs` |

---

## F14 — Equations

**Scope:** Equation editor, math symbols, fractions, integrals, matrices, Greek, LaTeX.

### Baseline status: **Complete** (F14.S1–S4 — OMML preserve, preview, equation editor, LaTeX import).

### F14.S1 — OMML preserve ✅

**Deliverables:** Import/export `m:oMath` (inline) and `m:oMathPara` (block) as opaque `RunContent::OfficeMath`; retention counts; math namespace on regenerated `document.xml`.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F14-S1-omml-passthrough` | Unit | Inline `m:oMath` round-trip + token preserved | ✅ `f14_s1_omml_passthrough.rs` |
| `U-F14-S1-omath-para-not-dropped` | Unit | Body `m:oMathPara` imported and exported | ✅ `f14_s1_omml_passthrough.rs` |
| `U-F14-S1-neighbor-edit-keeps-omml` | Unit | Edit adjacent paragraph → OMML survives save | ✅ `f14_s1_omml_passthrough.rs` |
| `U-F14-S1-same-para-edit-keeps-omml` | Unit | Edit text beside inline math → OMML survives | ✅ `f14_s1_omml_passthrough.rs` |
| `U-F14-S1-layout-survives-omml` | Unit | Layout engine does not panic on OMML paragraph | ✅ `f14_s1_omml_passthrough.rs` |

**Testing bar (F14.S2+):** Later slices must assert real geometry / mutated engine state / ribbon→canvas — not status strings or caption-only placeholders (see Chart/SmartArt lesson in W4 preserve-first work).

### F14.S2 — Equation layout (read-only) ✅

**Deliverables:** Extract `m:t` preview text from OMML; shape as scaled italic glyphs with Word-like math frame rect; display math (`m:oMathPara`) centered.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F14-S2-inline-equation-glyphs` | Unit | Inline OMML → glyph codepoints from `m:t`, not `[math]` | ✅ `f14_s2_equation_preview_display_list.rs` |
| `U-F14-S2-display-equation-glyphs` | Unit | Block `m:oMathPara` → frame rects + glyphs | ✅ `f14_s2_equation_preview_display_list.rs` |
| `U-F14-S2-mixed-text-equation` | Unit | Text + inline math on same line | ✅ `f14_s2_equation_preview_display_list.rs` |
| `U-F14-S2-equation-hit-test` | Unit | Preview lines not decorative; multi-run caret map | ✅ `f14_s2_equation_preview_layout.rs` |
| `U-F14-S2-math-frame-decoration` | Unit | `DecorationKind::MathFrame` on equation segments | ✅ `f14_s2_equation_preview_layout.rs` |
| `U-F14-S2-import-to-display-list` | Unit | DOCX import → layout → display list has OMML glyphs | ✅ `f14_s2_import_layout_display_list.rs` |

### F14.S3 — Equation editor UI ✅

**Deliverables:** Insert equation dialog; build OMML from palette.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F14-S3-insert-inline-omml` | Unit | `InsertOfficeMath` stores OMML on run | ✅ `f14_s3_insert_office_math.rs` |
| `U-F14-S3-insert-display-omml` | Unit | `InsertOfficeMathDisplay` adds centered block | ✅ `f14_s3_insert_office_math.rs` |
| `U-F14-S3-set-equation-undo` | Unit | `SetOfficeMath` undo restores prior XML | ✅ `f14_s3_insert_office_math.rs` |
| `U-F14-S3-equation-glyphs` | Unit | Insert → layout → display list has shaped glyphs | ✅ `f14_s3_equation_insert_display_list.rs` |
| `U-F14-S3-session-round-trip` | Unit | Session insert/fetch/set equation OMML | ✅ `f14_s3_equation_session.rs` |
| `I-F14-S3-insert-equation` | Integration | Ribbon → dialog → engine stores OMML | ✅ `f14_s3_equation_editor_test.dart` |
| `I-F14-S3-fraction-omml` | Integration | Fraction palette builds `<m:f>` OMML | ✅ `f14_s3_equation_editor_test.dart` |

### F14.S4 — LaTeX import (optional) ✅

**Deliverables:** LaTeX → OMML subset converter.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F14-S4-latex-frac` | Unit | `\frac{a}{b}` → `<m:f>` | ✅ `latex_omml.rs` |
| `U-F14-S4-latex-superscript` | Unit | `x^2` → `<m:sSup>` | ✅ `latex_omml.rs` |
| `U-F14-S4-latex-greek` | Unit | `\alpha`, `\beta` → Unicode in OMML | ✅ `latex_omml.rs` |
| `U-F14-S4-latex-sqrt` | Unit | `\sqrt{x}` → `<m:rad>` | ✅ `latex_omml.rs` |
| `U-F14-S4-latex-display-list` | Unit | LaTeX → layout → shaped glyphs | ✅ `f14_s4_latex_display_list.rs` |
| `U-F14-S4-latex-unknown` | Unit | Unknown command returns error | ✅ `latex_omml.rs` |
| `I-F14-S4-latex-equation-insert` | Integration | Equation dialog LaTeX tab → engine OMML | ✅ `f14_s4_latex_equation_dialog_test.dart` |
| `I-F14-S4-latex-model` | Integration | `EquationModel.latex` → display OMML | ✅ `f14_s4_latex_import_test.dart` |

---

## F15 — Symbols

**Scope:** Unicode, emoji, currency, mathematical symbols, special characters.

### Baseline status: **Complete** (F15.S1–S3 ✅).

### F15.S1 — Symbol dialog ✅

**Deliverables:** Modal grid by category; insert via `InsertText`.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F15-S1-catalog-has-copyright` | Unit | Catalog exposes © with stable id | ✅ `f15_s1_symbol_catalog_test.dart` |
| `U-F15-S1-categories-non-empty` | Unit | All symbol categories populated | ✅ `f15_s1_symbol_catalog_test.dart` |
| `U-F15-S1-insert-copyright` | Unit | `InsertText` stores © in paragraph | ✅ `f15_s1_insert_symbol.rs` |
| `U-F15-S1-insert-after-text` | Unit | Symbol appended after existing text | ✅ `f15_s1_insert_symbol.rs` |
| `U-F15-S1-currency-math-symbols` | Unit | € £ ± × ∞ → survive in one paragraph | ✅ `f15_s1_insert_symbol.rs` |
| `U-F15-S1-insert-symbol-undo` | Unit | Undo removes inserted ™ | ✅ `f15_s1_insert_symbol.rs` |
| `I-F15-S1-insert-copyright` | Integration | Ribbon → Symbol dialog → © in engine text | ✅ `f15_s1_symbol_dialog_test.dart` |
| `I-F15-S1-insert-euro-category` | Integration | Currency subset → € inserted | ✅ `f15_s1_symbol_dialog_test.dart` |
| `I-F15-S1-dismiss-no-insert` | Integration | Cancel leaves document unchanged | ✅ `f15_s1_symbol_dialog_test.dart` |
| `S-F15-S1-symbol-churn` | Stress | 500 symbol inserts + coalesced undo clears burst | ✅ `stress/f15_symbol_insert_churn.rs` |
| `S-F15-S1-symbol-docx-round-trip` | Stress | Full symbol palette export → import | ✅ `stress/f15_symbol_insert_churn.rs` |

### F15.S2 — Emoji and math symbols ✅

**Deliverables:** Emoji picker; math symbol subset (Greek letters, set theory).

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F15-S2-emoji-category-has-grinning` | Unit | Catalog exposes 😀 with stable id | ✅ `f15_s2_emoji_math_catalog_test.dart` |
| `U-F15-S2-emoji-symbols-non-empty` | Unit | Emoji category populated | ✅ `f15_s2_emoji_math_catalog_test.dart` |
| `U-F15-S2-math-greek-has-alpha` | Unit | Greek subset exposes α | ✅ `f15_s2_emoji_math_catalog_test.dart` |
| `U-F15-S2-math-extended-includes-set-theory` | Unit | ∀ ∃ ∈ ∅ in extended math | ✅ `f15_s2_emoji_math_catalog_test.dart` |
| `U-F15-S2-insert-emoji-grinning` | Unit | `InsertText` stores 😀 in paragraph | ✅ `f15_s2_insert_emoji_math.rs` |
| `U-F15-S2-insert-emoji-after-text` | Unit | Emoji appended after existing text | ✅ `f15_s2_insert_emoji_math.rs` |
| `U-F15-S2-insert-greek-set-theory` | Unit | α β γ ∀ ∃ ∈ ∅ survive in one paragraph | ✅ `f15_s2_insert_emoji_math.rs` |
| `U-F15-S2-insert-emoji-undo` | Unit | Undo removes inserted 🎉 | ✅ `f15_s2_insert_emoji_math.rs` |
| `I-F15-S2-insert-emoji-from-picker` | Integration | Ribbon → Emoji → 👍 in engine text | ✅ `f15_s2_symbol_dialog_test.dart` |
| `I-F15-S2-insert-greek-alpha` | Integration | Greek & Advanced → α inserted | ✅ `f15_s2_symbol_dialog_test.dart` |
| `I-F15-S2-emoji-and-math-direct-api` | Integration | Direct insert 😀α via controller | ✅ `f15_s2_symbol_dialog_test.dart` |
| `S-F15-S2-emoji-math-churn` | Stress | 500 emoji/math inserts + coalesced undo | ✅ `stress/f15_s2_emoji_math_churn.rs` |
| `S-F15-S2-emoji-math-docx-round-trip` | Stress | Full emoji + Greek palette export → import | ✅ `stress/f15_s2_emoji_math_churn.rs` |

### F15.S3 — Recent symbols ✅

**Deliverables:** Last-used list in dialog; MRU store with session persistence.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F15-S3-record-moves-to-front` | Unit | Re-insert bumps symbol to MRU head | ✅ `f15_s3_recent_symbols_test.dart` |
| `U-F15-S3-record-character-resolves-id` | Unit | Character insert maps to catalog id | ✅ `f15_s3_recent_symbols_test.dart` |
| `U-F15-S3-cap-trims-oldest` | Unit | List capped at max (12 default) | ✅ `f15_s3_recent_symbols_test.dart` |
| `U-F15-S3-load-export-round-trip` | Unit | loadIds/exportIds preserve order | ✅ `f15_s3_recent_symbols_test.dart` |
| `U-F15-S3-reinsert-recent-symbol` | Unit | Same symbol re-inserted at caret | ✅ `f15_s3_insert_recent.rs` |
| `U-F15-S3-recent-rotation-inserts` | Unit | Recent rotation © € ± α 😀 👍 in paragraph | ✅ `f15_s3_insert_recent.rs` |
| `U-F15-S3-recent-emoji-reinsert` | Unit | 👍 re-insert from recents pattern | ✅ `f15_s3_insert_recent.rs` |
| `I-F15-S3-recent-row-after-insert` | Integration | Dialog shows recent row after © insert | ✅ `f15_s3_symbol_dialog_test.dart` |
| `I-F15-S3-insert-from-recent-row` | Integration | Tap recent € → engine text | ✅ `f15_s3_symbol_dialog_test.dart` |
| `I-F15-S3-reinsert-bumps-mru` | Integration | Re-insert © moves to MRU front | ✅ `f15_s3_symbol_dialog_test.dart` |
| `U-F15-S3-session-store-persists-ids` | Unit | recent_symbols.json round-trip | ✅ `f15_s3_recent_symbols_test.dart` |
| `S-F15-S3-recent-reinsert-churn` | Stress | 600 recent-rotation inserts + undo | ✅ `stress/f15_s3_recent_symbol_churn.rs` |
| `S-F15-S3-recent-docx-round-trip` | Stress | 3× recent rotation export → import | ✅ `stress/f15_s3_recent_symbol_churn.rs` |

---

## F16 — References

**Scope:** Footnotes, endnotes, TOC, bibliography, citations, index, cross references.

### Baseline status: **Complete** (F16.S1–S4 — footnotes, TOC, citations/bibliography, index/cross-references).

### F16.S1 — Footnotes and endnotes ✅

**Deliverables:** `Footnote` model; layout bottom-of-page band; DOCX parts; References ribbon.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F16-S1-insert-footnote-creates-ref-and-body` | Unit | InsertFootnote → ref + footnote body | ✅ `f16_s1_insert_footnote.rs` |
| `U-F16-S1-insert-footnote-ref-is-superscript` | Unit | Footnote ref run has superscript format | ✅ `f16_s1_insert_footnote.rs` |
| `U-F16-S1-insert-footnote-after-text` | Unit | Footnote ref appended after text | ✅ `f16_s1_insert_footnote.rs` |
| `U-F16-S1-multiple-footnotes-renumber` | Unit | Second footnote gets display number 2 | ✅ `f16_s1_insert_footnote.rs` |
| `U-F16-S1-footnote-ref-layout` | Unit | Superscript ref + separator + body band | ✅ `f16_s1_footnote_ref_layout.rs` |
| `U-F16-S1-footnote-docx-round-trip` | Unit | Export/import preserves footnotes.xml | ✅ `f16_s1_footnote_roundtrip.rs` |
| `I-F16-S1-insert-footnote` | Integration | References → Insert Footnote → ¹ in engine | ✅ `f16_s1_insert_footnote_test.dart` |
| `I-F16-S1-insert-footnote-direct-api` | Integration | Controller.insertFootnote inserts marker | ✅ `f16_s1_insert_footnote_test.dart` |
| `S-F16-S1-footnote-insert-churn` | Stress | 50 footnote inserts in one paragraph | ✅ `stress/f16_s1_footnote_churn.rs` |
| `S-F16-S1-footnote-docx-round-trip` | Stress | 10 footnotes export → import | ✅ `stress/f16_s1_footnote_churn.rs` |

**Dependencies:** F06 (heading styles for TOC), F18 (cross-ref targets).

### F16.S2 — Table of contents ✅

**Deliverables:** TOC from Heading styles; literal page numbers from layout; References ribbon.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F16-S2-insert-toc-from-headings` | Unit | H1/H2 doc → TOC title + indented entries | ✅ `f16_s2_insert_toc.rs` |
| `U-F16-S2-toc-page-numbers` | Unit | Entries include supplied page numbers | ✅ `f16_s2_insert_toc.rs` |
| `U-F16-S2-toc-empty-outline-still-has-title` | Unit | No headings → title-only TOC | ✅ `f16_s2_insert_toc.rs` |
| `U-F16-S2-toc-docx-roundtrip` | Unit | Export/import preserves TOC paragraphs | ✅ `f16_s2_toc_roundtrip.rs` |
| `U-F16-S2-toc-entry-tab-and-page-layout` | Unit | Right tab stop lands page number | ✅ `f16_s2_toc_entry_layout.rs` |
| `U-F16-S2-session-toc-page-numbers` | Unit | Session TOC uses layout page numbers | ✅ `f16_s2_toc_session.rs` |
| `I-F16-S2-insert-toc-from-references-ribbon` | Integration | References → TOC → entries in engine | ✅ `f16_s2_insert_toc_test.dart` |
| `I-F16-S2-insert-toc-direct-api` | Integration | Controller.insertTableOfContents | ✅ `f16_s2_insert_toc_test.dart` |
| `S-F16-S2-toc-insert-churn` | Stress | 25 headings × 5 TOC inserts | ✅ `stress/f16_s2_toc_churn.rs` |
| `S-F16-S2-toc-docx-roundtrip` | Stress | 10 headings TOC export → import ×3 | ✅ `stress/f16_s2_toc_churn.rs` |

**Dependencies:** F06 (heading styles for TOC), F18 (cross-ref targets).

### F16.S3 — Citations and bibliography ✅

**Deliverables:** Bibliography source keys; inline `CitationRef`; `word/bibliography.xml`; References ribbon.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F16-S3-add-bibliography-source-registers-key` | Unit | AddBibliographySource → catalog entry | ✅ `f16_s3_insert_citation.rs` |
| `U-F16-S3-insert-citation-creates-ref-run` | Unit | CitationRef with author-year display | ✅ `f16_s3_insert_citation.rs` |
| `U-F16-S3-insert-citation-requires-source` | Unit | Unknown key rejected | ✅ `f16_s3_insert_citation.rs` |
| `U-F16-S3-insert-bibliography-from-citations` | Unit | Cited sources → Bibliography section | ✅ `f16_s3_insert_citation.rs` |
| `U-F16-S3-bibliography-xml-roundtrip` | Unit | Export/import preserves bibliography.xml + cites | ✅ `f16_s3_bibliography_roundtrip.rs` |
| `U-F16-S3-citation-ref-layout-text` | Unit | Layout renders citation display text | ✅ `f16_s3_citation_ref_layout.rs` |
| `I-F16-S3-insert-citation-from-references-ribbon` | Integration | References → Citation → (Smith, 2020) | ✅ `f16_s3_insert_citation_test.dart` |
| `I-F16-S3-insert-bibliography-from-references-ribbon` | Integration | Cite then Bibliography → entries | ✅ `f16_s3_insert_citation_test.dart` |
| `I-F16-S3-insert-citation-direct-api` | Integration | Controller.insertCitation | ✅ `f16_s3_insert_citation_test.dart` |
| `S-F16-S3-citation-insert-churn` | Stress | 20 sources × 20 citation inserts | ✅ `stress/f16_s3_citation_churn.rs` |
| `S-F16-S3-bibliography-docx-roundtrip` | Stress | Bibliography export → import ×3 | ✅ `stress/f16_s3_citation_churn.rs` |

### F16.S4 — Index and cross-references ✅

**Deliverables:** REF fields; bookmark targets (ties F19); References ribbon.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F16-S4-insert-bookmark-creates-anchor` | Unit | InsertBookmark → Bookmark run with name/id | ✅ `f16_s4_insert_cross_ref.rs` |
| `U-F16-S4-cross-ref-resolves-bookmark-text` | Unit | Bookmark + text → REF shows anchor text | ✅ `f16_s4_insert_cross_ref.rs` |
| `U-F16-S4-cross-ref-requires-bookmark` | Unit | Unknown bookmark rejected | ✅ `f16_s4_insert_cross_ref.rs` |
| `U-F16-S4-insert-index-from-bookmarks` | Unit | Multiple bookmarks → Index section | ✅ `f16_s4_insert_cross_ref.rs` |
| `U-F16-S4-cross-ref-docx-roundtrip` | Unit | bookmarkStart + REF fldSimple survive export/import | ✅ `f16_s4_cross_ref_roundtrip.rs` |
| `U-F16-S4-cross-ref-layout-text` | Unit | REF display text in layout | ✅ `f16_s4_cross_ref_layout.rs` |
| `I-F16-S4-insert-cross-ref-from-references-ribbon` | Integration | Ribbon → bookmark + cross-ref | ✅ `f16_s4_insert_cross_ref_test.dart` |
| `I-F16-S4-insert-index-from-references-ribbon` | Integration | Ribbon → bookmark + index | ✅ `f16_s4_insert_cross_ref_test.dart` |
| `I-F16-S4-insert-cross-ref-direct-api` | Integration | Controller.insertCrossReference | ✅ `f16_s4_insert_cross_ref_test.dart` |
| `S-F16-S4-cross-ref-insert-churn` | Stress | 20 bookmarks × 20 cross-refs | ✅ `stress/f16_s4_cross_ref_churn.rs` |
| `S-F16-S4-cross-ref-docx-roundtrip` | Stress | Index + REF export → import ×3 | ✅ `stress/f16_s4_cross_ref_churn.rs` |

---

## F17 — Review

**Scope:** Spell check, grammar, comments, track changes, compare, restrict editing, accept/reject.

### Baseline status: **Complete** (F17.S1–S4).

| Capability | Status |
|------------|--------|
| Spell | Implemented (Hunspell-compatible `suggest`) |
| Track changes | Implemented (caret accept/reject + next/previous) |
| Comments | Implemented (threads, margin markers, DOCX) |
| Grammar, compare, restrict | Implemented (rule-based grammar, line diff, read-only) |

### F17.S1 — Spell check upgrade ✅

**Deliverables:** Hunspell-compatible embedded dictionary; `suggest()` API; Review ribbon spell check.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F17-S1-hunspell-suggestions` | Unit | Known misspelling flagged; `teh` → `the` | ✅ `f17_s1_hunspell_suggestions.rs` |
| `U-F17-S1-known-good-words-pass` | Unit | Valid prose produces no issues | ✅ `f17_s1_hunspell_suggestions.rs` |
| `U-F17-S1-suggest-respects-limit` | Unit | Suggestion list capped | ✅ `f17_s1_hunspell_suggestions.rs` |
| `U-F17-S1-session-spell-check-misspellings` | Unit | Session spell check on typed typos | ✅ `f17_s1_spell_session.rs` |
| `I-F17-S1-spell-check-menu` | Integration | Review → Spelling lists words | ✅ `f17_s1_spell_check_test.dart` |
| `I-F17-S1-spell-check-direct-api` | Integration | Controller.spellCheckDocument | ✅ `f17_s1_spell_check_test.dart` |
| `I-F17-S1-spell-check-clean-document` | Integration | Clean doc → no issues status | ✅ `f17_s1_spell_check_test.dart` |
| `S-F17-S1-spell-check-churn` | Stress | 50 long typo paragraphs × check | ✅ `stress/f17_s1_spell_churn.rs` |
| `S-F17-S1-suggest-hot-path` | Stress | 5000 × suggest for `recieved` | ✅ `stress/f17_s1_spell_churn.rs` |

### F17.S2 — Track changes at caret ✅

**Deliverables:** Accept/Reject single revision; next/previous change.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F17-S2-accept-revision-caret` | Unit | `f17_s2_accept_revision_caret.rs` |
| `U-F17-S2-revision-session` | Unit | `f17_s2_revision_session.rs` |
| `I-F17-S2-accept-button` | Integration | Review→Accept |
| `S-F17-S2-revision-churn` | Stress | `stress/f17_s2_revision_churn.rs` |

**Status:** ✅ Delivered (caret accept/reject, next/previous navigation, FFI/WASM/Flutter wiring).

### F17.S3 — Comments ✅

**Deliverables:** `CommentThread` model; margin markers; DOCX `comments.xml`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F17-S3-insert-comment` | Unit | `f17_s3_insert_comment.rs` |
| `U-F17-S3-comment-margin-layout` | Unit | `f17_s3_comment_margin_layout.rs` |
| `U-F17-S3-comment-docx-roundtrip` | Unit | `f17_s3_comment_roundtrip.rs` |
| `U-F17-S3-comment-session` | Unit | `f17_s3_comment_session.rs` |
| `I-F17-S3-insert-comment` | Integration | Review→New Comment |
| `S-F17-S3-comment-churn` | Stress | `stress/f17_s3_comment_churn.rs` |

**Status:** ✅ Delivered (CommentThread model, margin markers, DOCX round-trip, FFI/WASM/Flutter wiring).

### F17.S4 — Grammar, compare, restrict ✅

**Deliverables:** Grammar via AI or LanguageTool; compare two docs; restrict editing flag.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F17-S4-grammar-rules` | Unit | `f17_s4_grammar_rules.rs` |
| `U-F17-S4-compare-text` | Unit | `f17_s4_compare_text.rs` |
| `U-F17-S4-grammar-session` | Unit | `f17_s4_grammar_session.rs` |
| `U-F17-S4-read-only-session` | Unit | `f17_s4_read_only_session.rs` |
| `U-F17-S4-read-only-model` | Unit | `f17_s4_read_only_model.rs` |
| `I-F17-S4-proofing-compare-restrict` | Integration | Review→Spelling & Grammar, Compare, Restrict Editing |
| `S-F17-S4-grammar-compare-churn` | Stress | `stress/f17_s4_grammar_compare_churn.rs` |

**Status:** ✅ Delivered (rule-based grammar, line diff compare, read-only worker guard, FFI/WASM/Flutter wiring).

---

## F18 — Search

**Scope:** Find, replace, regex, wildcards, navigation pane, search formatting.

### Baseline status: **Complete** (F18.S1–S4).

| Capability | Status |
|------------|--------|
| FindReplace command | Implemented |
| Find / Replace UI | Implemented |
| Regex / wildcards | Implemented |
| Format search | Implemented |

### F18.S1 — Find UI ✅

**Deliverables:** Find pane; highlight matches; next/previous.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F18-S1-find-case-sensitive` | Unit | `f18_s1_find_case_sensitive.rs` |
| `U-F18-S1-find-session` | Unit | `f18_s1_find_session.rs` |
| `I-F18-S1-find-pane` | Integration | Ctrl+F opens pane |
| `S-F18-S1-find-churn` | Stress | `stress/f18_s1_find_churn.rs` |

**Status:** ✅ Delivered (find matches API, find pane, match highlighting via selection, next/previous, Ctrl+F).

### F18.S2 — Replace ✅

| Test ID | Type | Spec |
|---------|------|------|
| `U-F18-S2-replace-all-count` | Unit | `f18_s2_replace_all_count.rs` |
| `U-F18-S2-replace-session` | Unit | `f18_s2_replace_session.rs` |
| `I-F18-S2-replace-all` | Integration | `f18_s2_replace_test.dart` |
| `S-F18-S2-replace-churn` | Stress | `stress/f18_s2_replace_churn.rs` |

**Status:** ✅ Delivered (FindReplace replacement count, Replace All in find pane, FFI/command bridge).

### F18.S3 — Regex and wildcards ✅

| Test ID | Type | Spec |
|---------|------|------|
| `U-F18-S3-regex-digit-runs` | Unit | `f18_s3_regex_wildcards.rs` |
| `U-F18-S3-wildcard-patterns` | Unit | `f18_s3_regex_wildcards.rs` |
| `U-F18-S3-regex-session` | Unit | `f18_s3_regex_session.rs` |
| `I-F18-S3-regex-pane` | Integration | `f18_s3_regex_test.dart` |
| `S-F18-S3-regex-churn` | Stress | `stress/f18_s3_regex_churn.rs` |

**Status:** ✅ Delivered (regex/wildcard find and replace, find pane toggles, FFI/command bridge).

### F18.S4 — Format search ✅

| Test ID | Type | Spec |
|---------|------|------|
| `U-F18-S4-find-bold-runs` | Unit | `f18_s4_format_search.rs` |
| `U-F18-S4-find-style-name` | Unit | `f18_s4_format_search.rs` |
| `U-F18-S4-format-session` | Unit | `f18_s4_format_session.rs` |
| `I-F18-S4-format-pane` | Integration | `f18_s4_format_search_test.dart` |
| `S-F18-S4-format-churn` | Stress | `stress/f18_s4_format_churn.rs` |

**Status:** ✅ Delivered (bold/style format filters in find, find pane toggles, FFI/command bridge).

---

## F19 — Navigation

**Scope:** Outline view, page thumbnails, bookmarks, hyperlinks, Go To, document map.

### Baseline status: **Complete** (F19.S1–S4).

| Capability | Status |
|------------|--------|
| Page thumbnails / nav pane | Implemented (F19.S1) |
| Outline view | Implemented (F19.S2) |
| Bookmarks, hyperlinks | Implemented (F19.S3) |
| Go To dialog | Implemented (F19.S4) |

### F19.S1 — Thumbnails and page strip (complete)

**Deliverables:** Navigation pane Pages tab with aspect-ratio thumbnails (display-list preview when available); click page N → scroll canvas (`jumpToPage`).

| Test ID | Type | Spec |
|---------|------|------|
| `I-F19-S1-page-nav-jump` | Integration | Click page N → scroll |

### F19.S2 — Outline view (complete)

**Deliverables:** Collapsible headings tree from `outline_level` / heading styles / numbered outline; click → caret + scroll (`jumpToOutlineEntry`); View → Document Outline opens the Outline tab.

| Test ID | Type | Spec |
|---------|------|------|
| `I-F19-S2-outline-tree` | Integration | Nested headings + collapse |
| `I-F19-S2-outline-jump` | Integration | Click heading → scroll |

### F19.S3 — Bookmarks and hyperlinks (complete)

**Deliverables:** `BookmarkAnchor` / `HyperlinkTarget` on model; `InsertHyperlink` (+ in-place edit); Insert → Link/Bookmark dialogs; `w:hyperlink` export with external relationships and `w:anchor` for internal targets; import resolves `r:id` to URL.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F19-S3-hyperlink-roundtrip-docx` | Unit | `w:hyperlink` import/export |
| `U-F19-S3-insert-hyperlink-creates-run` | Unit | InsertHyperlink → Hyperlink run |
| `I-F19-S3-insert-hyperlink-from-insert-ribbon` | Integration | Insert → Link dialog → insert |

### F19.S4 — Go To dialog (complete)

**Deliverables:** Go To dialog for page / bookmark / heading; `document_bookmarks` + `bookmarks_json` bridge; `jumpToBookmark` / reuse `jumpToPage` + `jumpToOutlineEntry`; entry points: Ctrl/Cmd+G, Edit → Go To…, View → Go To.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F19-S4-document-bookmarks-lists-anchors` | Unit | Bookmark nav entries from model |
| `I-F19-S4-goto-page` | Integration | Go To page N → scroll |
| `I-F19-S4-goto-bookmark` | Integration | Go To bookmark → caret + scroll |
| `I-F19-S4-goto-heading` | Integration | Go To heading → caret + scroll |

---

## F20 — Collaboration

**Scope:** Multiple users, live editing, presence, comments, version history, sharing, permissions.

### Baseline status: **Stub** (`tw-crdt` not implemented).

**Dependencies:** F17 (comments, TC), ADR-0009.

### F20.S1 — CRDT prototype

**Deliverables:** Map `Command` ↔ Yjs updates on single paragraph.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F20-S1-convergence-two-clients` | Unit | Two op streams merge |

### F20.S2 — Presence and live cursors

### F20.S3 — Comments sync

### F20.S4 — Version history and sharing

**Deliverables:** Snapshot list; share link + permissions (F27).

---

## F21 — Accessibility

**Scope:** Checker, alt text, read aloud, keyboard navigation, screen reader support.

### Baseline status: **Complete** (F21.S1–S5).

| Capability | Status |
|------------|--------|
| Semantic document tree | Implemented (F21.S1) |
| Keyboard / ribbon Tab order | Implemented (F21.S2) |
| Alt text / checker | Implemented (F21.S3–S4) |
| Read aloud | Implemented (F21.S5) |

### F21.S1 — Semantic document tree (complete)

**Deliverables:** Parallel model tree (`semantic_document_tree`): nested headings by outline level, body paragraphs, tables (+ cell children); JSON a11y API via `semantic_tree_json` / `tw_get_semantic_tree` / `fetchSemanticTree`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F21-S1-heading-structure` | Unit | H1→H2 order in tree |

### F21.S2 — Keyboard navigation audit (complete)

**Deliverables:** Ribbon `FocusTraversalGroup` + focusable controls (`RibbonFocusable`); Space/Enter activate; disabled controls skipped; wired shortcuts catalog in [`docs/keyboard.md`](keyboard.md) + `kWiredShortcuts`.

| Test ID | Type | Spec |
|---------|------|------|
| `I-F21-S2-ribbon-tab-order` | Integration | Tab visits ribbon tabs then enabled Home controls |
| `I-F21-S2-ribbon-activate` | Integration | Space/Enter on focused control invokes `onPressed` |
| `I-F21-S2-disabled-skip` | Integration | Disabled ribbon control not in traversal |
| `D-F21-S2-shortcuts-doc` | Docs | `docs/keyboard.md` matches `kWiredShortcuts` |

### F21.S3 — Alt text on images

**Deliverables:** `ImageBlock.alt_text`; inspector field.

**Status:** Complete.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F21-S3-set-alt-text` | Unit | `SetImageAltText` trims, clears, and undoes |
| `U-F21-S3-docx-descr-roundtrip` | Unit | Export/import preserves `wp:docPr/@descr` |
| `I-F21-S3-alt-text-field` | Integration | Picture inspector field applies alt text via engine |

### F21.S4 — Accessibility checker

**Deliverables:** Rules: missing alt, empty heading, low contrast warning.

**Status:** Complete.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F21-S4-missing-alt` | Unit | Flags images without alt; skips when set |
| `U-F21-S4-missing-alt-in-table` | Unit | Image in table cell is flagged |
| `U-F21-S4-empty-heading` | Unit | Heading style + whitespace-only text |
| `U-F21-S4-low-contrast` | Unit | Light gray on white → warning; black clean |
| `I-F21-S4-checker-review-button` | Integration | Review Check Accessibility runs check + status |
| `I-F21-S4-checker-pane-jump` | Integration | Results list jumps to heading / image |

### F21.S5 — Read aloud (optional)

**Deliverables:** Platform TTS reads selection.

**Status:** Complete.

| Test ID | Type | Spec |
|---------|------|------|
| `I-F21-S5-read-aloud-speaks-selection` | Integration | Review Read Aloud speaks selected text via TTS |
| `I-F21-S5-read-aloud-requires-selection` | Integration | Empty selection shows status prompt; no speech |
| `I-F21-S5-read-aloud-stop` | Integration | Stop cancels in-progress read aloud |

Platform backends: web `speechSynthesis`, macOS `NSSpeechSynthesizer` (`tutuaword/tts`).

---

## F22 — Security

**Scope:** Password protection, encryption, digital signatures, IRM, document inspection, remove metadata.

### Baseline status: **Complete** — F22.S1–S4 shipped (spec in `architecture/security.md`); S5 IRM deferred.

### F22.S1 — Password-protected open ✅

**Deliverables:** Detect encryption; prompt for password; decrypt via `office-crypto` (ECMA-376 Standard/Agile).

| ID | Type | Criteria |
|----|------|----------|
| `U-F22-S1-detects-encrypted` | Unit | ZIP/OLE encryption markers → password required |
| `U-F22-S1-decrypt-import` | Unit | Correct password decrypts fixture and imports DOCX |
| `I-F22-S1-prompt-open` | Integration | Password dialog / prompt callback opens encrypted doc |
| `I-F22-S1-wrong-password-retry` | Integration | Wrong password re-prompts; cancel aborts |

### F22.S2 — Encrypt on save ✅

**Deliverables:** Protect with Password sets a session encryption password; DOCX save encrypts via ECMA-376 Agile (`ms-offcrypto-writer`); F22.S1 open round-trips; Remove Password restores plaintext DOCX saves.

| ID | Type | Criteria |
|----|------|----------|
| `U-F22-S2-encrypt-produces-protected` | Unit | Plain DOCX + password → OLE encryption markers |
| `U-F22-S2-encrypt-decrypt-roundtrip` | Unit | Encrypt → decrypt/import restores content |
| `U-F22-S2-export-with-password` | Unit | `export_document` with `encryption_password` yields protected DOCX |
| `I-F22-S2-protect-dialog` | Integration | Protect dialog sets password; Save as DOCX writes encrypted file |
| `I-F22-S2-remove-password` | Integration | Remove password; subsequent save is plaintext |

### F22.S3 — Document inspector ✅

**Deliverables:** Inspect Document dialog lists comments, document properties, and hidden text; Remove clears selected categories in-model; DOCX export drops `word/comments.xml` when empty and rewrites `docProps/core.xml` from cleared properties.

| ID | Type | Criteria |
|----|------|----------|
| `U-F22-S3-inspect-findings` | Unit | `inspect_document` reports comments / metadata / hidden runs |
| `U-F22-S3-remove-categories` | Unit | `RemoveInspectFindings` clears selected categories |
| `U-F22-S3-export-strip-comments` | Unit | Empty comments → no `word/comments.xml` on export |
| `U-F22-S3-export-clear-core-props` | Unit | Cleared title/author round-trip via `docProps/core.xml` |
| `I-F22-S3-inspect-dialog` | Integration | Dialog shows findings; Remove clears mock engine state |

### F22.S4 — Digital signatures ✅

**Deliverables:** Sign document content (SHA-256 + Ed25519); embed signatures in DOCX `customXml/digitalSignatures.xml` and twdoc `signatures.json`; verify reports valid / tampered / invalid; File → Digital Signatures… dialog to sign, review, or remove.

| ID | Type | Criteria |
|----|------|----------|
| `U-F22-S4-sign-verify` | Unit | Sign → verify valid; content edit → tampered |
| `U-F22-S4-hash-ignores-sigs` | Unit | Content hash unchanged when only signatures list changes |
| `U-F22-S4-docx-roundtrip` | Unit | Export/import preserves signatures and validity |
| `U-F22-S4-twdoc-roundtrip` | Unit | twdoc `signatures.json` round-trips |
| `I-F22-S4-sign-dialog` | Integration | Dialog signs / clears via mock engine |

### F22.S5 — IRM (enterprise)

**Out of scope until P6 enterprise phase.

---

## F23 — File Formats

**Scope:** Import DOCX, DOC, RTF, TXT, ODT, HTML, Markdown; Export DOCX, PDF, ODT, HTML, EPUB, TXT.

### Baseline status: **Complete** — F23.S1–S4 shipped; binary `.doc` / EPUB remain deferred non-goals.

| Format | Import | Export |
|--------|--------|--------|
| DOCX | Implemented (F23.S1 hardened) | Implemented |
| ODT, MD, HTML, RTF, TXT | Implemented (F23.S3–S4 lists/tables/headings) | Implemented (TXT yes; RTF export limited) |
| PDF | — | Implemented (structural + VisualMatch F23.S2) |
| DOC | Missing (non-goal) | Missing |
| EPUB | Missing | Future |

### F23.S1 — DOCX hardening (critical path) ✅

**Deliverables:** ≥50 gate-eligible corpus fixtures; ≥95% open/layout; Tier A round-trip per category (numbering, styles, tables, char formats, images); 50-file import→export→re-import gate in CI. Encrypted/`_` benchmark fixtures excluded from the open gate.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F23-S1-tier-a-numbering` | Unit | `tier_a_numbering_styles.rs` (numbering + styles + table/char/image) |
| `I-F23-S1-corpus-render` | Integration | `corpus_render_gate_passes_95_percent` |
| `I-F23-S1-roundtrip-50` | Integration | `i_f23_s1_roundtrip_50` |

**Exit:** Continuous CI corpus + Tier A gates ([risk-mitigation](risk-mitigation.md) S1).

### F23.S2 — PDF VisualMatch ✅

**Deliverables:** Font embedding in `tw-pdf` (`/FontFile2` TrueType, `/FontFile3` CFF; TTC faces extracted); images as PDF XObjects; `PdfFidelity::VisualMatch` / `embed_fonts` export path.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F23-S2-visual-match-ready` | Unit | `PdfFidelity::VisualMatch` exports with `/FontFile2` |
| `U-F23-S2-embed-fonts` | Unit | `/FontFile2` in PDF when `embed_fonts` |

**Exit:** Structural PDF remains default; VisualMatch embeds layout faces.

### F23.S3 — ODT / HTML / MD fidelity ✅

**Deliverables:** ODT edit→export→reimport preserves text + bold (forced `content.xml` rewrite); HTML/MD map Heading 1–6 by style name (not “any `style_id`”); Quote/Normal stay `<p>` / non-ATX.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F23-S3-odt-roundtrip` | Unit | `u_f23_s3_odt_roundtrip` |
| `U-F23-S3-html-export-headings` | Unit | `u_f23_s3_html_export_headings` (H1→`<h1>`) |

### F23.S4 — RTF polish and TXT/EPUB ✅

**Deliverables:** RTF import for bullet/numbered lists (`NumberingRef` catalog ids 1/2) and simple `\trowd`/`\cell`/`\row` tables; TXT export writes UTF-8 lines (not TWDOC ZIP). EPUB export remains **future**.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F23-S4-rtf-list-bullet` | Unit | `u_f23_s4_rtf_list_bullet` |
| `U-F23-S4-rtf-list-numbered` | Unit | `u_f23_s4_rtf_list_numbered` |
| `U-F23-S4-rtf-table` | Unit | `u_f23_s4_rtf_table` |
| `U-F23-S4-txt-export` | Unit | `u_f23_s4_txt_export_roundtrip` |

**Out of scope:** Binary `.doc` — document as non-goal. EPUB implementation deferred.

---

## F24 — Templates

**Scope:** Resume, letter, invoice, brochure, newsletter, business proposal, research paper templates.

### Baseline status: **Complete** (F24.S1–S4).

### F24.S1 — Built-in template pack ✅

**Deliverables:** 7 starter `.docx` templates in `app/assets/templates/`; File → **New from Template…** opens untitled styled docs (`newFromTemplate`).

| Test ID | Type | Spec |
|---------|------|------|
| `I-F24-S1-new-from-resume` | Integration | New from Template → Resume opens styled doc untitled |

### F24.S2 — Theme binding ✅

**Deliverables:** Each starter template binds a Design gallery `DocumentTheme` (Office / Facet / Ion); `newFromTemplate` applies it so `documentThemeName` and the Design tab match.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F24-S2-template-theme-map` | Unit | All 7 templates map to gallery themes |
| `I-F24-S2-new-from-resume-theme` | Integration | Resume → Facet |
| `I-F24-S2-design-tab-reflects-template` | Integration | Design tab reflects template theme |

### F24.S3 — Save as template ✅

**Deliverables:** File → **Save as Template…** persists a `.docx` snapshot under `~/.tutuaword/templates/` with `index.json`; New from Template lists **My Templates**; opens untitled and re-applies the saved `documentThemeName`. Working path / dirty state unchanged.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F24-S3-template-store-roundtrip` | Unit | Save → list → read bytes round-trip |
| `U-F24-S3-slug-unique` | Unit | Duplicate titles get unique slug ids |
| `I-F24-S3-save-as-template-menu` | Integration | Dialog saves into My Templates |
| `I-F24-S3-new-from-user-template` | Integration | User template opens untitled with theme |

### F24.S4 — Template gallery UI ✅

**Deliverables:** New from Template is a Word-style card gallery — search, All / Built-in / My Templates chips, theme-accented page preview cards, selection summary, Create (or second click on selected card) to open untitled.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F24-S4-filter-query` | Unit | Search filters builtin titles |
| `U-F24-S4-filter-category` | Unit | Category chips hide other groups |
| `U-F24-S4-preview-kinds` | Unit | Each builtin maps to a preview kind + theme accent |
| `I-F24-S4-gallery-shows-cards` | Integration | Gallery shows all 7 builtin cards |
| `I-F24-S4-gallery-filter` | Integration | Search hides non-matches |
| `I-F24-S4-gallery-create` | Integration | Select + Create opens letter untitled |
| `I-F24-S4-gallery-my-templates-category` | Integration | My Templates chip shows only user cards |

---

## F25 — Printing

**Scope:** Duplex, multiple pages per sheet, booklet, margins, scaling, print selection.

### Baseline status: **Complete** (F25.S1–S4 print pipeline).

**Dependencies:** F23.S2 (font-embedded PDF for WYSIWYG).

### F25.S1 — OS print dialog ✅

**Deliverables:** File → **Print…** / ⌘P / title-bar Print builds a print-ready PDF (`prepare_print_pdf`: VisualMatch with structural fallback) and presents the OS print dialog (macOS PDFKit `NSPrintOperation`; other hosts fall back to a temp PDF). Injectable `DocumentPrintHost` for tests.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F25-S1-for-print-options` | Unit (Rust) | `PdfExportOptions::for_print` is VisualMatch |
| `U-F25-S1-prepare-print-pdf-header` | Unit (Rust) | `prepare_print_pdf` → `%PDF` |
| `U-F25-S1-pdf-header-gate` | Unit (Flutter) | `isPdfHeader` |
| `U-F25-S1-recording-print-host` | Unit (Flutter) | Recording host captures job |
| `I-F25-S1-export-pdf-for-print` | Integration (Rust) | Session `export_pdf_for_print` → `%PDF` |
| `I-F25-S1-print-dialog-opens` | Integration (Flutter) | `printDocument` presents dialog |
| `I-F25-S1-title-bar-print` | Integration (Flutter) | Title-bar Print invokes host |
| `S-F25-S1-print-pdf-churn` | Stress (Rust, `#[ignore]`) | 25× multi-para print PDF |
| `S-F25-S1-print-pdf-large-document` | Stress (Rust, `#[ignore]`) | 200-para multi-page PDF |
| `S-F25-S1-print-churn` | Stress (Flutter) | 100× `printDocument` |

### F25.S2 — Scaling and margins ✅

**Deliverables:** Print settings dialog (scale: 100% / Fit / Custom percent; margin presets) before the OS dialog. Engine applies `PrintLayoutOptions` as a content CTM in the print PDF (`prepare_print_pdf` / FFI / WASM). Flutter `PrintLayoutSettings` mirrors the Rust resolve math.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F25-S2-resolve-actual-size` | Unit (Rust/Flutter) | Identity transform at 100% |
| `U-F25-S2-resolve-custom-scale` | Unit (Rust/Flutter) | 50% centered |
| `U-F25-S2-resolve-fit-to-margins` | Unit (Rust/Flutter) | Fit shrinks into margin box |
| `U-F25-S2-prepare-print-pdf-embeds-scale-cm` | Unit (Rust) | PDF content includes scale CTM |
| `U-F25-S2-scale-mode-codes` | Unit (Flutter) | FFI discriminant 0/1/2 |
| `I-F25-S2-export-pdf-for-print-scaled` | Integration (Rust) | Session export with scale |
| `I-F25-S2-print-passes-layout` | Integration (Flutter) | Engine receives layout |
| `I-F25-S2-print-settings-dialog` | Integration (Flutter) | Settings UI → Custom |
| `I-F25-S2-title-bar-shows-settings` | Integration (Flutter) | Title-bar Print → settings |
| `S-F25-S2-print-scale-margin-churn` | Stress (Rust, `#[ignore]`) | Scale×margin matrix |
| `S-F25-S2-print-layout-churn` | Stress (Flutter) | 100× layout variants |

### F25.S3 — Print selection ✅

**Deliverables:** Print settings **Pages** scope (Document / Selection). When Selection is chosen, the engine clones the body `DocRange` via `document_from_range` and builds a print PDF (`prepare_print_pdf_selection` / FFI `tw_export_pdf_for_print_selection` / WASM). Selection is disabled in the dialog when the caret has no glyph selection.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F25-S3-document-from-range-partial-run` | Unit (Rust) | Subset text = selected slice |
| `U-F25-S3-document-from-range-preserves-format` | Unit (Rust) | Bold preserved |
| `U-F25-S3-prepare-print-pdf-selection-header` | Unit (Rust) | Selection PDF → `%PDF` |
| `U-F25-S3-print-scope-defaults-document` | Unit (Flutter) | Default scope = document |
| `U-F25-S3-mock-engine-records-selection` | Unit (Flutter) | Engine records range |
| `I-F25-S3-export-pdf-for-print-selection` | Integration (Rust) | Session selection export |
| `I-F25-S3-print-passes-selection` | Integration (Flutter) | `printDocument` passes range |
| `I-F25-S3-print-settings-selection-enabled` | Integration (Flutter) | Dialog Selection scope |
| `S-F25-S3-document-from-range-churn` | Stress (Rust, `#[ignore]`) | Many slice subsets |
| `S-F25-S3-print-selection-churn` | Stress (Rust `#[ignore]` / Flutter) | Selection PDF / 100× print |

### F25.S4 — Duplex, booklet, N-up ✅

**Deliverables:** Print settings for **Duplex** (one-sided / long edge / short edge), **Pages per sheet** (1/2/4/6/9/16), and **Booklet**. Engine plans imposition (`build_print_sheet_plan`, `booklet_page_order`, N-up cell transforms) and composes sheet pages into the print PDF when N-up/booklet is active. Duplex is forwarded as a platform attribute (`NSPrintInfo.duplex` on macOS). Booklet forces 2-up + long-edge duplex.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F25-S4-normalize-pages-per-sheet` | Unit (Rust/Flutter) | N-up clamp grid |
| `U-F25-S4-booklet-page-order` | Unit (Rust/Flutter) | 8-page signature order |
| `U-F25-S4-sheet-plan-nup-four` | Unit (Rust) | 5 pages → 2 sheets of 4 |
| `U-F25-S4-nup-cell-transform-two-up` | Unit (Rust) | Left/right cell placement |
| `U-F25-S4-prepare-print-pdf-nup-header` | Unit (Rust) | N-up PDF → `%PDF` |
| `U-F25-S4-booklet-forces-duplex-long-edge` | Unit (Flutter) | Booklet overrides duplex |
| `I-F25-S4-export-pdf-for-print-nup` | Integration (Rust) | Session N-up export |
| `I-F25-S4-export-pdf-for-print-booklet` | Integration (Rust) | Session booklet export |
| `I-F25-S4-print-passes-sheet-attributes` | Integration (Flutter) | Host receives duplex/N-up |
| `I-F25-S4-print-settings-sheet-controls` | Integration (Flutter) | Dialog booklet toggle |
| `S-F25-S4-sheet-plan-churn` | Stress (Rust, `#[ignore]`) | pages×N-up matrix |
| `S-F25-S4-print-sheet-attr-churn` | Stress (Flutter) | 100× attribute variants |

---

## F26 — Macros and Automation

**Scope:** Plugin support, mail merge, form fields.

**Out of scope:** VBA / scripting API execution — preserve `vbaProject.bin` only.

### Baseline status: **Complete** — F26.S1–S3 form fields + mail merge + WASM plugin host ✅.

### F26.S1 — Form fields ✅

**Deliverables:** Plain text / checkbox `FieldType`s + `FormFieldMeta` in model; `InsertFormField` / `SetFormFieldValue`; DOCX `FORMTEXT` / `FORMCHECKBOX` via `w:fldSimple`; Insert ribbon dialog; FFI/WASM bridge.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F26-S1-form-text-eval` | Unit | Form text evaluates to default text | ✅ `f26_s1_form_field_eval.rs` |
| `U-F26-S1-form-checkbox-eval` | Unit | Checkbox evaluates to ☑/☐ | ✅ `f26_s1_form_field_eval.rs` |
| `U-F26-S1-insert-form-text` | Unit | InsertFormField plain text + name | ✅ `f26_s1_insert_form_field.rs` |
| `U-F26-S1-insert-form-checkbox` | Unit | InsertFormField checkbox checked | ✅ `f26_s1_insert_form_field.rs` |
| `U-F26-S1-set-form-text-value` | Unit | SetFormFieldValue updates text | ✅ `f26_s1_insert_form_field.rs` |
| `U-F26-S1-toggle-form-checkbox` | Unit | SetFormFieldValue `toggle` flips checkbox | ✅ `f26_s1_insert_form_field.rs` |
| `I-F26-S1-form-text-docx-roundtrip` | Integration | Export/import FORMTEXT | ✅ `f26_s1_form_field_roundtrip.rs` |
| `I-F26-S1-form-checkbox-docx-roundtrip` | Integration | Export/import FORMCHECKBOX | ✅ `f26_s1_form_field_roundtrip.rs` |
| `U-F26-S1-mock-insert-form-text` | Unit | Mock engine inserts text field | ✅ `f26_s1_form_field_test.dart` |
| `U-F26-S1-mock-toggle-checkbox` | Unit | Mock engine toggles checkbox | ✅ `f26_s1_form_field_test.dart` |
| `I-F26-S1-insert-form-text-from-insert-ribbon` | Integration | Insert → Form Field → text | ✅ `f26_s1_form_field_test.dart` |
| `I-F26-S1-insert-checkbox-from-dialog` | Integration | Dialog checkbox insert | ✅ `f26_s1_form_field_test.dart` |
| `S-F26-S1-form-field-insert-churn` | Stress | 200 form field inserts | ✅ `f26_s1_insert_form_field.rs` (ignored) |
| `S-F26-S1-form-field-docx-roundtrip-churn` | Stress | 40 fields DOCX round-trip | ✅ `f26_s1_form_field_roundtrip.rs` (ignored) |
| `S-F26-S1-form-field-churn` | Stress | 100 inserts + 50 toggles (Flutter) | ✅ `f26_s1_form_field_test.dart` |

**Exit:** Insert Form Field from Insert ribbon; values update via `SetFormFieldValue`; DOCX round-trips `FORMTEXT` / `FORMCHECKBOX`.

### F26.S2 — Mail merge ✅

**Deliverables:** CSV data source → `MERGEFIELD` / `«Name»` replacement → generated documents; Mailings ribbon Start / Insert Field / Finish & Merge.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F26-S2-merge-field-replace` | Unit | `«Name»` → row value | ✅ `f26_s2_merge_field_replace.rs` |
| `U-F26-S2-merge-field-unbound-placeholder` | Unit | Unbound field displays `«Name»` | ✅ `f26_s2_merge_field_replace.rs` |
| `U-F26-S2-guillemet-text-replace` | Unit | Literal `«Name»` in text replaced | ✅ `f26_s2_merge_field_replace.rs` |
| `U-F26-S2-generate-documents-from-csv` | Unit | CSV → N cloned documents | ✅ `f26_s2_merge_field_replace.rs` |
| `U-F26-S2-insert-merge-field` | Unit | InsertMergeField command | ✅ `f26_s2_mail_merge.rs` |
| `U-F26-S2-apply-mail-merge-row-command` | Unit | ApplyMailMergeRow replaces field | ✅ `f26_s2_mail_merge.rs` |
| `I-F26-S2-merge-field-docx-roundtrip` | Integration | DOCX MERGEFIELD preserve | ✅ `f26_s2_merge_field_roundtrip.rs` |
| `I-F26-S2-merge-then-export-plaintext` | Integration | Merged value in DOCX export | ✅ `f26_s2_merge_field_roundtrip.rs` |
| `U-F26-S2-parse-csv` | Unit | Flutter CSV parse | ✅ `f26_s2_mail_merge_test.dart` |
| `U-F26-S2-mock-merge-field-replace` | Unit | Mock engine «Name» → value | ✅ `f26_s2_mail_merge_test.dart` |
| `I-F26-S2-start-mail-merge-from-ribbon` | Integration | Mailings → Start Mail Merge | ✅ `f26_s2_mail_merge_test.dart` |
| `I-F26-S2-insert-and-finish-merge` | Integration | Insert field + Finish & Merge | ✅ `f26_s2_mail_merge_test.dart` |
| `S-F26-S2-mail-merge-generate-churn` | Stress | 500-row generate | ✅ `f26_s2_mail_merge.rs` (ignored) |
| `S-F26-S2-merge-field-docx-churn` | Stress | Multi-field DOCX round-trip | ✅ `f26_s2_merge_field_roundtrip.rs` (ignored) |
| `S-F26-S2-mail-merge-churn` | Stress | 200-row Flutter churn | ✅ `f26_s2_mail_merge_test.dart` |

**Exit:** Mailings loads CSV; insert merge fields; Finish & Merge replaces `«Name»` / MERGEFIELD with row values; DOCX round-trips unbound fields.

### F26.S3 — WASM plugin host ✅

**Deliverables:** `tw-plugin` wasmtime sandbox (no WASI FS/network); capability gates on host imports; install/enable/disable/invoke lifecycle; Review → Plugins UI.

| Test ID | Type | Spec | Status |
|---------|------|------|--------|
| `U-F26-S3-capability-denied-without-grant` | Unit | Edit plugin denied without `document.edit` | ✅ `f26_s3_plugin_host.rs` |
| `U-F26-S3-wasm-edit-with-grant` | Unit | Granted plugin inserts undoable text | ✅ `f26_s3_plugin_host.rs` |
| `U-F26-S3-host-require-gate` | Unit | Host `require` denies Network by default | ✅ `f26_s3_plugin_host.rs` |
| `I-F26-S3-read-plugin-paragraph-count` | Integration | Read WAT returns paragraph count | ✅ `f26_s3_plugin_host.rs` |
| `I-F26-S3-lifecycle-enable-disable` | Integration | Disable blocks invoke; enable restores | ✅ `f26_s3_plugin_host.rs` |
| `U-F26-S3-capability-denied` | Unit | Flutter registry denies edit | ✅ `f26_s3_plugins_test.dart` |
| `U-F26-S3-capability-granted-invoke` | Unit | Flutter registry allows edit | ✅ `f26_s3_plugins_test.dart` |
| `I-F26-S3-plugins-dialog-from-review` | Integration | Review → Plugins → install/run | ✅ `f26_s3_plugins_test.dart` |
| `I-F26-S3-capability-denied-in-dialog` | Integration | Read-only install → Run shows denial | ✅ `f26_s3_plugins_test.dart` |
| `S-F26-S3-plugin-invoke-churn` | Stress | 200 wasmtime edit invokes | ✅ `f26_s3_plugin_host.rs` (ignored) |
| `S-F26-S3-sandbox-compile-churn` | Stress | 100 sandbox compiles | ✅ `f26_s3_plugin_host.rs` (ignored) |
| `S-F26-S3-plugin-registry-churn` | Stress | 200 Flutter registry install/invoke | ✅ `f26_s3_plugins_test.dart` |

**Exit:** Sample WASM plugin reads document / inserts text via host imports; capability denial when `document.edit` not granted; Plugins dialog manages lifecycle.

---

## F27 — Cloud Features

**Scope:** Sync, backup, document history, sharing, online editing.

### Baseline status: **Missing**.

**Dependencies:** F20 (real-time), auth (P6).

### F27.S1 — Document backup upload

### F27.S2 — Sync conflict resolution

### F27.S3 — Share link and permissions

### F27.S4 — Online editing session

**Deliverables:** Ties to F20 CRDT + cloud backend.

---

## F28 — AI Features

**Scope:** Writing assistant (rewrite, grammar, tone, expand/shorten, translate, summarize); document intelligence (chat, action items, explain, FAQs); content generation; visual assistance; smart editing (auto-format, layout fix, headings, TOC, citations).

### Baseline status: **Complete** — F28.S1–S6 shipped (providers, rewrite, chat/RAG, generation, visual assist, smart edit).

**Dependencies:** ADR-0010; all edits via `Command`.

### F28.S1 — Production providers ✅

**Deliverables:** OpenAI / Gemini / local llama.cpp HTTP adapters + `HybridRouter` / `AiService`; Flutter `AiClient` facade + Review → AI Settings (routing mode). No real network in CI (`MockHttpClient` / `MockAiHttpClient`).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F28-S1-hybrid-router-local` | Unit | Grammar route → local (`llama_cpp`) — Rust + Flutter |
| `U-F28-S1-hybrid-router-cloud` | Unit | Long summarize → cloud (`openai`) — Rust + Flutter |
| `U-F28-S1-openai-adapter-parses-response` | Unit | OpenAI chat JSON → text |
| `I-F28-S1-service-summarize-via-openai` | Integration | `AiService.summarize` via routed OpenAI |
| `I-F28-S1-service-grammar-via-llama` | Integration | `correct_grammar` via local llama |
| `I-F28-S1-ai-settings-from-review` | Integration | Review → AI Settings dialog; mode persists |
| `I-F28-S1-switch-routing-without-restart` | Integration | Always Cloud / Always Local without restart |
| `S-F28-S1-router-and-complete-churn` | Stress | 300 route+complete cycles (Rust `#[ignore]` + Flutter) |

**Evidence:** `crates/tw-ai/tests/f28_s1_production_providers.rs`; `app/test/f28_s1_ai_providers_test.dart`; Review tab `ai_settings`.

### F28.S2 — Writing assistant actions ✅

**Deliverables:** Rewrite selection → transactional `DeleteRange` + `InsertText` (single undo); Flutter Review → Rewrite dialog (Accept/Discard).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F28-S2-replace-commands-delete-then-insert` | Unit | Command list is DeleteRange then InsertText — Rust + Flutter |
| `U-F28-S2-apply-text-suggestion-replaces-run` | Unit | `apply_text_suggestion` mutates run text |
| `U-F28-S2-rewrite-routes-local` | Unit | Rewrite task prefers local provider (Flutter) |
| `I-F28-S2-rewrite-selection` | Integration | AI rewrite → Accept replaces text; undo restores — Rust + Flutter |
| `I-F28-S2-rewrite-discard-keeps-text` | Integration | Discard leaves document unchanged |
| `S-F28-S2-rewrite-apply-undo-churn` | Stress | 200 rewrite+apply+undo cycles (Rust `#[ignore]` + Flutter) |

**Evidence:** `crates/tw-ai/tests/f28_s2_rewrite_selection.rs`; `tw-edit::replace_run_range`; `app/test/f28_s2_rewrite_selection_test.dart`; Review tab `ai_rewrite`.

### F28.S3 — Document chat / RAG ✅

**Deliverables:** Paragraph chunking + keyword RAG index; chat session with paragraph-id citations; Flutter Review → Ask AI dialog.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F28-S3-chunk-document-by-paragraph` | Unit | One chunk per non-empty paragraph — Rust + Flutter |
| `U-F28-S3-rag-retrieve-by-keywords` | Unit | Query ranks matching paragraphs |
| `U-F28-S3-chat-routes-local-for-small-context` | Unit | Chat task prefers local for small context (Flutter) |
| `I-F28-S3-chat-returns-paragraph-citations` | Integration | Ask → assistant message cites paragraph id — Rust |
| `I-F28-S3-chat-from-review-with-citations` | Integration | Review → Ask AI → Send; citation chip — Flutter |
| `I-F28-S3-chat-fallback-retrieved-citations` | Integration | No `[[cite:]]` markers → retrieved chunks cited |
| `S-F28-S3-chunk-retrieve-chat-churn` | Stress | 200 retrieve+ask cycles (Rust `#[ignore]` + Flutter) |

**Evidence:** `crates/tw-ai/tests/f28_s3_document_chat.rs`; `app/test/f28_s3_document_chat_test.dart`; Review tab `ai_chat`.

### F28.S4 — Content generation ✅

**Deliverables:** Generate outline / minutes / report via `AiService.generate`; parse markdown into a new `Document` (Heading 1/2 styles); Flutter Review → Generate → Open as new document.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F28-S4-document-from-markdown-applies-headings` | Unit | `#` / `##` → Heading 1/2 — Rust + Flutter |
| `U-F28-S4-content-kind-prompt` | Unit | Outline/minutes/report prompt text |
| `U-F28-S4-generate-routes-local-for-small-context` | Unit | Generate prefers local for small context (Flutter) |
| `I-F28-S4-generate-outline-new-document` | Integration | Generate outline → new document body — Rust + Flutter |
| `I-F28-S4-generate-minutes-new-document` | Integration | Minutes structure materializes — Rust |
| `I-F28-S4-generate-minutes-api` | Integration | Flutter generate + openGeneratedDocument |
| `S-F28-S4-generate-churn` | Stress | 150 generate cycles (Rust `#[ignore]` + Flutter) |

**Evidence:** `crates/tw-ai/tests/f28_s4_content_generation.rs`; `app/test/f28_s4_content_generation_test.dart`; Review tab `ai_generate`.

### F28.S5 — Visual assistance ✅

**Deliverables:** AI suggests table / diagram / timeline (JSON); insert via `InsertTable` / `InsertDiagram` Commands (timeline → 1×N table); Flutter Review → Visual dialog (Suggest → Insert).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F28-S5-parse-table-suggestion` | Unit | JSON table → VisualKind::Table — Rust + Flutter |
| `U-F28-S5-parse-diagram-and-timeline` | Unit | hierarchy diagram + timeline stages |
| `U-F28-S5-suggestion-to-commands` | Unit | Commands are InsertTable / InsertDiagram |
| `I-F28-S5-suggest-and-insert-table` | Integration | Suggest → apply → table block; undo — Rust + Flutter |
| `I-F28-S5-insert-diagram-and-timeline-blocks` | Integration | Diagram + timeline blocks in session — Rust |
| `I-F28-S5-insert-diagram-and-timeline-api` | Integration | Flutter applyVisualSuggestion |
| `S-F28-S5-suggest-insert-undo-churn` | Stress | 120 suggest+insert(+undo) cycles (Rust `#[ignore]` + Flutter) |

**Evidence:** `crates/tw-ai/tests/f28_s5_visual_assistance.rs`; `app/test/f28_s5_visual_assistance_test.dart`; Review tab `ai_visual`.

### F28.S6 — Smart editing ✅

**Deliverables:** Heuristic (+ optional AI) smart-edit plan: heading suggestions, auto-format notes, TOC draft; apply only on Accept via `ApplyParagraphStyle` + `InsertTableOfContents`.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F28-S6-heuristics-suggest-headings-and-notes` | Unit | Messy doc → headings + format notes — Rust + Flutter |
| `U-F28-S6-parse-ai-plan-overrides-indices` | Unit | AI JSON indices override heuristics |
| `U-F28-S6-smart-edit-commands` | Unit | Commands include ApplyParagraphStyle + InsertTableOfContents |
| `I-F28-S6-apply-headings-and-toc` | Integration | Accept → H1 + TOC; heading undo — Rust |
| `I-F28-S6-smart-edit-from-review` | Integration | Review → Smart Edit → Accept — Flutter |
| `I-F28-S6-suggest-smart-edit-via-ai` | Integration | AI refine plan — Rust + Flutter |
| `S-F28-S6-analyze-apply-undo-churn` | Stress | 150 analyze+apply(+undo) cycles (Rust `#[ignore]` + Flutter) |

**Evidence:** `crates/tw-ai/tests/f28_s6_smart_editing.rs`; `app/test/f28_s6_smart_editing_test.dart`; Review tab `ai_smart_edit`.

**Out of scope:** Unsupervised auto-save of AI changes without user accept.

---

## CI gates by wave

| Wave | Required tests before merge |
|------|----------------------------|
| W0 | `cargo test -p tw-edit -p tw-docx --test corpus_render --test tier_a_numbering_styles`; `flutter test` stage2 editing |
| W1 | Above + find/style list tests as added |
| W2 | Above + table/image/HF integration tests |
| W3 | Reference + print smoke tests |
| W4 | Passthrough tests for diagram/chart parts |
| W5 | `tw-ai`, `tw-crdt` integration; security smoke |

---

## Document maintenance

When implementing a stage:

1. Add Rust/Flutter tests using the **exact Test ID** from this doc (rename existing tests if needed).
2. Update **Baseline status** table for that feature.
3. Update [long-tail-gaps.md](long-tail-gaps.md) if the stage closes a listed gap.
4. Do not mark roadmap exit criteria green until this doc’s stage exit checklist passes.

*Last updated: 2026-08-10 — phase completion rollup; baselines aligned to shipped F01–F19, F21–F26, F28 stages (F20/F27 open).*
