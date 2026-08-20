# Keyboard Shortcuts & Ribbon Focus (F21.S2)

Tutuaword follows Microsoft Word's keyboard map. Everything listed here is
**actually wired**; a test asserts every chord in the code catalog appears in
this document.

Canonical catalog in code: `app/lib/ui/keyboard_shortcuts.dart`
(`kWordChordSpecs` → `kWordChordShortcuts` → `kWiredShortcuts`).

The same catalog drives the in-app **Keyboard shortcuts** Help dialog
(`app/lib/ui/keyboard_help_dialog.dart`): title-bar gear → Keyboard Shortcuts,
Help → Keyboard Shortcuts on macOS, About → Keyboard shortcuts, or **F1**.

Ctrl and Cmd are both registered for every application chord, so one binary
serves Windows, Linux and macOS. On macOS the `PlatformMenuBar` in
`editor_menu.dart` mirrors the common items and shows their key equivalents.
Verified on a macOS build: the Flutter `Shortcuts` map handles a chord before
AppKit's menu key equivalents, so Cmd+Z reaches `WordChord.undo` rather than the
Edit menu's Undo item. Both routes call the same controller method, so the menu
stays correct for clicks.

## File

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+N | New document |
| Ctrl/Cmd+O | Open document |
| Ctrl/Cmd+S | Save |
| Ctrl/Cmd+Shift+S | Save As |
| F12 | Save As (Word for Windows chord) |
| Ctrl/Cmd+P | Print |
| Ctrl/Cmd+F2 | Print preview |
| F1 | Keyboard shortcuts help |

## Undo and clipboard

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+Z | Undo |
| Ctrl/Cmd+Y | Redo |
| Ctrl/Cmd+Shift+Z | Redo |
| Ctrl/Cmd+X | Cut |
| Ctrl/Cmd+C | Copy |
| Ctrl/Cmd+V | Paste |
| Ctrl/Cmd+Shift+C | Copy formatting |
| Ctrl/Cmd+Shift+V | Paste formatting |
| Ctrl/Cmd+Alt+V | Paste Special |
| Ctrl/Cmd+Alt+Shift+V | Paste and Match Style |

Copy formatting and paste formatting drive the same Format Painter the Home tab
exposes: the first chord picks the format up from the caret or selection, the
second lays it onto the current selection or caret. Word's "keep text only" has
no chord of its own on Windows, so it stays on Alt+Shift+V, where Word for Mac
puts it.

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
| Ctrl/Cmd+] | Grow font |
| Ctrl/Cmd+[ | Shrink font |
| Ctrl/Cmd+= | Subscript |
| Ctrl/Cmd+Shift+= | Superscript |
| Ctrl/Cmd+Shift+A | All caps |
| Ctrl/Cmd+Shift+K | Small caps |
| Ctrl/Cmd+Shift+X | Strikethrough |
| Ctrl/Cmd+Shift+H | Hidden text |
| Shift+F3 | Change case (lower → Title → UPPER) |
| Ctrl/Cmd+Space | Clear character formatting |

Word grows the font in preset steps with Shift+> and by exactly one point with
the bracket chords. The engine exposes preset steps only, so both chords take
that path. Shift+F3 reads the next case off the selection instead of counting
presses, so the cycle behaves the same after clicking away and back.

## Paragraph formatting

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+L | Align left |
| Ctrl/Cmd+E | Center |
| Ctrl/Cmd+R | Align right |
| Ctrl/Cmd+J | Justify |
| Ctrl/Cmd+M | Increase indent |
| Ctrl/Cmd+Shift+M | Decrease indent |
| Alt+Shift+Left | Promote list item / outline level |
| Alt+Shift+Right | Demote list item / outline level |
| Alt+Shift+Up | Move paragraph up |
| Alt+Shift+Down | Move paragraph down |
| Ctrl/Cmd+Q | Remove paragraph formatting |
| Ctrl/Cmd+T | Create / grow hanging indent |
| Ctrl/Cmd+Shift+T | Shrink hanging indent |
| Ctrl/Cmd+0 | Toggle 12 pt space before paragraph |
| Ctrl/Cmd+Shift+L | Apply List Bullet style |
| Ctrl/Cmd+1 | Single line spacing |
| Ctrl/Cmd+2 | Double line spacing |
| Ctrl/Cmd+5 | 1.5 line spacing |

Alt+Shift+Up / Down reorders the caret's paragraph among its siblings, carrying
its formatting with it, and is one undo step. It stops at the first and last
paragraph of the section rather than crossing a section break, and reports that
in the status bar. Unlike the promote / demote pair it is claimed on every
platform, because Word for Mac uses the same chord.

Ctrl+Q drops direct paragraph formatting — alignment, indents, spacing and tab
stops — so the paragraph style shows through again. List numbering, shading and
borders are left alone; Word treats those as list and border commands. Ctrl+T
gives the paragraph a hanging indent by moving the left indent in one step while
the first line stays put, and Ctrl+Shift+T walks it back.

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
| Ctrl/Cmd+Alt+F | Insert footnote |
| Ctrl/Cmd+Alt+D | Insert endnote |
| Alt+Shift+D | Insert date field |
| Alt+Shift+P | Insert page number field |
| F7 | Spelling and grammar |

## Characters that are hard to type

| Chord | Action |
|-------|--------|
| Ctrl/Cmd+Shift+Space | Nonbreaking space (U+00A0) |
| Ctrl/Cmd+Shift+Hyphen | Nonbreaking hyphen (U+2011) |
| Ctrl/Cmd+Hyphen | Optional (soft) hyphen (U+00AD) |
| Ctrl/Cmd+Alt+C | Insert © |
| Ctrl/Cmd+Alt+R | Insert ® |
| Ctrl/Cmd+Alt+T | Insert ™ |
| Ctrl/Cmd+Alt+Period | Insert … |

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
| Option+Shift+Left, Option+Shift+Right | Extend the selection by word (macOS and iPadOS; elsewhere Alt+Shift+Left/Right re-levels the list) |
| Ctrl+Up, Ctrl+Down | Start of this paragraph, then of the previous / next one |
| Ctrl+Shift+Up, Ctrl+Shift+Down | Extend the selection to the paragraph edge |
| Option+Up, Option+Down | Previous / next paragraph (macOS and iPadOS) |
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
- **Ctrl+Up / Ctrl+Down move by paragraph**, matching Word: the first press goes
  to the start of the caret's own paragraph and the next one continues to the
  paragraph before it, while Down always lands on the next paragraph's start.
  Both clamp to the document edges. The engine reports the neighbouring
  paragraph starts (`tw_get_paragraph_nav`); an engine build without that query
  falls back to a plain line step. macOS and iPadOS also accept Option+Up /
  Option+Down, which is how those platforms spell paragraph motion.
- **Alt+Shift+Left / Right re-levels a list item from anywhere in it**, which is
  the only keyboard route once the caret has left the paragraph start. Word for
  Windows is free to use that chord because it moves by word with Ctrl+Arrow;
  macOS and iPadOS reserve Option+Shift+Arrow for word-wise selection at the OS
  level, so there the editor surface claims it and Tab / Shift+Tab remain the
  way to change level. The chord only affects list paragraphs — plain paragraphs
  and heading outline levels are unchanged.
- A page is the scroll unit in this viewport, so Page Up / Page Down move one
  page and preserve the caret's column.

## macOS and iOS

The same Dart bindings serve every platform, with these differences:

- **macOS keeps Ctrl+Left / Ctrl+Right for Mission Control**, so those never
  reach the app. Word's macOS convention, Option+Left / Option+Right, is wired
  for word motion and is the one to use there (verified on a macOS build).
- **Cmd+M still increases the indent.** The Window menu's system Minimize item
  advertises Cmd+M, but because Flutter sees the chord first the Word binding
  wins; Minimize remains available from the menu.
- **iOS and iPadOS route keys through the hidden text field.** The document
  canvas has no `Focus` on those platforms, so hardware keystrokes arrive at
  `WebGlyphTextInput`'s focus node: editing keys are handled there and Cmd chords
  are passed up to the `Shortcuts` map. An iPad Magic Keyboard has no Home/End,
  so Cmd+Left / Cmd+Right are the line-edge chords there, as in Word.

## Verifying on a device

`app/test/word_keyboard_mapping_test.dart` covers the mapping against the mock
engine. `app/integration_test/word_keyboard_mapping_test.dart` runs the same
chords inside a real binary — platform focus tree, platform text input and the
Rust engine over FFI — which is the only way to catch a chord that the hidden
UIKit text field swallows, or caret arithmetic that only the real engine gets
wrong:

```bash
cd app
flutter test integration_test/word_keyboard_mapping_test.dart -d <device-id>
```

The suite is debug-only: the pod is restricted to the Debug configuration and the
generated plugin registrant is gated behind `#if DEBUG`, so a Profile or Release
build contains no trace of it. See
[testing-strategy.md](testing-strategy.md#keeping-the-harness-out-of-shipping-builds)
for the mechanism and the check to run after touching it.

Note that the on-device suite creates a fresh document per test: the FFI engine
is process-global and outlives the widget tree, so text would otherwise
accumulate across tests.

## Ribbon keyboard navigation

The ribbon (`WordRibbon`) is wrapped in a `FocusTraversalGroup` with
`ReadingOrderTraversalPolicy`:

1. **Tab strip** — Home → Insert → … → View (Share opens the OS share sheet with the current document).
2. **Active tab body** — enabled controls left-to-right / top-to-bottom.
3. **Activate** — Space or Enter invokes the focused control (`ActivateIntent`).
4. **Disabled** controls (`onPressed == null`) do not participate in Tab order.

Focus ring uses the active-tab underline color. Mouse hover still works as before.

When the glyph editor has focus, Tab stays an editing key (see above). Click a
ribbon control or tab to move focus into the ribbon traversal group.
