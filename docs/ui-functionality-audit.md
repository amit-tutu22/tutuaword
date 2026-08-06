# UI Functionality Audit

This document is the single source of truth for **Flutter ribbon, menu, and status-bar wiring** — what actually works today, what is partially wired, and what is an intentional placeholder. Use it to catch controls that *look* enabled but do nothing (the font/size dropdown bug class).

**Last reviewed:** 2026-08-05  
**Scope:** `app/lib/ui/**`, `app/lib/editor/editor_controller.dart`, `app/lib/editor/editor_menu.dart`

Related: [Long-Tail Gaps](long-tail-gaps.md) covers deferred **Rust/backend** work; this doc covers **UI ↔ engine** wiring.

---

## Executive summary

| Category | Approx. count | Risk |
|----------|---------------|------|
| **P0 — Broken / misleading** (looks enabled, fails silently) | 7 | High |
| **P1 — Partial** (write OK, read/sync missing; or engine exists, UI not wired) | ~22 | Medium |
| **P2 — Intentional placeholders** (disabled / grayed) | ~50+ | Low |
| **Working end-to-end** | ~35 controls | — |

---

## Three UI anti-patterns

### 1. Empty no-ops (highest priority)

Controls that appear enabled but call an empty handler. Same class of bug as the pre-fix font dropdown (`onPressed: () {}`).

| Location | Control | Code | Expected behavior |
|----------|---------|------|-------------------|
| Home → Styles | **Normal** | `home_tab.dart` — `selected: true`, `onPressed: () {}` | Apply Normal style; reflect actual style at caret |
| Ribbon | **Share** | `ribbon.dart` — `onPressed: () {}` | Share/export dialog, or disable until implemented |
| Title bar | **Home (QAT)** | `title_bar.dart` — `onPressed: () {}` | New doc / focus Home tab, or disable |
| Edit menu | **Undo** | `editor_menu.dart` — `onSelected: () {}` | `controller.undo` (title bar already wired) |
| Edit menu | **Redo** | `editor_menu.dart` — `onSelected: () {}` | `controller.redo` |

**Fix rule:** Never ship `onPressed: () {}` or `onSelected: () {}` on enabled-looking controls. Use `onPressed: null` (disabled styling) or wire to `EditorController`.

### 2. TextField-only clipboard (glyph mode gap)

Cut, copy, paste, and select-all use `_textController` only:

```dart
// editor_controller.dart — selectedText
if (editor == null || !editor.selection.isValid || editor.selection.isCollapsed) {
  return '';
}
```

In **glyph mode** (`usesGlyphRendering`), `_textController` is null → clipboard ribbon and menu items silently no-op.

| Control | Affected paths |
|---------|----------------|
| Cut / Copy / Paste | Home ribbon, Edit menu |
| Select All | Edit menu |
| Paste and Match Style | Edit menu |

**Fix rule:** Route clipboard through glyph selection (`_selAnchorRunId` / `_selFocusRunId`) and engine insert/delete when `usesGlyphRendering`.

### 3. Write-only ribbon state (read path missing)

Bold, italic, underline, strikethrough, sub/superscript, font family, font size, and alignment update local controller fields and **apply** format to the engine, but the ribbon **does not read** format back from the caret after navigation, import, or undo.

**Fix rule:** On caret move / hit-test / `_refreshFromEngine`, query engine or layout for char/para format at caret and update ribbon toggles and dropdowns.

---

## Tab-by-tab status

### Home (`app/lib/ui/ribbon_tabs/home_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Paste | Partial | TextField mode only |
| Cut / Copy | P0 | TextField only in glyph mode |
| Format Painter | P2 | `onPressed: null` |
| Font family dropdown | **Working** | Menu + `setFontFamily` (fixed 2026-08-05) |
| Font size dropdown | **Working** | Menu + `setFontSize` |
| Increase / decrease font | Working | |
| Change case | P2 | Disabled |
| Clear formatting | P1 | Disabled; should reset char/para format |
| Bold / Italic / Underline | Partial | Write OK; P1 read sync |
| Strikethrough / Sub / Super | Partial | Write OK; P1 read sync |
| Font color / Highlight | P2 | Disabled |
| Bullets / Numbering | Working | |
| Decrease / increase indent | P1 | Disabled; `tw_apply_para_format` exists |
| Sort / Show ¶ | P2 | Disabled |
| Align L/C/R/Justify | Partial | Write OK; P1 read sync |
| Line spacing / Shading & borders | P2 | Disabled |
| Style: Normal | **P0** | Empty no-op, always selected |
| Style: No Spacing | P2 | Disabled |
| Style: Heading 1 | Working | |
| Styles gallery chevron / pane | P2 | Disabled |
| Add-ins | P2 | Disabled |

### Insert (`insert_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Table | Working | `insertTable` |
| Pictures | Working | `insertImage` |
| Page Break | P1 | Disabled; `insert_page_break` in `tw-edit`, no FFI |
| Cover Page, Shapes, Video, Header, Footer, Page Number, Text Box, Symbol | P2 | Disabled |

### Design / Layout / References / Mailings

These tabs receive **no** `EditorController` (`ribbon.dart`). Entire tabs are visual shells; all controls use `onPressed: null` (correctly grayed). **P2** until tabs accept controller + engine commands.

### Review (`review_tab.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Spelling & Grammar | Working | `spellCheckDocument` |
| Track Changes | Working | Toggle flag |
| Accept / Reject | P1 | Wired to accept/reject **all** revisions; per-change nav still open |
| Translate, Thesaurus, Language, New Comment, Compare, Restrict Editing | P2 | Disabled |

### View (`view_tab.dart`) + Status bar (`status_bar.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Ruler | Working | `toggleRuler` |
| Navigation Pane | Working | `toggleNavigationPane` |
| Print Layout | P1 | Partial — exits preview only; not full toggle |
| Zoom (View tab) | P1 | Disabled; status bar slider **works** |
| Read / Web / One page / Multiple pages / Split / New window | P2 | Disabled |

### Title bar (`title_bar.dart`)

| Control | Status | Notes |
|---------|--------|-------|
| Save | Working | |
| Undo / Redo | Working | |
| Print | Working | `togglePrintPreview` |
| Home (QAT) | **P0** | Empty no-op |
| Search / More | P2 | Disabled |

### macOS menu (`editor_menu.dart`)

| Item | Status | Notes |
|------|--------|-------|
| File Open / Save / Save As | Working | |
| Edit Undo / Redo | **P0** | Empty no-op (⌘Z / ⇧⌘Z broken from menu) |
| Edit Cut / Copy / Paste / Delete / Select All | Partial | TextField only in glyph mode |
| Tools Spell Check / Track Changes | Working | |
| View | P2 | Platform fullscreen only |

---

## Engine vs UI gap

Backend capability exists (or partially exists) but Flutter has no caller:

| Feature | Rust / FFI | Flutter |
|---------|------------|---------|
| Page break | `tw_edit::insert_page_break` | Not in FFI; Insert tab disabled |
| Paragraph format (indent, spacing) | `tw_apply_para_format` | Not wired on ribbon |
| PDF export | `BridgeCommand::ExportPdf`, `exportPdf()` | No ribbon/menu entry |
| Accept / reject track changes | Track-changes flag only | Review buttons disabled |
| Format at caret (read) | Model + layout | Ribbon does not query |

See [Long-Tail Gaps](long-tail-gaps.md) for PDF fonts, Hunspell, plugins, AI, etc.

---

## Working end-to-end (reference)

Verified paths that round-trip through the Rust engine in glyph mode:

- Typing, Enter, Backspace, arrow keys, multi-page scroll
- Per-page display lists (after scroll/cache fixes)
- Bold, italic, underline, font family/size, alignment **apply**
- Bullets, numbered lists, Heading 1
- Table insert, image placeholder
- Open / Save / Save As (DOCX, ODT, MD, HTML)
- Undo / Redo via **title bar** (not Edit menu)
- Spell check, track-changes flag
- Ruler, navigation pane, zoom slider, print preview toggle

---

## Recommended fix order

### Phase A — Deceptive UI (same severity as dropdown bug)

1. Edit menu Undo / Redo → `controller.undo` / `redo`
2. Normal style → apply body style or disable + remove false `selected: true`
3. Share / QAT Home → disable or implement
4. Glyph-mode clipboard (cut / copy / paste / select-all)

### Phase B — Ribbon truthfulness

5. Sync font / size / bold / italic / alignment from engine at caret
6. Clear formatting button
7. Increase / decrease indent → `_applyParaFormatJson`
8. Insert → Page Break (FFI + `EditorController.insertPageBreak`)

### Phase C — Placeholders

9. Design / Layout / References / Mailings — keep disabled or add “Coming soon” tooltip
10. Track Accept/Reject, PDF export UI, etc. in [Long-Tail Gaps](long-tail-gaps.md)

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
| `app/lib/ui/ribbon_tabs/design_tab.dart` | Design shell |
| `app/lib/ui/ribbon_tabs/layout_tab.dart` | Layout shell |
| `app/lib/ui/ribbon_tabs/references_tab.dart` | References shell |
| `app/lib/ui/ribbon_tabs/mailings_tab.dart` | Mailings shell |
| `app/lib/ui/ribbon_tabs/review_tab.dart` | Review ribbon |
| `app/lib/ui/ribbon_tabs/view_tab.dart` | View ribbon |
| `app/lib/ui/ribbon_widgets.dart` | `RibbonDropdown`, buttons, style cards |
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
| 2026-08-05 | Phase A: Edit Undo/Redo wired; Normal style; Share/QAT Home disabled; glyph clipboard |
| 2026-08-05 | Phase B: Caret format sync; clear formatting; indent; page break FFI |
| 2026-08-05 | Phase C: Coming soon tooltips on placeholders; PDF export UI; track-change accept/reject documented in Long-Tail Gaps |
