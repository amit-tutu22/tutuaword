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

---

## F01 — Document Management

**Scope:** New, Open, Save, Save As, Auto Save, Recent documents, Password protection, Read-only mode, Print, Print Preview, Export PDF/HTML/ODT, Document properties.

### Baseline status

| Capability | Status | Evidence |
|------------|--------|----------|
| New | Implemented | File→New + `tw_new_document` FFI |
| Open / Save / Save As | Implemented | File menu; `.twdoc` + `.docx` Save As |
| Auto Save / Recent | Implemented | Timer autosave + File menu recent (max 10) |
| Password / Read-only | Implemented | Encrypted DOCX rejected; DOCX protection flag |
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

### Baseline status

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

### Baseline status

| Capability | Status |
|------------|--------|
| Font, B/I/U, strike, super/sub | Implemented |
| Color, highlight | Implemented (ribbon pickers; DOCX round-trip) |
| Double underline, spacing, caps, hidden, ligatures | Partial (double underline + char spacing + caps/hidden/liga) |

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

### Baseline status

| Capability | Status |
|------------|--------|
| Align L/C/R/J, indent | Implemented |
| Line/para spacing, tabs | Partial (spacing + tab-stop UI done) |
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

### Baseline status

| Capability | Status |
|------------|--------|
| Bullet / numbered | Implemented |
| Multi-level, restart | Implemented |
| Outline numbering | Implemented |
| Custom | Partial |
| Outline (F19 nav) | Partial |

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

### Baseline status

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

### Baseline status

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

### Baseline status

| Capability | Status |
|------------|--------|
| Import/layout HF blocks | Partial |
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
| `U-F08-S4-linked-export` | Unit | Linked section omits `w:headerReference` |

**Exit (S4):** New sections default to link-to-previous; `SetHeaderFooterLink` toggles per variant; layout resolves headers through the link chain; opening a linked band auto-unlinks and copies content; export skips header refs for linked sections.

---

## F09 — Tables

**Scope:** Insert, delete rows/columns, merge/split cells, AutoFit, borders, shading, sorting, formulas, nested tables.

### Baseline status

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

### Baseline status

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

### Baseline status: **Partial** (`ShapeBlock` import + placeholder render; insert disabled).

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

### F11.S3 — Text boxes and WordArt

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

### Baseline status: **Partial** (Tier C passthrough, preview raster, insert placeholder).

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

### Baseline status: **Partial** (F13.S1–S3 preserve + static preview + editable data).

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

### Baseline status: **Missing**.

### F14.S1 — OMML preserve

| Test ID | Type | Spec |
|---------|------|------|
| `U-F14-S1-omml-passthrough` | Unit | `m:oMath` blocks preserved in package |

### F14.S2 — Equation layout (read-only)

**Deliverables:** Layout math runs as scaled glyphs or image fallback.

### F14.S3 — Equation editor UI

**Deliverables:** Insert equation dialog; build OMML from palette.

### F14.S4 — LaTeX import (optional)

**Deliverables:** LaTeX → OMML subset converter.

---

## F15 — Symbols

**Scope:** Unicode, emoji, currency, mathematical symbols, special characters.

### Baseline status: **Stub** (Symbol button disabled).

### F15.S1 — Symbol dialog

**Deliverables:** Modal grid by category; insert via `InsertText`.

| Test ID | Type | Spec |
|---------|------|------|
| `I-F15-S1-insert-copyright` | Integration | Insert © at caret |

### F15.S2 — Emoji and math symbols

**Deliverables:** Emoji picker; math symbol subset.

### F15.S3 — Recent symbols

**Deliverables:** Last-used list in dialog.

---

## F16 — References

**Scope:** Footnotes, endnotes, TOC, bibliography, citations, index, cross references.

### Baseline status: **Stub** (References ribbon disabled).

**Dependencies:** F06 (heading styles for TOC), F18 (cross-ref targets).

### F16.S1 — Footnotes and endnotes

**Deliverables:** `Footnote` model; layout bottom-of-page band; DOCX parts.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F16-S1-footnote-ref-layout` | Unit | Superscript ref + note body |
| `I-F16-S1-insert-footnote` | Integration | References→Insert Footnote |

### F16.S2 — Table of contents

**Deliverables:** TOC field from Heading styles; page numbers.

### F16.S3 — Citations and bibliography

**Deliverables:** Citation keys; `bibliography.xml` passthrough minimum.

### F16.S4 — Index and cross-references

**Deliverables:** REF fields; bookmark targets (ties F19).

---

## F17 — Review

**Scope:** Spell check, grammar, comments, track changes, compare, restrict editing, accept/reject.

### Baseline status

| Capability | Status |
|------------|--------|
| Spell | Stub (`tw-spell` wordlist) |
| Track changes | Partial (accept/reject all) |
| Comments, grammar, compare | Missing / Stub |

### F17.S1 — Spell check upgrade

**Deliverables:** Hunspell integration; squiggles in display list (optional).

| Test ID | Type | Spec |
|---------|------|------|
| `U-F17-S1-hunspell-suggestions` | Unit | Known misspelling flagged |
| `I-F17-S1-spell-check-menu` | Integration | Review→Spelling lists words |

### F17.S2 — Track changes at caret

**Deliverables:** Accept/Reject single revision; next/previous change.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F17-S2-accept-revision-caret` | Unit | Extends `track_change_resolve.rs` |
| `I-F17-S2-accept-button` | Integration | Review→Accept |

### F17.S3 — Comments

**Deliverables:** `CommentThread` model; margin markers; DOCX `comments.xml`.

### F17.S4 — Grammar, compare, restrict

**Deliverables:** Grammar via AI or LanguageTool; compare two docs; restrict editing flag.

---

## F18 — Search

**Scope:** Find, replace, regex, wildcards, navigation pane, search formatting.

### Baseline status

| Capability | Status |
|------------|--------|
| FindReplace command | Partial |
| UI, regex, format search | Missing |

### F18.S1 — Find UI

**Deliverables:** Find pane; highlight matches; next/previous.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F18-S1-find-case-sensitive` | Unit | `FindReplace` match_case |
| `I-F18-S1-find-pane` | Integration | Ctrl+F opens pane |

### F18.S2 — Replace

| Test ID | Type | Spec |
|---------|------|------|
| `U-F18-S2-replace-all-count` | Unit | Returns replacement count |

### F18.S3 — Regex and wildcards

**Deliverables:** Regex mode in find (Rust `regex` crate).

### F18.S4 — Format search

**Deliverables:** Find bold text / specific style.

---

## F19 — Navigation

**Scope:** Outline view, page thumbnails, bookmarks, hyperlinks, Go To, document map.

### Baseline status

| Capability | Status |
|------------|--------|
| Page thumbnails / nav pane | Partial |
| Outline, bookmarks, hyperlinks | Missing |

### F19.S1 — Thumbnails and page strip (complete)

| Test ID | Type | Spec |
|---------|------|------|
| `I-F19-S1-page-nav-jump` | Integration | Click page N → scroll |

### F19.S2 — Outline view

**Deliverables:** Headings tree from `outline_level` / styles; click → scroll.

### F19.S3 — Bookmarks and hyperlinks

**Deliverables:** `Bookmark`, `HyperlinkTarget` on model; insert/edit link.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F19-S3-hyperlink-roundtrip-docx` | Unit | `w:hyperlink` import/export |

### F19.S4 — Go To dialog

**Deliverables:** Go to page, bookmark, heading.

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

### Baseline status: **Missing** (custom glyph paint; no semantic tree).

### F21.S1 — Semantic document tree

**Deliverables:** Parallel tree: headings, paragraphs, tables for a11y API.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F21-S1-heading-structure` | Unit | H1→H2 order in tree |

### F21.S2 — Keyboard navigation audit

**Deliverables:** Tab order through ribbon; documented shortcuts.

### F21.S3 — Alt text on images

**Deliverables:** `ImageBlock.alt_text`; inspector field.

### F21.S4 — Accessibility checker

**Deliverables:** Rules: missing alt, empty heading, low contrast warning.

### F21.S5 — Read aloud (optional)

**Deliverables:** Platform TTS reads selection.

---

## F22 — Security

**Scope:** Password protection, encryption, digital signatures, IRM, document inspection, remove metadata.

### Baseline status: **Missing** (spec in `architecture/security.md`).

### F22.S1 — Password-protected open

**Deliverables:** Detect encryption; prompt for password (decrypt library TBD).

### F22.S2 — Encrypt on save

### F22.S3 — Document inspector

**Deliverables:** Remove comments, metadata, hidden text.

### F22.S4 — Digital signatures

### F22.S5 — IRM (enterprise)

**Out of scope until P6 enterprise phase.

---

## F23 — File Formats

**Scope:** Import DOCX, DOC, RTF, TXT, ODT, HTML, Markdown; Export DOCX, PDF, ODT, HTML, EPUB, TXT.

### Baseline status

| Format | Import | Export |
|--------|--------|--------|
| DOCX | Partial | Partial |
| ODT, MD, HTML, RTF, TXT | Partial | Partial |
| PDF | — | Partial (structural) |
| DOC, EPUB | Missing | Missing |

### F23.S1 — DOCX hardening (critical path)

**Deliverables:** Corpus gate ≥95% open; Tier A round-trip tests per category.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F23-S1-tier-a-numbering` | Unit | `tier_a_numbering_styles.rs` |
| `I-F23-S1-corpus-render` | Integration | `corpus_render.rs` gate |
| `I-F23-S1-roundtrip-50` | Integration | 50 DOCX round-trip |

**Exit:** Continuous CI corpus + Tier A gates ([risk-mitigation](risk-mitigation.md) S1).

### F23.S2 — PDF VisualMatch

**Deliverables:** Font embedding in `tw-pdf`; images in PDF.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F23-S2-visual-match-rejects-until-ready` | Unit | `PdfFidelity::VisualMatch` errors |
| `U-F23-S2-embed-fonts` | Unit | `/FontFile2` in PDF when enabled |

### F23.S3 — ODT / HTML / MD fidelity

| Test ID | Type | Spec |
|---------|------|------|
| `U-F23-S3-odt-roundtrip` | Unit | `tw-odt` tests |
| `U-F23-S3-html-export-headings` | Unit | H1→`<h1>` |

### F23.S4 — RTF polish and TXT/EPUB

**Deliverables:** RTF import lists/tables; EPUB export (future).

**Out of scope:** Binary `.doc` — document as non-goal.

---

## F24 — Templates

**Scope:** Resume, letter, invoice, brochure, newsletter, business proposal, research paper templates.

### Baseline status: **Stub** (`template_name` field only).

### F24.S1 — Built-in template pack

**Deliverables:** 7 starter `.docx` templates in `assets/templates/`; New from template.

| Test ID | Type | Spec |
|---------|------|------|
| `I-F24-S1-new-from-resume` | Integration | New→Resume opens styled doc |

### F24.S2 — Theme binding

**Deliverables:** Template applies `DocumentTheme`.

### F24.S3 — Save as template

### F24.S4 — Template gallery UI

---

## F25 — Printing

**Scope:** Duplex, multiple pages per sheet, booklet, margins, scaling, print selection.

### Baseline status: **Partial** (preview only).

**Dependencies:** F23.S2 (font-embedded PDF for WYSIWYG).

### F25.S1 — OS print dialog

**Deliverables:** Print via PDF/XPS or platform API on macOS/Windows/Linux.

| Test ID | Type | Spec |
|---------|------|------|
| `I-F25-S1-print-dialog-opens` | Integration | File→Print |

### F25.S2 — Scaling and margins

### F25.S3 — Print selection

### F25.S4 — Duplex, booklet, N-up

**Deliverables:** Platform print attributes where supported.

---

## F26 — Macros and Automation

**Scope:** Macros, scripting, plugin support, mail merge, form fields.

### Baseline status: **Stub** (`tw-plugin` traits only; Mailings disabled).

### F26.S1 — Form fields

**Deliverables:** Plain text / checkbox fields in model; DOCX preserve.

### F26.S2 — Mail merge

**Deliverables:** Data source CSV → generate documents.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F26-S2-merge-field-replace` | Unit | `«Name»` → row value |

### F26.S3 — WASM plugin host

**Deliverables:** `tw-plugin` wasmtime sandbox; capability gates.

### F26.S4 — Scripting API

**Out of scope:** VBA execution — preserve `vbaProject.bin` only.

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

### Baseline status: **Stub** (`tw-ai` mocks only).

**Dependencies:** ADR-0010; all edits via `Command`.

### F28.S1 — Production providers

**Deliverables:** OpenAI/Gemini/local llama adapters replace mocks.

| Test ID | Type | Spec |
|---------|------|------|
| `U-F28-S1-hybrid-router-local` | Unit | Grammar route → local |
| `U-F28-S1-hybrid-router-cloud` | Unit | Summarize route → cloud |

### F28.S2 — Writing assistant actions

**Deliverables:** Rewrite selection → `InsertText`/`DeleteRange` commands.

| Test ID | Type | Spec |
|---------|------|------|
| `I-F28-S2-rewrite-selection` | Integration | AI sidebar replaces text; undo works |

### F28.S3 — Document chat / RAG

**Deliverables:** Chunk document; chat UI; citations to paragraph ids.

### F28.S4 — Content generation

**Deliverables:** Generate outline/minutes/report into new document.

### F28.S5 — Visual assistance

**Deliverables:** Suggest diagram/table/timeline — insert as blocks.

### F28.S6 — Smart editing

**Deliverables:** Auto-format messy doc; suggest headings; TOC draft.

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

*Last updated: feature-phase plan initial authoring.*
