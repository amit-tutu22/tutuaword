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
  bool _draggingText = false;

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
    if (key == LogicalKeyboardKey.delete) {
      widget.controller.deleteGlyphForward();
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.enter || key == LogicalKeyboardKey.numpadEnter) {
      widget.controller.insertGlyphParagraphBreak();
      return KeyEventResult.handled;
    }
    // Tab must be handled here: Flutter steals it for focus traversal when
    // ignored, and `event.character` is often null for Tab on macOS.
    if (key == LogicalKeyboardKey.tab) {
      if (HardwareKeyboard.instance.isShiftPressed) {
        widget.controller.decreaseIndent();
      } else {
        widget.controller.insertGlyphCharacter('\t');
      }
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
    if (char == '\t') {
      widget.controller.insertGlyphCharacter('\t');
      return KeyEventResult.handled;
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
    if (widget.controller.isPointInGlyphSelection(
      widget.pageIndex,
      event.localPosition,
    )) {
      _draggingText = true;
      widget.controller.beginGlyphDrag(widget.pageIndex);
    } else {
      _draggingText = false;
      widget.controller.beginGlyphSelection(
        widget.pageIndex,
        event.localPosition.dx,
        event.localPosition.dy,
      );
    }
    _focusNode.requestFocus();
  }

  void _onPointerMove(PointerMoveEvent event) {
    if (_draggingText) {
      widget.controller.updateGlyphDragDropCaret(
        widget.pageIndex,
        event.localPosition.dx,
        event.localPosition.dy,
      );
      return;
    }
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
    if (_draggingText) {
      widget.controller.completeGlyphDrag(
        widget.pageIndex,
        event.localPosition.dx,
        event.localPosition.dy,
      );
      _draggingText = false;
    } else if (_selecting) {
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
    // Rebind Tab before WidgetsApp's NextFocusIntent steals it.
    return Shortcuts(
      shortcuts: const <ShortcutActivator, Intent>{
        SingleActivator(LogicalKeyboardKey.tab): _InsertTabIntent(),
        SingleActivator(LogicalKeyboardKey.tab, shift: true): _OutdentIntent(),
        SingleActivator(LogicalKeyboardKey.keyA, meta: true): _SelectAllIntent(),
      },
      child: Actions(
        actions: <Type, Action<Intent>>{
          _InsertTabIntent: CallbackAction<_InsertTabIntent>(
            onInvoke: (_) {
              widget.controller.insertGlyphCharacter('\t');
              return null;
            },
          ),
          _OutdentIntent: CallbackAction<_OutdentIntent>(
            onInvoke: (_) {
              widget.controller.decreaseIndent();
              return null;
            },
          ),
          _SelectAllIntent: CallbackAction<_SelectAllIntent>(
            onInvoke: (_) {
              widget.controller.selectAll();
              return null;
            },
          ),
        },
        child: Focus(
          focusNode: _focusNode,
          onKeyEvent: _handleKey,
          child: GestureDetector(
            behavior: HitTestBehavior.translucent,
            onDoubleTapDown: (details) {
              widget.controller.selectGlyphWordAt(
                widget.pageIndex,
                details.localPosition.dx,
                details.localPosition.dy,
              );
              _focusNode.requestFocus();
            },
            child: Listener(
              behavior: HitTestBehavior.translucent,
              onPointerDown: _onPointerDown,
              onPointerMove: _onPointerMove,
              onPointerUp: _onPointerUp,
              onPointerCancel: (_) {
                if (_draggingText) {
                  widget.controller.cancelGlyphDrag();
                }
                _pointerDown = null;
                _selecting = false;
                _draggingText = false;
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
                if (caret != null && selection.isEmpty && !widget.controller.isGlyphDragActive)
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
        ),
      ),
      ),
    );
  }
}

class _InsertTabIntent extends Intent {
  const _InsertTabIntent();
}

class _OutdentIntent extends Intent {
  const _OutdentIntent();
}

class _SelectAllIntent extends Intent {
  const _SelectAllIntent();
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
