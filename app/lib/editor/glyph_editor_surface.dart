import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/native_engine.dart';
import 'package:tutuaword/editor/document_painter.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';

class GlyphEditorSurface extends StatefulWidget {
  const GlyphEditorSurface({
    super.key,
    required this.controller,
    required this.pageIndex,
    required this.snapshot,
    required this.atlasImage,
    this.images = const {},
  });

  final EditorController controller;
  final int pageIndex;
  final DisplayListSnapshot snapshot;
  final ui.Image? atlasImage;
  final Map<String, ui.Image> images;

  @override
  State<GlyphEditorSurface> createState() => _GlyphEditorSurfaceState();
}

class _GlyphEditorSurfaceState extends State<GlyphEditorSurface> {
  late final FocusNode _focusNode;
  Offset? _pointerDown;
  bool _selecting = false;

  @override
  void initState() {
    super.initState();
    _focusNode = FocusNode();
    widget.controller.addListener(_onControllerUpdate);
    if (widget.pageIndex == 0) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        widget.controller.ensureGlyphCaret();
        _focusNode.requestFocus();
      });
    }
  }

  @override
  void dispose() {
    widget.controller.removeListener(_onControllerUpdate);
    _focusNode.dispose();
    super.dispose();
  }

  void _onControllerUpdate() => setState(() {});

  KeyEventResult _handleKey(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.ignored;
    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.backspace) {
      widget.controller.deleteGlyphBackward();
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.enter || key == LogicalKeyboardKey.numpadEnter) {
      widget.controller.insertGlyphParagraphBreak();
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.arrowLeft ||
        key == LogicalKeyboardKey.arrowRight ||
        key == LogicalKeyboardKey.arrowUp ||
        key == LogicalKeyboardKey.arrowDown) {
      widget.controller.moveGlyphCaretByArrow(key);
      return KeyEventResult.handled;
    }
    // On macOS, `event.character` can be null/empty for some whitespace keys
    // (notably Space). Handle them explicitly so the caret advances.
    String? char = event.character;
    if (key == LogicalKeyboardKey.space ||
        key.keyLabel.toLowerCase() == 'space') {
      char = ' ';
    }
    // Never treat Enter / Return as a printable character (avoids □ tofu).
    if (char == '\n' || char == '\r') {
      widget.controller.insertGlyphParagraphBreak();
      return KeyEventResult.handled;
    }
    if (char != null &&
        char.isNotEmpty &&
        !HardwareKeyboard.instance.isControlPressed &&
        !HardwareKeyboard.instance.isMetaPressed) {
      widget.controller.insertGlyphCharacter(char);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _onPointerDown(PointerDownEvent event) {
    _pointerDown = event.localPosition;
    _selecting = false;
    widget.controller.beginGlyphSelection(
      widget.pageIndex,
      event.localPosition.dx,
      event.localPosition.dy,
    );
    _focusNode.requestFocus();
  }

  void _onPointerMove(PointerMoveEvent event) {
    final origin = _pointerDown;
    if (origin == null) return;
    final delta = event.localPosition - origin;
    // Prefer vertical page scroll; only start a text selection once the drag
    // looks horizontal (or after a small intentional move).
    if (!_selecting) {
      if (delta.dy.abs() > delta.dx.abs() && delta.dy.abs() > 8) {
        _pointerDown = null;
        return;
      }
      if (delta.distance < 4) return;
      _selecting = true;
    }
    widget.controller.updateGlyphSelection(
      widget.pageIndex,
      event.localPosition.dx,
      event.localPosition.dy,
    );
  }

  void _onPointerUp(PointerUpEvent event) {
    if (_selecting) {
      widget.controller.endGlyphSelection(
        widget.pageIndex,
        event.localPosition.dx,
        event.localPosition.dy,
      );
    }
    _pointerDown = null;
    _selecting = false;
  }

  @override
  Widget build(BuildContext context) {
    final onCaretPage = widget.pageIndex == widget.controller.caretPage;
    final caret = onCaretPage ? widget.controller.caretGeometry : null;
    final selection = onCaretPage ? widget.controller.selectionRects : const <GlyphSelectionRect>[];
    return Focus(
      focusNode: _focusNode,
      onKeyEvent: _handleKey,
      child: Listener(
        behavior: HitTestBehavior.translucent,
        onPointerDown: _onPointerDown,
        onPointerMove: _onPointerMove,
        onPointerUp: _onPointerUp,
        onPointerCancel: (_) {
          _pointerDown = null;
          _selecting = false;
        },
        child: Stack(
          children: [
            CustomPaint(
              size: Size(widget.controller.pageWidth, widget.controller.pageHeight),
              painter: DocumentPainter(
                snapshot: widget.snapshot,
                atlasImage: widget.atlasImage,
                images: widget.images,
              ),
            ),
            if (selection.isNotEmpty)
              CustomPaint(
                size: Size(widget.controller.pageWidth, widget.controller.pageHeight),
                painter: _SelectionPainter(rects: selection),
              ),
            if (caret != null && selection.isEmpty)
              Positioned(
                left: caret.x,
                top: caret.y - caret.height,
                child: Container(
                  width: 1.5,
                  height: caret.height,
                  color: Colors.black,
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _SelectionPainter extends CustomPainter {
  _SelectionPainter({required this.rects});

  final List<GlyphSelectionRect> rects;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..color = const Color(0x553B82F6);
    for (final rect in rects) {
      canvas.drawRect(
        Rect.fromLTWH(rect.x, rect.y, rect.width, rect.height),
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _SelectionPainter oldDelegate) =>
      oldDelegate.rects != rects;
}
