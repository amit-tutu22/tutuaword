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

    if (key == 'Backspace') {
      event.preventDefault();
      _controller.markWebSpecialKeyConsumed();
      unawaited(_controller.handleEditorInput(const EditorInputEvent.backspace()));
      return;
    }
    if (key == 'Delete') {
      event.preventDefault();
      _controller.markWebSpecialKeyConsumed();
      unawaited(_controller.handleEditorInput(const EditorInputEvent.delete()));
      return;
    }
    if (key == 'Enter') {
      event.preventDefault();
      _controller.markWebSpecialKeyConsumed();
      unawaited(_controller.handleEditorInput(const EditorInputEvent.newline()));
      return;
    }
    if (key == 'Tab') {
      event.preventDefault();
      _controller.markWebSpecialKeyConsumed();
      unawaited(
        _controller.handleEditorInput(EditorInputEvent.tab(shift: event.shiftKey)),
      );
      return;
    }
    if (key == 'ArrowLeft' ||
        key == 'ArrowRight' ||
        key == 'ArrowUp' ||
        key == 'ArrowDown') {
      event.preventDefault();
      _controller.markWebSpecialKeyConsumed();
      unawaited(_controller.handleEditorInput(_arrowEvent(key)));
      return;
    }

    if (event.ctrlKey || event.metaKey || event.altKey) return;
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

EditorInputEvent _arrowEvent(String key) {
  switch (key) {
    case 'ArrowLeft':
      return const EditorInputEvent.arrowLeft();
    case 'ArrowRight':
      return const EditorInputEvent.arrowRight();
    case 'ArrowUp':
      return const EditorInputEvent.arrowUp();
    case 'ArrowDown':
      return const EditorInputEvent.arrowDown();
    default:
      return const EditorInputEvent.arrowRight();
  }
}

WebKeyListener createWebKeyListener(EditorController controller) =>
    WebKeyListenerImpl(controller);
