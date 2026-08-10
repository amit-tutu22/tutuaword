import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/editor/document_painter.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/formatting_marks.dart';
import 'package:tutuaword/editor/image_hit_test.dart';
import 'package:tutuaword/editor/key_event_text.dart';

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
  bool _resizingImage = false;
  bool _movingImage = false;

  void _requestEditorFocus() {
    if (kIsWeb) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) widget.controller.focusGlyphInput();
      });
    } else {
      _focusNode.requestFocus();
    }
  }

  @override
  void initState() {
    super.initState();
    _focusNode = FocusNode();
    widget.controller.addListener(_onControllerUpdate);
    if (widget.pageIndex == 0) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        widget.controller.ensureGlyphCaret();
        _requestEditorFocus();
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
      unawaited(widget.controller.deleteGlyphBackward());
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.delete) {
      unawaited(widget.controller.deleteGlyphForward());
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.enter || key == LogicalKeyboardKey.numpadEnter) {
      unawaited(widget.controller.insertGlyphParagraphBreak());
      return KeyEventResult.handled;
    }
    // Tab must be handled here: Flutter steals it for focus traversal when
    // ignored, and `event.character` is often null for Tab on macOS.
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
    // (notably Space). On web, character is usually null for all keys.
    String? char = printableCharacterFromKeyEvent(event);
    if (char == null &&
        (key == LogicalKeyboardKey.space || key.keyLabel.toLowerCase() == 'space')) {
      char = ' ';
    }
    if (char == '\t') {
      unawaited(widget.controller.insertGlyphCharacter('\t'));
      return KeyEventResult.handled;
    }
    // Never treat Enter / Return as a printable character (avoids □ tofu).
    if (char == '\n' || char == '\r') {
      unawaited(widget.controller.insertGlyphParagraphBreak());
      return KeyEventResult.handled;
    }
    if (char != null &&
        char.isNotEmpty &&
        !HardwareKeyboard.instance.isControlPressed &&
        !HardwareKeyboard.instance.isMetaPressed) {
      unawaited(widget.controller.insertGlyphCharacter(char));
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _onPointerDown(PointerDownEvent event) {
    _pointerDown = event.localPosition;
    _selecting = false;
    _resizingImage = false;
    _movingImage = false;

    final controller = widget.controller;
    final onImagePage = controller.selectedImagePage == widget.pageIndex;
    if (onImagePage && controller.hasSelectedImage) {
      final handle = controller.imageHandleAt(event.localPosition);
      if (handle != null) {
        controller.beginImageResize(handle);
        _resizingImage = true;
        _requestEditorFocus();
        return;
      }
      if (controller.isPointOnSelectedImage(event.localPosition)) {
        controller.beginImageMove(event.localPosition);
        _movingImage = true;
        _requestEditorFocus();
        return;
      }
    }

    if (controller.trySelectDiagramAt(
      widget.pageIndex,
      event.localPosition,
      widget.snapshot,
    )) {
      _requestEditorFocus();
      return;
    }

    if (controller.trySelectImageAt(
      widget.pageIndex,
      event.localPosition,
      widget.snapshot,
    )) {
      _requestEditorFocus();
      return;
    }

    if (controller.isPointInGlyphSelection(
      widget.pageIndex,
      event.localPosition,
    )) {
      _draggingText = true;
      controller.beginGlyphDrag(widget.pageIndex);
    } else {
      _draggingText = false;
      controller.beginGlyphSelection(
        widget.pageIndex,
        event.localPosition.dx,
        event.localPosition.dy,
      );
    }
    _requestEditorFocus();
  }

  void _onPointerMove(PointerMoveEvent event) {
    if (_resizingImage) {
      widget.controller.updateImageResize(
        event.localPosition,
        lockAspectRatio: HardwareKeyboard.instance.isShiftPressed,
      );
      return;
    }
    if (_movingImage) {
      widget.controller.updateImageMove(event.localPosition);
      return;
    }
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
    if (_resizingImage) {
      unawaited(widget.controller.commitImageResize());
      _resizingImage = false;
    } else if (_movingImage) {
      unawaited(widget.controller.commitImageMove());
      _movingImage = false;
    } else if (_draggingText) {
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
        SingleActivator(LogicalKeyboardKey.keyA, control: true): _SelectAllIntent(),
      },
      child: Actions(
        actions: <Type, Action<Intent>>{
          _InsertTabIntent: CallbackAction<_InsertTabIntent>(
            onInvoke: (_) {
              if (widget.controller.isInList) {
                widget.controller.promoteListLevel();
              } else {
                unawaited(widget.controller.insertGlyphCharacter('\t'));
              }
              return null;
            },
          ),
          _OutdentIntent: CallbackAction<_OutdentIntent>(
            onInvoke: (_) {
              if (widget.controller.isInList) {
                widget.controller.demoteListLevel();
              } else {
                widget.controller.decreaseIndent();
              }
              return null;
            },
          ),
          _SelectAllIntent: CallbackAction<_SelectAllIntent>(
            onInvoke: (_) {
              unawaited(widget.controller.selectAll().catchError((_) {}));
              return null;
            },
          ),
        },
        child: kIsWeb
            ? _buildEditorStack(caret: caret, selection: selection)
            : Focus(
                focusNode: _focusNode,
                onKeyEvent: _handleKey,
                child: _buildEditorStack(caret: caret, selection: selection),
              ),
      ),
    );
  }

  Widget _buildEditorStack({
    required CaretGeometry? caret,
    required List<GlyphSelectionRect> selection,
  }) {
    return Stack(
      children: [
        GestureDetector(
          behavior: HitTestBehavior.translucent,
          onDoubleTapDown: (details) {
            unawaited(() async {
              final opened = await widget.controller.editChartDataAt(
                context,
                widget.pageIndex,
                details.localPosition,
                widget.snapshot,
              );
              if (opened) {
                _requestEditorFocus();
                return;
              }
              widget.controller.selectGlyphWordAt(
                widget.pageIndex,
                details.localPosition.dx,
                details.localPosition.dy,
              );
              _requestEditorFocus();
            }());
          },
          child: Listener(
            behavior: HitTestBehavior.translucent,
            onPointerDown: _onPointerDown,
            onPointerMove: _onPointerMove,
            onPointerUp: _onPointerUp,
            onPointerCancel: (_) {
              if (_resizingImage) {
                widget.controller.cancelImageResize();
                _resizingImage = false;
              }
              if (_movingImage) {
                widget.controller.cancelImageMove();
                _movingImage = false;
              }
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
                if (widget.controller.showFormattingMarks)
                  CustomPaint(
                    size: Size(
                      widget.controller.pageWidth,
                      widget.controller.pageHeight,
                    ),
                    painter: FormattingMarksPainter(
                      marks: widget.controller
                          .formattingMarksForPage(widget.pageIndex),
                    ),
                  ),
                if (widget.controller.selectedDiagramPage == widget.pageIndex &&
                    widget.controller.selectedDiagramRect != null)
                  CustomPaint(
                    size: Size(widget.controller.pageWidth, widget.controller.pageHeight),
                    painter: _DiagramSelectionPainter(
                      bounds: widget.controller.selectedDiagramRect!,
                    ),
                  ),
                if (widget.controller.selectedImagePage == widget.pageIndex &&
                    widget.controller.selectedImageRect != null)
                  CustomPaint(
                    size: Size(widget.controller.pageWidth, widget.controller.pageHeight),
                    painter: _ImageSelectionPainter(
                      bounds: widget.controller.selectedImageRect!,
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
      ],
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

class _ImageSelectionPainter extends CustomPainter {
  _ImageSelectionPainter({required this.bounds});

  final Rect bounds;

  @override
  void paint(Canvas canvas, Size size) {
    final border = Paint()
      ..color = const Color(0xFF2563EB)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    canvas.drawRect(bounds, border);

    final handlePaint = Paint()..color = const Color(0xFF2563EB);
    for (final point in imageHandlePoints(bounds)) {
      canvas.drawCircle(point, 4, handlePaint);
      canvas.drawCircle(
        point,
        4,
        Paint()
          ..color = Colors.white
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _ImageSelectionPainter oldDelegate) =>
      oldDelegate.bounds != bounds;
}

class _DiagramSelectionPainter extends CustomPainter {
  _DiagramSelectionPainter({required this.bounds});

  final Rect bounds;

  @override
  void paint(Canvas canvas, Size size) {
    // Match image selection chrome so object selection is obvious.
    final border = Paint()
      ..color = const Color(0xFF2563EB)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    canvas.drawRect(bounds, border);

    final handlePaint = Paint()..color = const Color(0xFF2563EB);
    const handles = <Offset>[
      Offset(0, 0),
      Offset(0.5, 0),
      Offset(1, 0),
      Offset(0, 0.5),
      Offset(1, 0.5),
      Offset(0, 1),
      Offset(0.5, 1),
      Offset(1, 1),
    ];
    for (final h in handles) {
      final point = Offset(
        bounds.left + bounds.width * h.dx,
        bounds.top + bounds.height * h.dy,
      );
      canvas.drawCircle(point, 4, handlePaint);
      canvas.drawCircle(
        point,
        4,
        Paint()
          ..color = Colors.white
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _DiagramSelectionPainter oldDelegate) =>
      oldDelegate.bounds != bounds;
}
