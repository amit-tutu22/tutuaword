# UI Functionality Audit

This document is the single source of truth for **Flutter ribbon, menu, and status-bar wiring** — what actually works today, what is partially wired, and what is an intentional placeholder. Use it to catch controls that *look* enabled but do nothing (the font/size dropdown bug class).

**Last reviewed:** 2026-08-10  
**Scope:** `app/lib/ui/**`, `app/lib/editor/editor_controller.dart`, `app/lib/editor/controllers/**`, `app/lib/editor/editor_menu.dart`

Related: [Long-Tail Gaps](long-tail-gaps.md) covers deferred **Rust/backend** work; this doc covers **UI ↔ engine** wiring.

---

## Executive summary

| Category | Approx. count | Risk |
|----------|---------------|------|
| **P0 — Broken / misleading** (looks enabled, fails silently) | 0 | — |
| **P1 — Partial** (write OK, read/sync missing; or engine exists, UI not wired) | ~1 | Low |
| **P2 — Intentional placeholders** (disabled / grayed) | ~5 | Low |
| **Working end-to-end** | ~90 controls | — |

P0 remains **0**: `rg 'onPressed: \(\) \{\}' app/lib` and `rg 'onSelected: \(\) \{\}' app/lib` both return no matches. Primary ribbon tabs have **no hard-coded `onPressed: null` stubs**; remaining disabled controls are **contextual** (e.g. image tools without selection, Finish Mail Merge without data).

### Next release (deferred from current UAT)

| Area | Notes |
|------|-------|
| **F20 Collaboration** | CRDT / live co-editing — requires online |
| **F27 Cloud** | Sync, backup, share links |

**Shipped this release:** Endnote, Table of Figures, Envelopes/Labels, mail Rules (Next Record + IF field), PDF structural bold/italic fonts, spell suggestions UX, smart AI features (consistency checker, audience rewrite, auto alt-text). **VBA:** macro parts preserved on DOCX passthrough; execution not supported.

### UAT scope by parity layer

| Layer | UAT-ready? | Notes |
|-------|------------|-------|
| L1 DOCX | Yes (scoped) | Real DOCX open/edit/save; corpus gates; VBA preserve |
| L2 Layout | Partial | Core pagination OK; Word baseline parity not claimed |
| L3 Editing | Yes | Typing, format, undo, most ribbon edits |
| L4 Tables/images | Yes (scoped) | Insert/merge/common wrap; EMF/OLE placeholders |
| L5 Review | Yes (scoped) | Spell + suggestions with chip apply, TC accept/reject/all, comments pane |
| L6 Automation API | No | Internal FFI only until F26.S4 |
| L7 Macro preservation | Yes (test) | Preserve on save; no execution |
| L8 Enterprise | No | IRM / tenant policy deferred |
| L9 VBA execution | No | Explicit non-goal |

---

## Three UI anti-patterns

All three are **resolved** as of 2026-08-07. The fix rules stay as standing review criteria for new controls.

### 1. Empty no-ops (highest priority) — resolved

Controls that appear enabled but call an empty handler. Same class of bug as the pre-fix font dropdown (`onPressed: () {}`).

| Location | Control | Code | Status |
|----------|---------|------|--------|
| Home → Styles | **Normal** | `home_tab.dart` — `onPressed: controller.applyNormalStyle`, `selected:` reads `activeParagraphStyle` | Fixed |
| Ribbon | **Share** | `ribbon.dart` — `onPressed: null` + `kComingSoonTooltip` | Disabled (P2) |
| Title bar | **Home (QAT)** | `title_bar.dart` — `onPressed: null` | Disabled (P2) |
| Edit menu | **Undo** | `editor_menu.dart:195` — `onSelected: controller.undo` | Fixed |
| Edit menu | **Redo** | `editor_menu.dart:204` — `onSelected: controller.redo` | Fixed |

**Fix rule:** Never ship `onPressed: () {}` or `onSelected: () {}` on enabled-looking controls. Use `onPressed: null` (disabled styling) or wire to `EditorController`.

### 2. TextField-only clipboard (glyph mode gap) — resolved

The TextField fallback was removed in R2.4. `EditorController.textController` now returns `null` unconditionally and `attachTextEditor` / `detachTextEditor` are no-ops; there is no `TextEditingController` left in the editing path.

| Control | Path today |
|---------|-----------|
| Cut / Copy / Paste | DocRange + engine (`cutSelection` / `copySelection` / `paste`) |
| Select All | DocRange + engine (`selectAll`) |
| Paste and Match Style | `controller.paste(plainText: true)` |

**Fix rule:** Route clipboard through glyph selection (`DocRange` anchor/focus) and engine insert/delete — never through a widget-local text controller.

### 3. Write-only ribbon state (read path missing) — resolved

`FormattingController.syncFromCaret()` reads char and para format back from the engine (`fetchCaretFormat`) and repopulates bold, italic, underline, strikethrough, sub/superscript, all/small caps, hidden, ligatures, font family, font size, font color, highlight, alignment, indent, and style name.

It is invoked from:

| Trigger | Call site |
|---------|-----------|
| Any selection/caret change (hit-test, arrow move, selection update/end, select-all, word select) | `editor_controller.dart:50` → `selection_controller.dart` (7 `onSelectionChanged()` sites) |
| Undo / Redo | `document_session_controller.dart:503,516` |
| Recover / Open / Import | `document_session_controller.dart:177,233,295` |
| After a char-format apply / clear formatting | `formatting_controller.dart:139,314` |
| After a para-format apply (alignment, indent) | `formatting_controller.dart:158` |

**Fix rule:** On caret move / hit-test / refresh, query the engine for char/para format at caret and update ribbon toggles and dropdowns.

---

## Tab-by-tab status

### Home (`app/lib/ui/ribbon_tabs/home_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Paste | **Working** | Engine paste via DocRange caret |
| Cut / Copy | **Working** | DocRange + engine in glyph mode (R2.4) |
| Format Painter | Working | Pickup / paint next selection |
| Font family dropdown | **Working** | Menu + `setFontFamily` (fixed 2026-08-05) |
| Font size dropdown | **Working** | Menu + `setFontSize` |
| Increase / decrease font | Working | |
| Change case | Working | Menu → `applyChangeCase` |
| Clear formatting | **Working** | `clearFormatting` → `clearFormatAsync` + `syncFromCaret` |
| Bold / Italic / Underline | **Working** | Apply + read sync via `syncFromCaret` |
| Strikethrough / Sub / Super | **Working** | Apply + read sync via `syncFromCaret` |
| All caps / Small caps / Hidden / Ligatures | **Working** | Apply + read sync via `syncFromCaret` |
| Font color / Highlight | **Working** | `ribbon_color_picker.dart` |
| Bullets / Numbering | Working | |
| Decrease / increase indent | **Working** | `decreaseIndent` / `increaseIndent` |
| Sort | Working | Menu → `sortParagraphs` |
| Show ¶ (eye) | Working | Toggle formatting marks |
| Align L/C/R/Justify | **Working** | Apply + read sync on caret move |
| Line spacing / Shading & borders | Working | Home ribbon dialogs |
| Style: Normal / No Spacing / Headings | Working | `applyParagraphStyle` |
| Styles Pane | Working | `toggleStyleInspector` |
| Add-ins | Working | Opens Plugins manager (F26.S3) |

### Insert (`insert_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Table | Working | `insertTable` |
| Pictures | Working | `insertImage` |
| Page Break | **Working** | `insertPageBreak` → `insertPageBreakAtAsync` → `tw_insert_page_break` |
| Cover Page | Working | Dialog → title/subtitle/author + page break |
| Change / Rotate / Caption / Compress picture | Working | Enabled when an image is selected (tooltip: select first) |
| Shapes, Header, Footer, Page Number, Text Box, Symbol | Working | See feature phases; audit row was stale |

### Design (`design_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Themes gallery | Working | `applyDocumentTheme` |
| Watermark / Page Color / Page Borders | Working | |

### Layout (`layout_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Page Setup / Breaks / Table / Wrap | Working | |
| Indent −/+ | Working | `decreaseIndent` / `increaseIndent` |
| Space Before / After | Working | Nudge +6 pt via `increaseSpaceBefore` / `increaseSpaceAfter` |
| Tabs | Working | `showTabStopsDialog` |

### References (`references_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| TOC / Footnote / Citation / Bibliography / Bookmark / Cross-ref / Index | Working | |
| Insert Caption | Working | `insertSelectedImageCaption` when image selected |

### Mailings (`mailings_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Start / Finish Mail Merge / Insert Merge Field | Working | Finish disabled until CSV loaded |

### Review (`review_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Spelling & Grammar | Working | `proofDocument` |
| Track Changes | Working | Toggle flag |
| Accept / Reject / Next / Previous | Working | Per-change at caret + navigation |
| Export PDF | Working | `controller.exportPdf` |
| Translate / Thesaurus / Language | Working | AI translate + thesaurus + proofing language dialog |
| New Comment | Working | Dialog → `insertComment` |
| Compare | Working | File picker → `compareWithDocumentPicker` |
| Restrict Editing | Working | `toggleRestrictEditing` |

### View (`view_tab.dart`) + Status bar (`status_bar.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Ruler | Working | `toggleRuler` |
| Navigation Pane | Working | `toggleNavigationPane` |
| Print Layout / Print Preview | Working | `setPrintLayout` / `setPrintPreviewMode` |
| Read Mode / Web Layout | Working | Read-only immersive / continuous canvas |
| Zoom / One Page / Multiple Pages | Working | Dialog + fit-to-viewport; status bar slider also works |
| New Window / Arrange All / Split | Working | In-app second window; arrange enables split; split toggles dual panes |

### Title bar (`title_bar.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Save | Working | |
| Undo / Redo | Working | |
| Print | Working | |
| Search (Find) | Working | `openFindPane` |

### macOS menu (`editor_menu.dart`)

| Item | Status | Notes |
|------|--------|-------|
| File Open / Save / Save As | Working | |
| Edit Undo / Redo | **Working** | `controller.undo` / `redo` |
| Edit Cut / Copy / Paste / Delete / Select All | **Working** | DocRange + engine (R2.4) |
| Tools Spell Check / Track Changes | Working | |
| View | P2 | Platform fullscreen only |

---

## Engine vs UI gap

Backend capability exists (or partially exists) but Flutter has no caller:

| Feature | Rust / FFI | Flutter |
|---------|------------|---------|
| Proofing language → engine spell dict | Partial | UI status + translate default only |

Closed since the last review: page break, paragraph indent, PDF export, format-at-caret read, Design/Layout/References/Mailings wiring, View modes, Cover Page, Change Case, Sort, Compare file picker, comment dialog.

See [Long-Tail Gaps](long-tail-gaps.md) for PDF fonts, Hunspell, plugins, AI, etc.

---

## Working end-to-end (reference)

Verified paths that round-trip through the Rust engine in glyph mode:

- Typing, Enter, Backspace, arrow keys, multi-page scroll
- Per-page display lists (after scroll/cache fixes)
- Bold, italic, underline, strikethrough, sub/superscript, caps, hidden, ligatures — apply **and** read back at caret
- Font family/size, font color, highlight — apply **and** read back at caret
- Alignment, indent (ribbon and Tab / Shift+Tab), clear formatting
- Bullets, numbered lists, Normal style, Heading 1
- Table insert, image placeholder, page break
- Cut / copy / paste / paste-plain / delete / select-all via DocRange + engine
- Open / Save / Save As (DOCX, ODT, MD, HTML), PDF export
- Undo / Redo via title bar **and** Edit menu
- Spell check, track-changes flag, accept/reject all revisions
- Ruler, navigation pane, zoom slider, print layout / print preview toggle

---

## Recommended fix order

### Phase A — Deceptive UI (same severity as dropdown bug) — **done**

1. ~~Edit menu Undo / Redo → `controller.undo` / `redo`~~
2. ~~Normal style → apply body style or disable + remove false `selected: true`~~
3. ~~Share / QAT Home → disable or implement~~
4. ~~Glyph-mode clipboard (cut / copy / paste / select-all)~~

### Phase B — Ribbon truthfulness — **done**

5. ~~Sync font / size / bold / italic / alignment from engine at caret~~
6. ~~Clear formatting button~~
7. ~~Increase / decrease indent → `_applyParaFormatJson`~~
8. ~~Insert → Page Break (FFI + `EditorController.insertPageBreak`)~~

### Phase C — Placeholders

9. Design / Layout / References / Mailings — keep disabled or add “Coming soon” tooltip (done for Design)
10. Per-change track-change navigation (next / previous / accept one) — see [Long-Tail Gaps](long-tail-gaps.md)
11. ~~Re-sync ribbon immediately after `_applyParaFormatJson`, matching the char-format path~~ — done 2026-08-07

---

## Review checklist (for new ribbon controls)

Before merging UI work, verify each control:

1. **Handler** — not `null` and not `() {}` unless intentionally disabled
2. **Mode** — works in glyph mode, not only TextField fallback
3. **Read path** — displayed state reflects caret/selection (after import, undo, click)
4. **Engine** — if Rust command exists, FFI + `EditorController` method exist
5. **Test** — widget or integration test in `app/test/ribbon_integration_test.dart` or `stage2_editing_integration_test.dart`

### Grep commands for audits

```bash
# Empty no-ops (must be zero for enabled controls)
rg 'onPressed: \(\) \{\}' app/lib
rg 'onSelected: \(\) \{\}' app/lib

# All disabled placeholders (document each)
rg 'onPressed: null' app/lib/ui
```

---

## File index

| File | Role |
|------|------|
| `app/lib/ui/ribbon_tabs/home_tab.dart` | Home ribbon |
| `app/lib/ui/ribbon_tabs/insert_tab.dart` | Insert ribbon |
| `app/lib/ui/ribbon_tabs/design_tab.dart` | Design ribbon |
| `app/lib/ui/ribbon_tabs/layout_tab.dart` | Layout ribbon |
| `app/lib/ui/ribbon_tabs/references_tab.dart` | References ribbon |
| `app/lib/ui/ribbon_tabs/mailings_tab.dart` | Mailings ribbon |
| `app/lib/ui/ribbon_tabs/review_tab.dart` | Review ribbon |
| `app/lib/ui/ribbon_tabs/view_tab.dart` | View ribbon |
| `app/lib/ui/ribbon_widgets.dart` | `RibbonDropdown`, buttons, style cards |
| `app/lib/ui/ribbon_color_picker.dart` | `RibbonColorButton`, `WordColorPalettePanel`, theme/highlight swatches |
| `app/lib/ui/ribbon.dart` | Tab host + Share button |
| `app/lib/ui/title_bar.dart` | QAT + save/undo/print |
| `app/lib/ui/status_bar.dart` | Zoom + view mode |
| `app/lib/editor/editor_controller.dart` | UI ↔ engine bridge |
| `app/lib/editor/editor_menu.dart` | macOS menu bar |
| `app/lib/editor/glyph_editor_surface.dart` | Keyboard + pointer in glyph mode |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-08-05 | Initial audit; font/size dropdowns fixed with real menus |
| 2026-08-07 | R2.4: EditorController decomposed; TextField fallback removed; P0 = 0 |
| 2026-08-05 | Phase A: Edit Undo/Redo wired; Normal style; Share/QAT Home disabled; glyph clipboard |
| 2026-08-05 | Phase B: Caret format sync; clear formatting; indent; page break FFI |
| 2026-08-05 | Phase C: Coming soon tooltips on placeholders; PDF export UI; track-change accept/reject documented in Long-Tail Gaps |
| 2026-08-07 | Accuracy pass against code + green suites (Rust 401, Flutter 179): R2.4 controller decomposition (`FormattingController.syncFromCaret`) closes the write-only ribbon gap; F03.S2 font color / highlight landed via `ribbon_color_picker.dart`; clear formatting, indent, and page break confirmed wired. Anti-patterns 1–3 marked resolved; P1 count corrected ~22 → ~2 |
| 2026-08-10 | Release closeout: wired Layout indent/spacing, Home Change Case/Sort/No Spacing, Review Comment/Compare, References Caption; removed Endnote/ToF/Envelopes/Labels/Rules/chevrons/More; F20/F27 marked next release |
