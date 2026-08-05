import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
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
    widget.controller.hitTestAt(
      widget.pageIndex,
      details.localPosition.dx,
      details.localPosition.dy,
    );
    _focusNode.requestFocus();
  }

  @override
  Widget build(BuildContext context) {
    final caret = widget.controller.caretGeometry;
    return Focus(
      focusNode: _focusNode,
      onKeyEvent: _handleKey,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: _handleTapDown,
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
            if (caret != null)
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
