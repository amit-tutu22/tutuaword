import 'dart:async';
import 'dart:html' as html;

import 'package:flutter/services.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_input.dart';

abstract class WebKeyListener {
  void attach();
  void dispose();
}

class WebKeyListenerImpl implements WebKeyListener {
  WebKeyListenerImpl(this._controller);

  final EditorController _controller;
  void Function(html.Event)? _handler;

  @override
  void attach() {
    _handler = (html.Event event) {
      if (event is html.KeyboardEvent) _onKeyDown(event);
    };
    html.window.addEventListener('keydown', _handler!, true);
  }

  @override
  void dispose() {
    final handler = _handler;
    if (handler != null) {
      html.window.removeEventListener('keydown', handler, true);
      _handler = null;
    }
  }

  void _onKeyDown(html.KeyboardEvent event) {
    if (!_shouldCapture(event)) return;

    final key = event.key;
    if (key == null || key.isEmpty) return;

    // Editing keys are ours on every platform. Chords we do not own (Ctrl+B,
    // Ctrl+Z, …) fall through untouched so Flutter's Shortcuts can claim them.
    final editingKey = _editingKeys[key];
    if (editingKey != null) {
      final input = EditorInputEvent.fromLogicalKey(
        editingKey,
        shift: event.shiftKey == true,
        control: event.ctrlKey == true,
        alt: event.altKey == true,
        meta: event.metaKey == true,
      );
      if (input == null) return;
      event.preventDefault();
      _controller.markWebSpecialKeyConsumed();
      unawaited(_controller.handleEditorInput(input));
      return;
    }

    if (event.ctrlKey == true || event.metaKey == true || event.altKey == true) {
      return;
    }
    if (key.length != 1) return;

    event.preventDefault();
    _controller.ensureGlyphCaret();
    unawaited(_controller.handleEditorInput(EditorInputEvent.character(key)));
  }

  bool _shouldCapture(html.KeyboardEvent event) {
    if (_controller.findPaneVisible) {
      final active = html.document.activeElement;
      if (active != null && active.tagName.toLowerCase() == 'input') {
        return false;
      }
    }
    return _controller.webGlyphFocusNode.hasFocus ||
        !_isForeignInput(html.document.activeElement);
  }

  bool _isForeignInput(html.Element? element) {
    if (element == null) return false;
    final tag = element.tagName.toLowerCase();
    if (tag != 'input' && tag != 'textarea') return false;
    // Flutter's hidden editor input uses flt-text-editing or contenteditable.
    final classes = element.classes.join(' ');
    if (classes.contains('flt-text-editing') ||
        element.getAttribute('data-semantics-role') == 'text-field') {
      return false;
    }
    return true;
  }
}

/// DOM `KeyboardEvent.key` names the glyph editor owns, mapped onto the same
/// logical keys the desktop path uses so both share one chord table.
const _editingKeys = <String, LogicalKeyboardKey>{
  'Backspace': LogicalKeyboardKey.backspace,
  'Delete': LogicalKeyboardKey.delete,
  'Enter': LogicalKeyboardKey.enter,
  'Tab': LogicalKeyboardKey.tab,
  'ArrowLeft': LogicalKeyboardKey.arrowLeft,
  'ArrowRight': LogicalKeyboardKey.arrowRight,
  'ArrowUp': LogicalKeyboardKey.arrowUp,
  'ArrowDown': LogicalKeyboardKey.arrowDown,
  'Home': LogicalKeyboardKey.home,
  'End': LogicalKeyboardKey.end,
  'PageUp': LogicalKeyboardKey.pageUp,
  'PageDown': LogicalKeyboardKey.pageDown,
};

WebKeyListener createWebKeyListener(EditorController controller) =>
    WebKeyListenerImpl(controller);
