# Keyboard Shortcuts & Ribbon Focus (F21.S2)

This document lists shortcuts that are **actually wired** in the Flutter UI today.
For the broader aspirational text-engine map (many of which are not yet bound), see
[architecture/text-engine.md](architecture/text-engine.md) § Keyboard Shortcuts.

Canonical catalog in code: `app/lib/ui/keyboard_shortcuts.dart` (`kWiredShortcuts`).

## Application shortcuts

Bound in `editor_screen.dart` (`Shortcuts` / `Actions`) and mirrored on the macOS
Edit menu where noted.

| Chord | Action | Binding |
|-------|--------|---------|
| Ctrl/Cmd+N | New document | macOS File menu |
| Ctrl/Cmd+O | Open document | macOS File menu |
| Ctrl/Cmd+S | Save | macOS File menu |
| Ctrl/Cmd+Z | Undo | macOS Edit menu |
| Ctrl/Cmd+Shift+Z | Redo | macOS Edit menu |
| Ctrl/Cmd+X | Cut | macOS Edit menu |
| Ctrl/Cmd+C | Copy | macOS Edit menu |
| Ctrl/Cmd+V | Paste | macOS Edit menu |
| Ctrl/Cmd+Alt+Shift+V | Paste and Match Style | macOS Edit menu |
| Ctrl/Cmd+A | Select all | App + Edit menu |
| Ctrl/Cmd+F | Find | App + Edit menu |
| Ctrl/Cmd+G | Go To | App + Edit menu |

Notes:

- Non-macOS builds rely on the `Shortcuts` map in `editor_screen.dart` for Find /
  Go To / Select All. File/Edit menu accelerators are macOS `PlatformMenuBar` only.
- Printing, Bold/Italic/Underline, Find-and-Replace, and other chords in
  `text-engine.md` are **not** wired yet.

## Editor (glyph surface)

While the document canvas has focus, Tab is claimed for editing (not UI focus):

| Chord | Action |
|-------|--------|
| Tab | Insert tab / demote list item / increase indent |
| Shift+Tab | Promote list item / decrease indent |
| Arrow keys | Move caret |
| Enter | Split paragraph |
| Backspace / Delete | Delete grapheme |

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
