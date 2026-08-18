# Keyboard Shortcuts & Ribbon Focus (F21.S2)

Tutuaword follows Microsoft Word's keyboard map. Everything listed here is
**actually wired**; a test asserts every chord in the code catalog appears in
this document.

Canonical catalog in code: `app/lib/ui/keyboard_shortcuts.dart`
(`kWordChordSpecs` → `kWordChordShortcuts` → `kWiredShortcuts`).

Ctrl and Cmd are both registered for every application chord, so one binary
serves Windows, Linux and macOS. On macOS the `PlatformMenuBar` in
`editor_menu.dart` mirrors the common items; the native menu claims the chord
first, which is why the menu and the `Shortcuts` map can bind the same keys.

## File

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+N | New document |
| Ctrl/Cmd+O | Open document |
| Ctrl/Cmd+S | Save |
| Ctrl/Cmd+Shift+S | Save As |
| Ctrl/Cmd+P | Print |

## Undo and clipboard

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+Z | Undo |
| Ctrl/Cmd+Y | Redo |
| Ctrl/Cmd+Shift+Z | Redo |
| Ctrl/Cmd+X | Cut |
| Ctrl/Cmd+C | Copy |
| Ctrl/Cmd+V | Paste |
| Ctrl/Cmd+Shift+V | Paste keeping text only |
| Ctrl/Cmd+Alt+Shift+V | Paste and Match Style |

## Select, find and navigate

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+A | Select all |
| Ctrl/Cmd+F | Find |
| Ctrl/Cmd+H | Find and Replace |
| Shift+F4 | Find next |
| Ctrl/Cmd+G | Go To |
| F5 | Go To |
| Ctrl/Cmd+Shift+8 | Show/hide formatting marks |

## Character formatting

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+B | Bold |
| Ctrl/Cmd+I | Italic |
| Ctrl/Cmd+U | Underline |
| Ctrl/Cmd+Shift+> | Grow font |
| Ctrl/Cmd+Shift+< | Shrink font |
| Ctrl/Cmd+= | Subscript |
| Ctrl/Cmd+Shift+= | Superscript |
| Ctrl/Cmd+Shift+A | All caps |
| Ctrl/Cmd+Shift+K | Small caps |
| Ctrl/Cmd+Space | Clear character formatting |

## Paragraph formatting

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+L | Align left |
| Ctrl/Cmd+E | Center |
| Ctrl/Cmd+R | Align right |
| Ctrl/Cmd+J | Justify |
| Ctrl/Cmd+M | Increase indent |
| Ctrl/Cmd+Shift+M | Decrease indent |
| Ctrl/Cmd+1 | Single line spacing |
| Ctrl/Cmd+2 | Double line spacing |
| Ctrl/Cmd+5 | 1.5 line spacing |

## Styles

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+Shift+N | Apply Normal style |
| Ctrl/Cmd+Alt+1 | Apply Heading 1 |
| Ctrl/Cmd+Alt+2 | Apply Heading 2 |
| Ctrl/Cmd+Alt+3 | Apply Heading 3 |

## Insert and review

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+K | Insert hyperlink |
| Ctrl/Cmd+Alt+M | New comment |
| Ctrl/Cmd+Shift+E | Toggle track changes |

## Editor (glyph surface)

These keys never reach a `Shortcuts` map: while the document canvas has focus
they are translated by `EditorInputEvent.fromLogicalKey` and dispatched through
`EditorController.handleEditorInput`. Tab is claimed for editing, not UI focus.

| Chord | Action |
|-------|--------|
| Enter | Split paragraph |
| Shift+Enter | Manual line break inside the paragraph |
| Ctrl/Cmd+Enter | Page break |
| Tab | Insert tab, or demote the list item at paragraph start |
| Shift+Tab | Promote list item / decrease indent |
| Backspace | Delete grapheme before the caret |
| Delete | Delete grapheme after the caret |
| Ctrl/Alt+Backspace | Delete word before the caret |
| Ctrl/Alt+Delete | Delete word after the caret |
| Arrow keys | Move caret by character / line |
| Shift+Arrows | Extend the selection |
| Ctrl/Alt+Left, Ctrl/Alt+Right | Move caret by word |
| Home, End | Start / end of line |
| Ctrl+Home, Ctrl+End | Start / end of document |
| Cmd+Left, Cmd+Right | Start / end of line (macOS) |
| Cmd+Up, Cmd+Down | Start / end of document (macOS) |
| Page Up, Page Down | Previous / next screen |

Notes:

- Shift combines with every caret motion above to extend the selection.
- **Shift+Enter** stores `\n` inside the run. Layout treats it as a mandatory
  break and the DOCX exporter writes `<w:br/>`, which is what Word does; a plain
  Enter splits the paragraph instead.
- **Tab in a list** only changes the level when the caret is at the start of the
  list paragraph, matching Word. Elsewhere it inserts a tab character. Shift+Tab
  promotes from anywhere in the item.
- A page is the scroll unit in this viewport, so Page Up / Page Down move one
  page and preserve the caret's column.

## Ribbon keyboard navigation

The ribbon (`WordRibbon`) is wrapped in a `FocusTraversalGroup` with
`ReadingOrderTraversalPolicy`:

1. **Tab strip** — Home → Insert → … → View (Share is disabled and skipped).
2. **Active tab body** — enabled controls left-to-right / top-to-bottom.
3. **Activate** — Space or Enter invokes the focused control (`ActivateIntent`).
4. **Disabled** controls (`onPressed == null`) do not participate in Tab order.

Focus ring uses the active-tab underline color. Mouse hover still works as before.

When the glyph editor has focus, Tab stays an editing key (see above). Click a
ribbon control or tab to move focus into the ribbon traversal group.
