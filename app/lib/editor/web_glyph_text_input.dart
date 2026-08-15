import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_input.dart';
import 'package:tutuaword/editor/key_event_text.dart';
import 'package:tutuaword/editor/web_key_listener.dart' show WebKeyListener, createWebKeyListener;

/// Invisible text field overlay for Flutter web / mobile soft keyboard.
///
/// Kept as a 1×1 focus target so the platform editable node cannot cover the
/// document canvas. On Flutter web a full-screen [TextField] steals pointer
/// hits at the DOM layer even when wrapped in [IgnorePointer].
///
/// Special keys (Enter, Tab, arrows, Backspace) are owned by [WebKeyListener]
/// on web (DOM capture). On iOS/Android they are handled via the focus node's
/// [FocusNode.onKeyEvent] so selected images/shapes can still be deleted.
class WebGlyphTextInput extends StatefulWidget {
  const WebGlyphTextInput({super.key, required this.controller});

  final EditorController controller;

  @override
  State<WebGlyphTextInput> createState() => _WebGlyphTextInputState();
}

class _WebGlyphTextInputState extends State<WebGlyphTextInput> {
  late final TextEditingController _textController;
  late final WebKeyListener _domKeys;
  String _lastFieldValue = '';

  @override
  void initState() {
    super.initState();
    _textController = TextEditingController();
    _domKeys = createWebKeyListener(widget.controller)..attach();
    // Mobile / desktop-soft-keyboard: special keys never reach GlyphEditorSurface
    // Focus (that Focus is omitted when usesSoftKeyboardGlyphInput is true).
    if (!kIsWeb) {
      widget.controller.webGlyphFocusNode.onKeyEvent = _handleFocusKey;
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      widget.controller.ensureGlyphCaret();
      // Request focus directly (not via focusGlyphInput): widget tests mount
      // this overlay on desktop where usesSoftKeyboardGlyphInput is false.
      final node = widget.controller.webGlyphFocusNode;
      if (node.canRequestFocus && !node.hasFocus) {
        node.requestFocus();
      }
    });
  }

  @override
  void dispose() {
    if (!kIsWeb) {
      widget.controller.webGlyphFocusNode.onKeyEvent = null;
    }
    _domKeys.dispose();
    _textController.dispose();
    super.dispose();
  }

  KeyEventResult _handleFocusKey(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.ignored;
    final key = event.logicalKey;
    String? char = printableCharacterFromKeyEvent(event);
    if (char == null &&
        (key == LogicalKeyboardKey.space || key.keyLabel.toLowerCase() == 'space')) {
      char = ' ';
    }
    final input = EditorInputEvent.fromLogicalKey(
      key,
      shift: HardwareKeyboard.instance.isShiftPressed,
      character: char,
    );
    if (input == null) return KeyEventResult.ignored;
    // Printable characters still flow through TextField.onChanged / IME.
    if (input.kind == EditorInputKind.character) {
      return KeyEventResult.ignored;
    }
    unawaited(widget.controller.handleEditorInput(input));
    return KeyEventResult.handled;
  }

  void _clearField() {
    if (_textController.text.isEmpty) {
      _lastFieldValue = '';
      return;
    }
    _textController.clear();
    _lastFieldValue = '';
  }

  Future<void> _applyAddedText(String added) async {
    if (added.isEmpty) return;
    widget.controller.ensureGlyphCaret();
    var sawNewline = false;
    for (var i = 0; i < added.length; i++) {
      final ch = added[i];
      if (ch == '\n' || ch == '\r') {
        sawNewline = true;
        await widget.controller.handleEditorInput(const EditorInputEvent.newline());
      } else {
        await widget.controller.handleEditorInput(EditorInputEvent.character(ch));
      }
    }
    if (sawNewline && mounted) {
      _clearField();
    }
  }

  void _onChanged(String value) {
    if (widget.controller.consumeWebSkipNextFieldChange()) {
      _lastFieldValue = value;
      if (value.contains('\n') || value.contains('\r')) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted) _clearField();
        });
      }
      return;
    }
    // Desktop browsers route printable keys through the DOM listener above.
    // onChanged remains for mobile soft-keyboard / IME composition input.
    if (value.length < _lastFieldValue.length) {
      // Soft-keyboard Backspace: delete selected object or prior character.
      final removed = _lastFieldValue.length - value.length;
      _lastFieldValue = value;
      unawaited(() async {
        for (var i = 0; i < removed; i++) {
          await widget.controller.handleEditorInput(const EditorInputEvent.backspace());
        }
      }());
      return;
    }
    if (value.length == _lastFieldValue.length) {
      _lastFieldValue = value;
      return;
    }
    final added = value.substring(_lastFieldValue.length);
    _lastFieldValue = value;
    unawaited(_applyAddedText(added));
  }

  void _onSubmitted(String value) {
    if (widget.controller.consumeWebSkipNextFieldChange()) {
      _clearField();
      return;
    }
    unawaited(() async {
      await widget.controller.handleEditorInput(const EditorInputEvent.newline());
      if (mounted) _clearField();
    }());
  }

  @override
  Widget build(BuildContext context) {
    // Never Positioned.fill — on web the DOM <input> would cover the canvas
    // and block image/shape selection even under IgnorePointer.
    // Keep a tiny real layout box (not Opacity(0)): iOS UIKit logs
    // TUIKeyplane/UIKeyboardImpl constraint conflicts when the keyboard
    // attaches to a zero-alpha / zero-size host during keyplane changes.
    return Positioned(
      left: 0,
      top: 0,
      width: 2,
      height: 2,
      child: TextField(
        focusNode: widget.controller.webGlyphFocusNode,
        controller: _textController,
        // Focus is requested explicitly when the canvas is tapped — autofocus
        // races with that and can re-attach the iOS keyboard mid-animation.
        autofocus: false,
        keyboardType: TextInputType.multiline,
        textInputAction: TextInputAction.newline,
        // maxLines > 1 keeps Return as a newline keyplane on iOS instead of
        // toggling between 216pt / 250pt temporary layouts.
        minLines: 1,
        maxLines: 3,
        style: const TextStyle(fontSize: 16, height: 1, color: Colors.transparent),
        cursorColor: Colors.transparent,
        showCursor: false,
        enableInteractiveSelection: false,
        decoration: const InputDecoration(
          border: InputBorder.none,
          contentPadding: EdgeInsets.zero,
          isCollapsed: true,
        ),
        onChanged: _onChanged,
        onSubmitted: _onSubmitted,
      ),
    );
  }
}
