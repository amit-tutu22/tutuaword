import 'package:flutter/services.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Semantic keyboard input kinds routed through [EditorController.handleEditorInput].
enum EditorInputKind {
  character,
  newline,
  tab,
  backspace,
  delete,
  arrowLeft,
  arrowRight,
  arrowUp,
  arrowDown,
}

/// One logical editor keystroke — the only shape accepted by the input dispatcher.
class EditorInputEvent {
  const EditorInputEvent._(this.kind, {this.character, this.shift = false});

  final EditorInputKind kind;
  final String? character;
  final bool shift;

  factory EditorInputEvent.character(String character) =>
      EditorInputEvent._(EditorInputKind.character, character: character);

  const EditorInputEvent.newline() : this._(EditorInputKind.newline);

  const EditorInputEvent.tab({bool shift = false})
      : this._(EditorInputKind.tab, shift: shift);

  const EditorInputEvent.backspace() : this._(EditorInputKind.backspace);

  const EditorInputEvent.delete() : this._(EditorInputKind.delete);

  const EditorInputEvent.arrowLeft() : this._(EditorInputKind.arrowLeft);

  const EditorInputEvent.arrowRight() : this._(EditorInputKind.arrowRight);

  const EditorInputEvent.arrowUp() : this._(EditorInputKind.arrowUp);

  const EditorInputEvent.arrowDown() : this._(EditorInputKind.arrowDown);

  static EditorInputEvent? fromLogicalKey(
    LogicalKeyboardKey key, {
    bool shift = false,
    String? character,
  }) {
    if (key == LogicalKeyboardKey.backspace) {
      return const EditorInputEvent.backspace();
    }
    if (key == LogicalKeyboardKey.delete) {
      return const EditorInputEvent.delete();
    }
    if (key == LogicalKeyboardKey.enter || key == LogicalKeyboardKey.numpadEnter) {
      return const EditorInputEvent.newline();
    }
    if (key == LogicalKeyboardKey.tab) {
      return EditorInputEvent.tab(shift: shift);
    }
    if (key == LogicalKeyboardKey.arrowLeft) {
      return const EditorInputEvent.arrowLeft();
    }
    if (key == LogicalKeyboardKey.arrowRight) {
      return const EditorInputEvent.arrowRight();
    }
    if (key == LogicalKeyboardKey.arrowUp) {
      return const EditorInputEvent.arrowUp();
    }
    if (key == LogicalKeyboardKey.arrowDown) {
      return const EditorInputEvent.arrowDown();
    }
    if (character == '\n' || character == '\r') {
      return const EditorInputEvent.newline();
    }
    if (character == '\t') {
      return EditorInputEvent.tab(shift: shift);
    }
    if (character != null && character.isNotEmpty) {
      return EditorInputEvent.character(character);
    }
    return null;
  }
}
