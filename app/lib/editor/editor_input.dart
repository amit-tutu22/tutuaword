import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Semantic keyboard input kinds routed through [EditorController.handleEditorInput].
enum EditorInputKind {
  character,
  newline,
  /// Shift+Enter — a manual line break inside the paragraph (`<w:br/>`).
  lineBreak,
  /// Ctrl+Enter — a hard page break.
  pageBreak,
  tab,
  backspace,
  delete,
  deleteWordBackward,
  deleteWordForward,
  arrowLeft,
  arrowRight,
  arrowUp,
  arrowDown,
  wordLeft,
  wordRight,
  /// Ctrl+Up / Ctrl+Down — the start of this paragraph, then of the one before.
  paragraphUp,
  paragraphDown,
  lineStart,
  lineEnd,
  documentStart,
  documentEnd,
  pageUp,
  pageDown,
}

/// Kinds that only move the caret, so Shift means "extend the selection".
const _navigationKinds = <EditorInputKind>{
  EditorInputKind.arrowLeft,
  EditorInputKind.arrowRight,
  EditorInputKind.arrowUp,
  EditorInputKind.arrowDown,
  EditorInputKind.wordLeft,
  EditorInputKind.wordRight,
  EditorInputKind.paragraphUp,
  EditorInputKind.paragraphDown,
  EditorInputKind.lineStart,
  EditorInputKind.lineEnd,
  EditorInputKind.documentStart,
  EditorInputKind.documentEnd,
  EditorInputKind.pageUp,
  EditorInputKind.pageDown,
};

bool get _isApple =>
    defaultTargetPlatform == TargetPlatform.macOS ||
    defaultTargetPlatform == TargetPlatform.iOS;

/// One logical editor keystroke — the only shape accepted by the input dispatcher.
class EditorInputEvent {
  const EditorInputEvent._(this.kind, {this.character, this.shift = false});

  final EditorInputKind kind;
  final String? character;
  final bool shift;

  /// True when this keystroke should grow the selection instead of collapsing it.
  bool get extendsSelection => shift && _navigationKinds.contains(kind);

  factory EditorInputEvent.character(String character) =>
      EditorInputEvent._(EditorInputKind.character, character: character);

  const EditorInputEvent.newline() : this._(EditorInputKind.newline);

  const EditorInputEvent.lineBreak() : this._(EditorInputKind.lineBreak);

  const EditorInputEvent.pageBreak() : this._(EditorInputKind.pageBreak);

  const EditorInputEvent.tab({bool shift = false})
      : this._(EditorInputKind.tab, shift: shift);

  const EditorInputEvent.backspace() : this._(EditorInputKind.backspace);

  const EditorInputEvent.delete() : this._(EditorInputKind.delete);

  const EditorInputEvent.deleteWordBackward()
      : this._(EditorInputKind.deleteWordBackward);

  const EditorInputEvent.deleteWordForward()
      : this._(EditorInputKind.deleteWordForward);

  const EditorInputEvent.arrowLeft({bool shift = false})
      : this._(EditorInputKind.arrowLeft, shift: shift);

  const EditorInputEvent.arrowRight({bool shift = false})
      : this._(EditorInputKind.arrowRight, shift: shift);

  const EditorInputEvent.arrowUp({bool shift = false})
      : this._(EditorInputKind.arrowUp, shift: shift);

  const EditorInputEvent.arrowDown({bool shift = false})
      : this._(EditorInputKind.arrowDown, shift: shift);

  const EditorInputEvent.wordLeft({bool shift = false})
      : this._(EditorInputKind.wordLeft, shift: shift);

  const EditorInputEvent.wordRight({bool shift = false})
      : this._(EditorInputKind.wordRight, shift: shift);

  const EditorInputEvent.paragraphUp({bool shift = false})
      : this._(EditorInputKind.paragraphUp, shift: shift);

  const EditorInputEvent.paragraphDown({bool shift = false})
      : this._(EditorInputKind.paragraphDown, shift: shift);

  const EditorInputEvent.lineStart({bool shift = false})
      : this._(EditorInputKind.lineStart, shift: shift);

  const EditorInputEvent.lineEnd({bool shift = false})
      : this._(EditorInputKind.lineEnd, shift: shift);

  const EditorInputEvent.documentStart({bool shift = false})
      : this._(EditorInputKind.documentStart, shift: shift);

  const EditorInputEvent.documentEnd({bool shift = false})
      : this._(EditorInputKind.documentEnd, shift: shift);

  const EditorInputEvent.pageUp({bool shift = false})
      : this._(EditorInputKind.pageUp, shift: shift);

  const EditorInputEvent.pageDown({bool shift = false})
      : this._(EditorInputKind.pageDown, shift: shift);

  /// Translate a physical chord into an editing intent using Word's mapping.
  ///
  /// Word supports two conventions for word- and document-wise motion and this
  /// accepts both: the Windows one (Ctrl+Arrow, Ctrl+Home/End) and the macOS one
  /// (Option+Arrow for words, Cmd+Arrow for line and document edges).
  static EditorInputEvent? fromLogicalKey(
    LogicalKeyboardKey key, {
    bool shift = false,
    bool control = false,
    bool alt = false,
    bool meta = false,
    String? character,
  }) {
    final byWord = control || alt;
    // Word for Windows spends Alt+Shift+Left/Right on promoting and demoting the
    // list item, and uses Ctrl+Arrow for word motion. Apple platforms keep
    // Option+Shift+Left/Right as word-wise selection, which the OS does
    // everywhere. Alt+Shift+Up/Down moves the paragraph in both Word editions,
    // so it is a command chord on every platform.
    final outlineChord = alt &&
        shift &&
        (key == LogicalKeyboardKey.arrowUp ||
            key == LogicalKeyboardKey.arrowDown ||
            (!_isApple &&
                (key == LogicalKeyboardKey.arrowLeft ||
                    key == LogicalKeyboardKey.arrowRight)));
    if (outlineChord) return null;

    if (key == LogicalKeyboardKey.backspace) {
      return byWord
          ? const EditorInputEvent.deleteWordBackward()
          : const EditorInputEvent.backspace();
    }
    if (key == LogicalKeyboardKey.delete) {
      return byWord
          ? const EditorInputEvent.deleteWordForward()
          : const EditorInputEvent.delete();
    }
    if (key == LogicalKeyboardKey.enter || key == LogicalKeyboardKey.numpadEnter) {
      if (control || meta) return const EditorInputEvent.pageBreak();
      if (shift) return const EditorInputEvent.lineBreak();
      return const EditorInputEvent.newline();
    }
    if (key == LogicalKeyboardKey.tab) {
      return EditorInputEvent.tab(shift: shift);
    }
    if (key == LogicalKeyboardKey.home) {
      return control
          ? EditorInputEvent.documentStart(shift: shift)
          : EditorInputEvent.lineStart(shift: shift);
    }
    if (key == LogicalKeyboardKey.end) {
      return control
          ? EditorInputEvent.documentEnd(shift: shift)
          : EditorInputEvent.lineEnd(shift: shift);
    }
    if (key == LogicalKeyboardKey.pageUp) {
      return EditorInputEvent.pageUp(shift: shift);
    }
    if (key == LogicalKeyboardKey.pageDown) {
      return EditorInputEvent.pageDown(shift: shift);
    }
    if (key == LogicalKeyboardKey.arrowLeft) {
      if (meta) return EditorInputEvent.lineStart(shift: shift);
      if (byWord) return EditorInputEvent.wordLeft(shift: shift);
      return EditorInputEvent.arrowLeft(shift: shift);
    }
    if (key == LogicalKeyboardKey.arrowRight) {
      if (meta) return EditorInputEvent.lineEnd(shift: shift);
      if (byWord) return EditorInputEvent.wordRight(shift: shift);
      return EditorInputEvent.arrowRight(shift: shift);
    }
    // Ctrl+Up/Down is Word's paragraph motion; Option+Up/Down is how macOS
    // spells the same move, so Apple platforms accept both.
    final byParagraph = control || (alt && _isApple);
    if (key == LogicalKeyboardKey.arrowUp) {
      if (meta) return EditorInputEvent.documentStart(shift: shift);
      if (byParagraph) return EditorInputEvent.paragraphUp(shift: shift);
      return EditorInputEvent.arrowUp(shift: shift);
    }
    if (key == LogicalKeyboardKey.arrowDown) {
      if (meta) return EditorInputEvent.documentEnd(shift: shift);
      if (byParagraph) return EditorInputEvent.paragraphDown(shift: shift);
      return EditorInputEvent.arrowDown(shift: shift);
    }
    if (character == '\n' || character == '\r') {
      return shift
          ? const EditorInputEvent.lineBreak()
          : const EditorInputEvent.newline();
    }
    if (character == '\t') {
      return EditorInputEvent.tab(shift: shift);
    }
    // A Ctrl/Cmd chord is a command, not text. Returning null leaves it for the
    // application `Shortcuts` map — otherwise Ctrl+B would type a "b".
    if (control || meta) return null;
    if (character != null && character.isNotEmpty) {
      return EditorInputEvent.character(character);
    }
    return null;
  }

  /// Same mapping, reading the live modifier state from [HardwareKeyboard].
  static EditorInputEvent? fromKeyEvent(KeyEvent event, {String? character}) {
    final keyboard = HardwareKeyboard.instance;
    return fromLogicalKey(
      event.logicalKey,
      shift: keyboard.isShiftPressed,
      control: keyboard.isControlPressed,
      alt: keyboard.isAltPressed,
      meta: keyboard.isMetaPressed,
      character: character,
    );
  }
}
