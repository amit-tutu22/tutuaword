/// Catalog of shortcuts that are actually wired in the Flutter UI (F21.S2).
///
/// Keep in sync with [docs/keyboard.md] and the `Shortcuts` / menu bindings
/// in `editor_screen.dart` / `editor_menu.dart` / `glyph_editor_surface.dart`.
class WiredShortcut {
  const WiredShortcut({
    required this.id,
    required this.chord,
    required this.action,
    required this.scope,
  });

  final String id;
  /// Human-readable chord, e.g. `Ctrl/Cmd+F`.
  final String chord;
  final String action;
  /// Where the binding lives: `app`, `menu`, or `editor`.
  final String scope;
}

/// Shortcuts implemented today (not the aspirational text-engine table).
const kWiredShortcuts = <WiredShortcut>[
  WiredShortcut(
    id: 'new',
    chord: 'Ctrl/Cmd+N',
    action: 'New document',
    scope: 'menu',
  ),
  WiredShortcut(
    id: 'open',
    chord: 'Ctrl/Cmd+O',
    action: 'Open document',
    scope: 'menu',
  ),
  WiredShortcut(
    id: 'save',
    chord: 'Ctrl/Cmd+S',
    action: 'Save',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'undo',
    chord: 'Ctrl/Cmd+Z',
    action: 'Undo',
    scope: 'menu',
  ),
  WiredShortcut(
    id: 'redo',
    chord: 'Ctrl/Cmd+Shift+Z',
    action: 'Redo',
    scope: 'menu',
  ),
  WiredShortcut(
    id: 'cut',
    chord: 'Ctrl/Cmd+X',
    action: 'Cut',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'copy',
    chord: 'Ctrl/Cmd+C',
    action: 'Copy',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'paste',
    chord: 'Ctrl/Cmd+V',
    action: 'Paste',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'paste_match',
    chord: 'Ctrl/Cmd+Alt+Shift+V',
    action: 'Paste and Match Style',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'select_all',
    chord: 'Ctrl/Cmd+A',
    action: 'Select all',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'find',
    chord: 'Ctrl/Cmd+F',
    action: 'Find',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'goto',
    chord: 'Ctrl/Cmd+G',
    action: 'Go To',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'print',
    chord: 'Ctrl/Cmd+P',
    action: 'Print',
    scope: 'app',
  ),
  WiredShortcut(
    id: 'tab_insert',
    chord: 'Tab',
    action: 'Insert tab / demote list / indent',
    scope: 'editor',
  ),
  WiredShortcut(
    id: 'tab_outdent',
    chord: 'Shift+Tab',
    action: 'Promote list / outdent',
    scope: 'editor',
  ),
];
