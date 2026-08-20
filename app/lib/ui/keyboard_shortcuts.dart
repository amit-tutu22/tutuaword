import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

/// Canonical keyboard map (F21.S2) — Microsoft Word's chords, in one table.
///
/// Everything here is really wired: [kWordChordShortcuts] feeds the `Shortcuts`
/// map in `editor_screen.dart`, and editor-surface keys are dispatched by
/// `EditorInputEvent.fromLogicalKey`. Keep [docs/keyboard.md] in sync — a test
/// asserts every chord label below appears in that document.

/// A command reachable by chord. The name doubles as the catalog id.
enum WordChord {
  newDocument,
  openDocument,
  save,
  saveAs,
  print,
  undo,
  redo,
  cut,
  copy,
  paste,
  copyFormatting,
  pasteFormatting,
  pasteSpecial,
  pasteMatchStyle,
  selectAll,
  find,
  replace,
  findNext,
  goTo,
  formattingMarks,
  bold,
  italic,
  underline,
  growFont,
  shrinkFont,
  subscript,
  superscript,
  allCaps,
  smallCaps,
  strikethrough,
  hiddenText,
  changeCase,
  clearFormatting,
  alignLeft,
  alignCenter,
  alignRight,
  alignJustify,
  increaseIndent,
  decreaseIndent,
  promoteOutline,
  demoteOutline,
  moveParagraphUp,
  moveParagraphDown,
  clearParagraphFormatting,
  hangingIndent,
  removeHangingIndent,
  toggleSpaceBefore,
  bulletList,
  singleSpace,
  oneAndAHalfSpace,
  doubleSpace,
  normalStyle,
  heading1,
  heading2,
  heading3,
  hyperlink,
  comment,
  trackChanges,
  footnote,
  endnote,
  dateField,
  pageNumberField,
  nonbreakingSpace,
  nonbreakingHyphen,
  optionalHyphen,
  copyright,
  registered,
  trademark,
  ellipsis,
  spelling,
  printPreview,
  keyboardHelp,
}

/// Dispatched by the `Shortcuts` map; `editor_screen.dart` holds the one
/// [Action] that runs [chord].
class WordChordIntent extends Intent {
  const WordChordIntent(this.chord);

  final WordChord chord;
}

class WordChordSpec {
  const WordChordSpec({
    required this.chord,
    required this.label,
    required this.action,
    required this.key,
    this.shift = false,
    this.alt = false,
    this.bare = false,
  });

  final WordChord chord;

  /// Human-readable chord for the docs, e.g. `Ctrl/Cmd+B`.
  final String label;
  final String action;
  final LogicalKeyboardKey key;
  final bool shift;
  final bool alt;

  /// Function keys carry no Ctrl/Cmd modifier.
  final bool bare;

  /// Ctrl and Cmd are both registered: one binary serves Windows, Linux and
  /// macOS, and neither platform sees the other's modifier.
  List<ShortcutActivator> get activators {
    if (bare) {
      return [SingleActivator(key, shift: shift, alt: alt)];
    }
    return [
      SingleActivator(key, control: true, shift: shift, alt: alt),
      SingleActivator(key, meta: true, shift: shift, alt: alt),
    ];
  }
}

const kWordChordSpecs = <WordChordSpec>[
  // ── File ────────────────────────────────────────────────────────────────
  WordChordSpec(
    chord: WordChord.newDocument,
    label: 'Ctrl/Cmd+N',
    action: 'New document',
    key: LogicalKeyboardKey.keyN,
  ),
  WordChordSpec(
    chord: WordChord.openDocument,
    label: 'Ctrl/Cmd+O',
    action: 'Open document',
    key: LogicalKeyboardKey.keyO,
  ),
  WordChordSpec(
    chord: WordChord.save,
    label: 'Ctrl/Cmd+S',
    action: 'Save',
    key: LogicalKeyboardKey.keyS,
  ),
  WordChordSpec(
    chord: WordChord.saveAs,
    label: 'Ctrl/Cmd+Shift+S',
    action: 'Save As',
    key: LogicalKeyboardKey.keyS,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.saveAs,
    label: 'F12',
    action: 'Save As',
    key: LogicalKeyboardKey.f12,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.print,
    label: 'Ctrl/Cmd+P',
    action: 'Print',
    key: LogicalKeyboardKey.keyP,
  ),
  WordChordSpec(
    chord: WordChord.printPreview,
    label: 'Ctrl/Cmd+F2',
    action: 'Print preview',
    key: LogicalKeyboardKey.f2,
  ),
  WordChordSpec(
    chord: WordChord.keyboardHelp,
    label: 'F1',
    action: 'Keyboard shortcuts help',
    key: LogicalKeyboardKey.f1,
    bare: true,
  ),

  // ── Undo / clipboard ────────────────────────────────────────────────────
  WordChordSpec(
    chord: WordChord.undo,
    label: 'Ctrl/Cmd+Z',
    action: 'Undo',
    key: LogicalKeyboardKey.keyZ,
  ),
  WordChordSpec(
    chord: WordChord.redo,
    label: 'Ctrl/Cmd+Y',
    action: 'Redo',
    key: LogicalKeyboardKey.keyY,
  ),
  WordChordSpec(
    chord: WordChord.redo,
    label: 'Ctrl/Cmd+Shift+Z',
    action: 'Redo',
    key: LogicalKeyboardKey.keyZ,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.cut,
    label: 'Ctrl/Cmd+X',
    action: 'Cut',
    key: LogicalKeyboardKey.keyX,
  ),
  WordChordSpec(
    chord: WordChord.copy,
    label: 'Ctrl/Cmd+C',
    action: 'Copy',
    key: LogicalKeyboardKey.keyC,
  ),
  WordChordSpec(
    chord: WordChord.paste,
    label: 'Ctrl/Cmd+V',
    action: 'Paste',
    key: LogicalKeyboardKey.keyV,
  ),
  // Word's format-painter pair: Shift+C picks the format up, Shift+V lays it
  // down. Keep-text-only stays on Alt+Shift+V, where Word for Mac puts it.
  WordChordSpec(
    chord: WordChord.copyFormatting,
    label: 'Ctrl/Cmd+Shift+C',
    action: 'Copy formatting',
    key: LogicalKeyboardKey.keyC,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.pasteFormatting,
    label: 'Ctrl/Cmd+Shift+V',
    action: 'Paste formatting',
    key: LogicalKeyboardKey.keyV,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.pasteSpecial,
    label: 'Ctrl/Cmd+Alt+V',
    action: 'Paste Special',
    key: LogicalKeyboardKey.keyV,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.pasteMatchStyle,
    label: 'Ctrl/Cmd+Alt+Shift+V',
    action: 'Paste and Match Style',
    key: LogicalKeyboardKey.keyV,
    shift: true,
    alt: true,
  ),

  // ── Select, find, navigate ──────────────────────────────────────────────
  WordChordSpec(
    chord: WordChord.selectAll,
    label: 'Ctrl/Cmd+A',
    action: 'Select all',
    key: LogicalKeyboardKey.keyA,
  ),
  WordChordSpec(
    chord: WordChord.find,
    label: 'Ctrl/Cmd+F',
    action: 'Find',
    key: LogicalKeyboardKey.keyF,
  ),
  WordChordSpec(
    chord: WordChord.replace,
    label: 'Ctrl/Cmd+H',
    action: 'Find and Replace',
    key: LogicalKeyboardKey.keyH,
  ),
  WordChordSpec(
    chord: WordChord.findNext,
    label: 'Shift+F4',
    action: 'Find next',
    key: LogicalKeyboardKey.f4,
    shift: true,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.goTo,
    label: 'Ctrl/Cmd+G',
    action: 'Go To',
    key: LogicalKeyboardKey.keyG,
  ),
  WordChordSpec(
    chord: WordChord.goTo,
    label: 'F5',
    action: 'Go To',
    key: LogicalKeyboardKey.f5,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.formattingMarks,
    label: 'Ctrl/Cmd+Shift+8',
    action: 'Show/hide formatting marks',
    key: LogicalKeyboardKey.digit8,
    shift: true,
  ),

  // ── Character formatting ────────────────────────────────────────────────
  WordChordSpec(
    chord: WordChord.bold,
    label: 'Ctrl/Cmd+B',
    action: 'Bold',
    key: LogicalKeyboardKey.keyB,
  ),
  WordChordSpec(
    chord: WordChord.italic,
    label: 'Ctrl/Cmd+I',
    action: 'Italic',
    key: LogicalKeyboardKey.keyI,
  ),
  WordChordSpec(
    chord: WordChord.underline,
    label: 'Ctrl/Cmd+U',
    action: 'Underline',
    key: LogicalKeyboardKey.keyU,
  ),
  WordChordSpec(
    chord: WordChord.growFont,
    label: 'Ctrl/Cmd+Shift+>',
    action: 'Grow font',
    key: LogicalKeyboardKey.period,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.shrinkFont,
    label: 'Ctrl/Cmd+Shift+<',
    action: 'Shrink font',
    key: LogicalKeyboardKey.comma,
    shift: true,
  ),
  // Word grows in preset steps with Shift+> and by a single point with ]. The
  // engine only offers preset steps, so both chords share it.
  WordChordSpec(
    chord: WordChord.growFont,
    label: 'Ctrl/Cmd+]',
    action: 'Grow font',
    key: LogicalKeyboardKey.bracketRight,
  ),
  WordChordSpec(
    chord: WordChord.shrinkFont,
    label: 'Ctrl/Cmd+[',
    action: 'Shrink font',
    key: LogicalKeyboardKey.bracketLeft,
  ),
  WordChordSpec(
    chord: WordChord.subscript,
    label: 'Ctrl/Cmd+=',
    action: 'Subscript',
    key: LogicalKeyboardKey.equal,
  ),
  WordChordSpec(
    chord: WordChord.superscript,
    label: 'Ctrl/Cmd+Shift+=',
    action: 'Superscript',
    key: LogicalKeyboardKey.equal,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.allCaps,
    label: 'Ctrl/Cmd+Shift+A',
    action: 'All caps',
    key: LogicalKeyboardKey.keyA,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.smallCaps,
    label: 'Ctrl/Cmd+Shift+K',
    action: 'Small caps',
    key: LogicalKeyboardKey.keyK,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.strikethrough,
    label: 'Ctrl/Cmd+Shift+X',
    action: 'Strikethrough',
    key: LogicalKeyboardKey.keyX,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.hiddenText,
    label: 'Ctrl/Cmd+Shift+H',
    action: 'Hidden text',
    key: LogicalKeyboardKey.keyH,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.changeCase,
    label: 'Shift+F3',
    action: 'Change case (lower → Title → UPPER)',
    key: LogicalKeyboardKey.f3,
    shift: true,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.clearFormatting,
    label: 'Ctrl/Cmd+Space',
    action: 'Clear character formatting',
    key: LogicalKeyboardKey.space,
  ),

  // ── Paragraph formatting ────────────────────────────────────────────────
  WordChordSpec(
    chord: WordChord.alignLeft,
    label: 'Ctrl/Cmd+L',
    action: 'Align left',
    key: LogicalKeyboardKey.keyL,
  ),
  WordChordSpec(
    chord: WordChord.alignCenter,
    label: 'Ctrl/Cmd+E',
    action: 'Center',
    key: LogicalKeyboardKey.keyE,
  ),
  WordChordSpec(
    chord: WordChord.alignRight,
    label: 'Ctrl/Cmd+R',
    action: 'Align right',
    key: LogicalKeyboardKey.keyR,
  ),
  WordChordSpec(
    chord: WordChord.alignJustify,
    label: 'Ctrl/Cmd+J',
    action: 'Justify',
    key: LogicalKeyboardKey.keyJ,
  ),
  WordChordSpec(
    chord: WordChord.increaseIndent,
    label: 'Ctrl/Cmd+M',
    action: 'Increase indent',
    key: LogicalKeyboardKey.keyM,
  ),
  WordChordSpec(
    chord: WordChord.decreaseIndent,
    label: 'Ctrl/Cmd+Shift+M',
    action: 'Decrease indent',
    key: LogicalKeyboardKey.keyM,
    shift: true,
  ),
  // Word's outline chords. Tab and Shift+Tab only change the level at the very
  // start of a list paragraph, so these are the way to re-level from mid-text.
  // On macOS and iPadOS the editor surface claims Option+Shift+Arrow first, as
  // the platform convention for word-wise selection.
  WordChordSpec(
    chord: WordChord.promoteOutline,
    label: 'Alt+Shift+Left',
    action: 'Promote list item / outline level',
    key: LogicalKeyboardKey.arrowLeft,
    shift: true,
    alt: true,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.demoteOutline,
    label: 'Alt+Shift+Right',
    action: 'Demote list item / outline level',
    key: LogicalKeyboardKey.arrowRight,
    shift: true,
    alt: true,
    bare: true,
  ),
  // Alt+Shift+Up/Down is Word's move-paragraph chord on Windows and on Mac, so
  // unlike the promote/demote pair it is claimed on every platform.
  WordChordSpec(
    chord: WordChord.moveParagraphUp,
    label: 'Alt+Shift+Up',
    action: 'Move paragraph up',
    key: LogicalKeyboardKey.arrowUp,
    shift: true,
    alt: true,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.moveParagraphDown,
    label: 'Alt+Shift+Down',
    action: 'Move paragraph down',
    key: LogicalKeyboardKey.arrowDown,
    shift: true,
    alt: true,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.clearParagraphFormatting,
    label: 'Ctrl/Cmd+Q',
    action: 'Remove paragraph formatting',
    key: LogicalKeyboardKey.keyQ,
  ),
  WordChordSpec(
    chord: WordChord.hangingIndent,
    label: 'Ctrl/Cmd+T',
    action: 'Create / grow hanging indent',
    key: LogicalKeyboardKey.keyT,
  ),
  WordChordSpec(
    chord: WordChord.removeHangingIndent,
    label: 'Ctrl/Cmd+Shift+T',
    action: 'Shrink hanging indent',
    key: LogicalKeyboardKey.keyT,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.toggleSpaceBefore,
    label: 'Ctrl/Cmd+0',
    action: 'Toggle 12 pt space before paragraph',
    key: LogicalKeyboardKey.digit0,
  ),
  WordChordSpec(
    chord: WordChord.bulletList,
    label: 'Ctrl/Cmd+Shift+L',
    action: 'Apply List Bullet style',
    key: LogicalKeyboardKey.keyL,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.singleSpace,
    label: 'Ctrl/Cmd+1',
    action: 'Single line spacing',
    key: LogicalKeyboardKey.digit1,
  ),
  WordChordSpec(
    chord: WordChord.doubleSpace,
    label: 'Ctrl/Cmd+2',
    action: 'Double line spacing',
    key: LogicalKeyboardKey.digit2,
  ),
  WordChordSpec(
    chord: WordChord.oneAndAHalfSpace,
    label: 'Ctrl/Cmd+5',
    action: '1.5 line spacing',
    key: LogicalKeyboardKey.digit5,
  ),

  // ── Styles ──────────────────────────────────────────────────────────────
  WordChordSpec(
    chord: WordChord.normalStyle,
    label: 'Ctrl/Cmd+Shift+N',
    action: 'Apply Normal style',
    key: LogicalKeyboardKey.keyN,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.heading1,
    label: 'Ctrl/Cmd+Alt+1',
    action: 'Apply Heading 1',
    key: LogicalKeyboardKey.digit1,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.heading2,
    label: 'Ctrl/Cmd+Alt+2',
    action: 'Apply Heading 2',
    key: LogicalKeyboardKey.digit2,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.heading3,
    label: 'Ctrl/Cmd+Alt+3',
    action: 'Apply Heading 3',
    key: LogicalKeyboardKey.digit3,
    alt: true,
  ),

  // ── Insert & review ─────────────────────────────────────────────────────
  WordChordSpec(
    chord: WordChord.hyperlink,
    label: 'Ctrl/Cmd+K',
    action: 'Insert hyperlink',
    key: LogicalKeyboardKey.keyK,
  ),
  WordChordSpec(
    chord: WordChord.comment,
    label: 'Ctrl/Cmd+Alt+M',
    action: 'New comment',
    key: LogicalKeyboardKey.keyM,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.trackChanges,
    label: 'Ctrl/Cmd+Shift+E',
    action: 'Toggle track changes',
    key: LogicalKeyboardKey.keyE,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.spelling,
    label: 'F7',
    action: 'Spelling and grammar',
    key: LogicalKeyboardKey.f7,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.footnote,
    label: 'Ctrl/Cmd+Alt+F',
    action: 'Insert footnote',
    key: LogicalKeyboardKey.keyF,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.endnote,
    label: 'Ctrl/Cmd+Alt+D',
    action: 'Insert endnote',
    key: LogicalKeyboardKey.keyD,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.dateField,
    label: 'Alt+Shift+D',
    action: 'Insert date field',
    key: LogicalKeyboardKey.keyD,
    shift: true,
    alt: true,
    bare: true,
  ),
  WordChordSpec(
    chord: WordChord.pageNumberField,
    label: 'Alt+Shift+P',
    action: 'Insert page number field',
    key: LogicalKeyboardKey.keyP,
    shift: true,
    alt: true,
    bare: true,
  ),

  // ── Characters that are hard to type ────────────────────────────────────
  WordChordSpec(
    chord: WordChord.nonbreakingSpace,
    label: 'Ctrl/Cmd+Shift+Space',
    action: 'Nonbreaking space',
    key: LogicalKeyboardKey.space,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.optionalHyphen,
    label: 'Ctrl/Cmd+Hyphen',
    action: 'Optional (soft) hyphen',
    key: LogicalKeyboardKey.minus,
  ),
  WordChordSpec(
    chord: WordChord.nonbreakingHyphen,
    label: 'Ctrl/Cmd+Shift+Hyphen',
    action: 'Nonbreaking hyphen',
    key: LogicalKeyboardKey.minus,
    shift: true,
  ),
  WordChordSpec(
    chord: WordChord.copyright,
    label: 'Ctrl/Cmd+Alt+C',
    action: 'Insert ©',
    key: LogicalKeyboardKey.keyC,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.registered,
    label: 'Ctrl/Cmd+Alt+R',
    action: 'Insert ®',
    key: LogicalKeyboardKey.keyR,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.trademark,
    label: 'Ctrl/Cmd+Alt+T',
    action: 'Insert ™',
    key: LogicalKeyboardKey.keyT,
    alt: true,
  ),
  WordChordSpec(
    chord: WordChord.ellipsis,
    label: 'Ctrl/Cmd+Alt+Period',
    action: 'Insert …',
    key: LogicalKeyboardKey.period,
    alt: true,
  ),
];

/// The `Shortcuts` map wired in `editor_screen.dart`.
final Map<ShortcutActivator, Intent> kWordChordShortcuts = {
  for (final spec in kWordChordSpecs)
    for (final activator in spec.activators)
      activator: WordChordIntent(spec.chord),
};

/// Keys the document canvas claims directly (no `Shortcuts` entry) — dispatched
/// by `EditorInputEvent.fromLogicalKey`.
const kEditorKeyActions = <String, String>{
  'Enter': 'Split paragraph',
  'Shift+Enter': 'Manual line break inside the paragraph',
  'Ctrl/Cmd+Enter': 'Page break',
  'Tab': 'Insert tab, or demote the list item at paragraph start',
  'Shift+Tab': 'Promote list item / decrease indent',
  'Backspace': 'Delete grapheme before the caret',
  'Delete': 'Delete grapheme after the caret',
  'Ctrl/Alt+Backspace': 'Delete word before the caret',
  'Ctrl/Alt+Delete': 'Delete word after the caret',
  'Arrow keys': 'Move caret by character / line',
  'Shift+Arrows': 'Extend the selection',
  'Ctrl/Alt+Left, Ctrl/Alt+Right': 'Move caret by word',
  'Option+Shift+Left, Option+Shift+Right':
      'Extend the selection by word (macOS and iPadOS; elsewhere Alt+Shift+Left/Right re-levels the list)',
  'Ctrl+Up, Ctrl+Down': 'Start of this paragraph, then of the previous / next one',
  'Ctrl+Shift+Up, Ctrl+Shift+Down': 'Extend the selection to the paragraph edge',
  'Option+Up, Option+Down': 'Previous / next paragraph (macOS and iPadOS)',
  'Home, End': 'Start / end of line',
  'Ctrl+Home, Ctrl+End': 'Start / end of document',
  'Cmd+Left, Cmd+Right': 'Start / end of line (macOS)',
  'Cmd+Up, Cmd+Down': 'Start / end of document (macOS)',
  'Page Up, Page Down': 'Previous / next screen',
};

/// Catalog of shortcuts that are actually wired in the Flutter UI (F21.S2).
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

  /// Where the binding lives: `app` or `editor`.
  final String scope;
}

/// Flat catalog, derived so it cannot drift from the bindings above.
final kWiredShortcuts = <WiredShortcut>[
  for (final spec in kWordChordSpecs)
    WiredShortcut(
      id: '${spec.chord.name}:${spec.label}',
      chord: spec.label,
      action: spec.action,
      scope: 'app',
    ),
  for (final entry in kEditorKeyActions.entries)
    WiredShortcut(
      id: 'editor:${entry.key}',
      chord: entry.key,
      action: entry.value,
      scope: 'editor',
    ),
];
