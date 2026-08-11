import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/web_key_listener.dart' show WebKeyListener, createWebKeyListener;

/// Invisible text field overlay for Flutter web.
///
/// Browsers deliver printable keys through a focused editable DOM node. This
/// widget sits above the page canvas with [IgnorePointer] so clicks pass
/// through, while it keeps keyboard focus for [TextField.onChanged].
class WebGlyphTextInput extends StatefulWidget {
  const WebGlyphTextInput({super.key, required this.controller});

  final EditorController controller;

  @override
  State<WebGlyphTextInput> createState() => _WebGlyphTextInputState();
}

class _WebGlyphTextInputState extends State<WebGlyphTextInput> {
  late final TextEditingController _textController;
  late final WebKeyListener _domKeys;
  bool Function(KeyEvent event)? _keyboardHandler;
  String _lastFieldValue = '';

  @override
  void initState() {
    super.initState();
    _textController = TextEditingController();
    _domKeys = createWebKeyListener(widget.controller)..attach();
    _keyboardHandler = _onHardwareKey;
    HardwareKeyboard.instance.addHandler(_keyboardHandler!);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      widget.controller.ensureGlyphCaret();
      widget.controller.focusGlyphInput();
    });
  }

  @override
  void dispose() {
    _domKeys.dispose();
    if (_keyboardHandler != null) {
      HardwareKeyboard.instance.removeHandler(_keyboardHandler!);
    }
    _textController.dispose();
    super.dispose();
  }

  bool _onHardwareKey(KeyEvent event) {
    if (!widget.controller.webGlyphFocusNode.hasFocus) return false;
    return _handleSpecialKey(event);
  }

  bool _handleSpecialKey(KeyEvent event) {
    if (event is! KeyDownEvent) return false;
    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.backspace) {
      unawaited(widget.controller.deleteGlyphBackward());
      return true;
    }
    if (key == LogicalKeyboardKey.delete) {
      unawaited(widget.controller.deleteGlyphForward());
      return true;
    }
    if (key == LogicalKeyboardKey.enter || key == LogicalKeyboardKey.numpadEnter) {
      unawaited(widget.controller.insertGlyphParagraphBreak());
      return true;
    }
    if (key == LogicalKeyboardKey.tab) {
      if (widget.controller.isInList) {
        if (HardwareKeyboard.instance.isShiftPressed) {
          widget.controller.demoteListLevel();
        } else {
          widget.controller.promoteListLevel();
        }
      } else if (HardwareKeyboard.instance.isShiftPressed) {
        widget.controller.decreaseIndent();
      } else {
        unawaited(widget.controller.insertGlyphCharacter('\t'));
      }
      return true;
    }
    if (key == LogicalKeyboardKey.arrowLeft ||
        key == LogicalKeyboardKey.arrowRight ||
        key == LogicalKeyboardKey.arrowUp ||
        key == LogicalKeyboardKey.arrowDown) {
      widget.controller.moveGlyphCaretByArrow(key);
      return true;
    }
    return false;
  }

  void _onChanged(String value) {
    // Desktop browsers route printable keys through the DOM listener above.
    // onChanged remains for mobile soft-keyboard / IME composition input.
    if (value.length <= _lastFieldValue.length) {
      _lastFieldValue = value;
      return;
    }
    final added = value.substring(_lastFieldValue.length);
    _lastFieldValue = value;
    if (added.isEmpty) return;
    widget.controller.ensureGlyphCaret();
    for (var i = 0; i < added.length; i++) {
      final ch = added[i];
      if (ch == '\n' || ch == '\r') {
        unawaited(widget.controller.insertGlyphParagraphBreak());
      } else {
        unawaited(widget.controller.insertGlyphCharacter(ch));
      }
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      if (_textController.text.isNotEmpty) {
        _textController.clear();
        _lastFieldValue = '';
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Positioned.fill(
      child: IgnorePointer(
        child: Focus(
          onKeyEvent: (node, event) {
            if (_handleSpecialKey(event)) return KeyEventResult.handled;
            return KeyEventResult.ignored;
          },
          child: TextField(
            focusNode: widget.controller.webGlyphFocusNode,
            controller: _textController,
            autofocus: true,
            style: const TextStyle(fontSize: 16, height: 1, color: Colors.transparent),
            cursorColor: Colors.transparent,
            showCursor: false,
            enableInteractiveSelection: false,
            decoration: const InputDecoration(
              border: InputBorder.none,
              contentPadding: EdgeInsets.zero,
            ),
            onChanged: _onChanged,
          ),
        ),
      ),
    );
  }
}
