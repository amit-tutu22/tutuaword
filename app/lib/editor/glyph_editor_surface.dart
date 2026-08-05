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

  @override
  void initState() {
    super.initState();
    _focusNode = FocusNode();
    widget.controller.addListener(_onControllerUpdate);
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
    final char = event.character;
    if (char != null &&
        char.isNotEmpty &&
        char.length == 1 &&
        !HardwareKeyboard.instance.isControlPressed) {
      widget.controller.insertGlyphCharacter(char);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _handleTapDown(TapDownDetails details) {
    widget.controller.beginGlyphSelection(
      widget.pageIndex,
      details.localPosition.dx,
      details.localPosition.dy,
    );
    _focusNode.requestFocus();
  }

  void _handlePanStart(DragStartDetails details) {
    widget.controller.beginGlyphSelection(
      widget.pageIndex,
      details.localPosition.dx,
      details.localPosition.dy,
    );
    _focusNode.requestFocus();
  }

  void _handlePanUpdate(DragUpdateDetails details) {
    widget.controller.updateGlyphSelection(
      widget.pageIndex,
      details.localPosition.dx,
      details.localPosition.dy,
    );
  }

  void _handlePanEnd(DragEndDetails details) {
    // Final geometry already updated during pan; keep focus for typing.
    _focusNode.requestFocus();
  }

  @override
  Widget build(BuildContext context) {
    final caret = widget.controller.caretGeometry;
    final selection = widget.controller.selectionRects;
    return Focus(
      focusNode: _focusNode,
      onKeyEvent: _handleKey,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: _handleTapDown,
        onPanStart: _handlePanStart,
        onPanUpdate: _handlePanUpdate,
        onPanEnd: _handlePanEnd,
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
