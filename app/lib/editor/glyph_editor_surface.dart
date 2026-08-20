import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/editor/document_painter.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_input.dart';
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
  bool _movingDiagram = false;
  bool _glyphTap = false;
  DateTime? _lastShapeTapAt;
  String? _lastShapeTapId;
  int _seenEditorFocusEpoch = 0;
  int _lastRepaintDisplayVersion = -1;
  int _lastRepaintPageVersion = -1;

  void _requestEditorFocus() {
    if (EditorController.usesSoftKeyboardGlyphInput) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) widget.controller.focusGlyphInput();
      });
    } else {
      _focusNode.requestFocus();
      // Ribbon / dialog rebuilds can steal focus in the same frame as a click.
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _focusNode.requestFocus();
      });
    }
  }

  @override
  void initState() {
    super.initState();
    _focusNode = FocusNode(debugLabel: EditorController.glyphEditorFocusLabel);
    widget.controller.addListener(_onControllerUpdate);
    if (widget.pageIndex == 0) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        widget.controller.ensureGlyphCaret();
        if (widget.pageIndex == widget.controller.caretPage) {
          _requestEditorFocus();
        }
      });
    }
  }

  @override
  void dispose() {
    widget.controller.removeListener(_onControllerUpdate);
    _focusNode.dispose();
    super.dispose();
  }

  void _onControllerUpdate() {
    final epoch = widget.controller.editorFocusEpoch;
    final focusPage = widget.pageIndex == widget.controller.currentPage ||
        widget.pageIndex == widget.controller.caretPage ||
        widget.pageIndex == widget.controller.selectedDiagramPage;
    if (epoch != _seenEditorFocusEpoch && focusPage) {
      _seenEditorFocusEpoch = epoch;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _requestEditorFocus();
      });
    }
    final displayVersion = widget.controller.displayVersion;
    final pageVersion = widget.controller.pageDisplayVersion(widget.pageIndex);
    final caretOnPage = widget.controller.caretPage == widget.pageIndex;
    final selectionOnPage = widget.controller.selection?.page == widget.pageIndex;
    final objectOnPage =
        widget.controller.selectedDiagramPage == widget.pageIndex ||
            widget.controller.selectedImagePage == widget.pageIndex;
    if (displayVersion == _lastRepaintDisplayVersion &&
        pageVersion == _lastRepaintPageVersion &&
        !caretOnPage &&
        !selectionOnPage &&
        !objectOnPage) {
      return;
    }
    _lastRepaintDisplayVersion = displayVersion;
    _lastRepaintPageVersion = pageVersion;
    setState(() {});
  }

  KeyEventResult _handleKey(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }

    // Map editing keys by logical identity first. Platforms often attach a
    // DEL (U+007F) character to Delete; treating that as text swallows forward
    // delete. Tab must also win before WidgetsApp's NextFocusIntent.
    final editing = EditorInputEvent.fromKeyEvent(event);
    if (editing != null && editing.kind != EditorInputKind.character) {
      unawaited(widget.controller.handleEditorInput(editing));
      return KeyEventResult.handled;
    }

    final char = printableCharacterFromKeyEvent(event);
    if (char != null &&
        char.isNotEmpty &&
        !HardwareKeyboard.instance.isControlPressed &&
        !HardwareKeyboard.instance.isMetaPressed) {
      unawaited(widget.controller.handleEditorInput(EditorInputEvent.character(char)));
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _onPointerDown(PointerDownEvent event) {
    _pointerDown = event.localPosition;
    _selecting = false;
    _resizingImage = false;
    _movingImage = false;
    _movingDiagram = false;
    _glyphTap = false;

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

    final onDiagramPage = controller.selectedDiagramPage == widget.pageIndex;
    if (onDiagramPage &&
        controller.hasSelectedDiagram &&
        controller.isPointOnSelectedDiagram(event.localPosition)) {
      final id = controller.selectedDiagramId;
      final now = DateTime.now();
      final isDoubleTap = id != null &&
          id == _lastShapeTapId &&
          _lastShapeTapAt != null &&
          now.difference(_lastShapeTapAt!) < const Duration(milliseconds: 400);
      _lastShapeTapAt = now;
      _lastShapeTapId = id;
      if (isDoubleTap) {
        unawaited(controller.enterSelectedShapeTextEdit());
        _requestEditorFocus();
        return;
      }
      controller.beginShapeMove(event.localPosition);
      _movingDiagram = true;
      _requestEditorFocus();
      return;
    }

    if (controller.trySelectDiagramAt(
      widget.pageIndex,
      event.localPosition,
      widget.snapshot,
    )) {
      final id = controller.selectedDiagramId;
      final now = DateTime.now();
      final isDoubleTap = id != null &&
          id == _lastShapeTapId &&
          _lastShapeTapAt != null &&
          now.difference(_lastShapeTapAt!) < const Duration(milliseconds: 400);
      _lastShapeTapAt = now;
      _lastShapeTapId = id;
      if (isDoubleTap && controller.hasSelectedDiagram) {
        unawaited(controller.enterSelectedShapeTextEdit());
      }
      _requestEditorFocus();
      return;
    }
    _lastShapeTapAt = null;
    _lastShapeTapId = null;

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
      _glyphTap = true;
      if (HardwareKeyboard.instance.isShiftPressed) {
        controller.extendGlyphSelectionTo(
          widget.pageIndex,
          event.localPosition.dx,
          event.localPosition.dy,
        );
      } else {
        controller.beginGlyphSelection(
          widget.pageIndex,
          event.localPosition.dx,
          event.localPosition.dy,
        );
      }
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
    if (_movingDiagram) {
      widget.controller.updateShapeMove(event.localPosition);
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
    } else if (_movingDiagram) {
      unawaited(widget.controller.commitShapeMove());
      _movingDiagram = false;
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
    } else if (_glyphTap) {
      final allowExternal = EditorController.usesSoftKeyboardGlyphInput ||
          HardwareKeyboard.instance.isControlPressed ||
          HardwareKeyboard.instance.isMetaPressed;
      if (!mounted) return;
      unawaited(widget.controller.tryFollowHyperlink(
        allowExternal: allowExternal,
        context: context,
      ));
    }
    _pointerDown = null;
    _selecting = false;
    _glyphTap = false;
  }

  @override
  Widget build(BuildContext context) {
    final onCaretPage = widget.pageIndex == widget.controller.caretPage;
    // Object selection (handles) suppresses the text caret — Word does the same.
    final caret = onCaretPage &&
            !widget.controller.hasSelectedDiagram &&
            !widget.controller.hasSelectedImage
        ? widget.controller.caretGeometry
        : null;
    final selection = widget.controller.hasGlyphSelection
        ? widget.controller.selectionRectsForPage(widget.pageIndex)
        : const <GlyphSelectionRect>[];
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
              unawaited(widget.controller.handleEditorInput(const EditorInputEvent.tab()));
              return null;
            },
          ),
          _OutdentIntent: CallbackAction<_OutdentIntent>(
            onInvoke: (_) {
              unawaited(
                widget.controller.handleEditorInput(const EditorInputEvent.tab(shift: true)),
              );
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
        child: EditorController.usesSoftKeyboardGlyphInput
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
              if (_movingDiagram) {
                widget.controller.cancelShapeMove();
                _movingDiagram = false;
              }
              if (_draggingText) {
                widget.controller.cancelGlyphDrag();
              }
              _pointerDown = null;
              _selecting = false;
              _draggingText = false;
              _glyphTap = false;
            },
            child: Stack(
              children: [
                CustomPaint(
                  size: Size(widget.controller.pageWidth, widget.controller.pageHeight),
                  painter: DocumentPainter(
                    snapshot: widget.snapshot,
                    atlasImage: widget.atlasImage,
                    images: widget.images,
                    dragSourceRect: widget.controller.contentDragSourceRect,
                    dragOffset: widget.controller.contentDragOffset,
                  ),
                ),
                if (widget.controller.showFormattingMarks)
                  CustomPaint(
                    size: Size(
                      widget.controller.pageWidth,
                      widget.controller.pageHeight,
                    ),
                    painter: FormattingMarksPainter(
                      marks: widget.controller.formattingMarksForPage(
                        widget.pageIndex,
                        widget.snapshot,
                      ),
                    ),
                  ),
                if (widget.controller.selectedDiagramPage == widget.pageIndex &&
                    widget.controller.selectedDiagramRect != null)
                  CustomPaint(
                    key: const Key('shape_selection_handles'),
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
                    key: const Key('glyph_caret'),
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
