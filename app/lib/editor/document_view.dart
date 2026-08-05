import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_painter.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/editor/page_navigator.dart';
import 'package:tutuaword/editor/rulers.dart';
import 'package:tutuaword/ui/word_theme.dart';

class DocumentView extends StatefulWidget {
  const DocumentView({super.key, required this.controller});

  final EditorController controller;

  @override
  State<DocumentView> createState() => _DocumentViewState();
}

/// Vertical gap between pages in the continuous scroll, in unscaled points.
const double _pageGap = 24.0;

class _DocumentViewState extends State<DocumentView> {
  /// Per-page display lists. Every page of a layout version shares one atlas,
  /// so the decoded texture is kept separately and reused across pages.
  final Map<int, DisplayListSnapshot> _snapshots = {};
  final Set<int> _pending = {};
  ui.Image? _atlasImage;
  bool _buildingAtlas = false;
  int _loadedVersion = -1;
  final ScrollController _scrollController = ScrollController();

  /// Decoded document images shared across pages, keyed by asset id.
  final Map<String, ui.Image> _images = {};

  @override
  void initState() {
    super.initState();
    widget.controller.addListener(_onControllerUpdate);
    _scrollController.addListener(_onScroll);
    _loadedVersion = widget.controller.displayVersion;
  }

  @override
  void dispose() {
    widget.controller.removeListener(_onControllerUpdate);
    _scrollController.dispose();
    _atlasImage?.dispose();
    _disposeImages();
    super.dispose();
  }

  void _disposeImages() {
    for (final image in _images.values) {
      image.dispose();
    }
    _images.clear();
  }

  void _onControllerUpdate() {
    if (widget.controller.displayVersion != _loadedVersion) {
      _loadedVersion = widget.controller.displayVersion;
      _snapshots.clear();
      _pending.clear();
      _atlasImage?.dispose();
      _atlasImage = null;
      _buildingAtlas = false;
      _disposeImages();
    }
    if (mounted) setState(() {});
  }

  double get _pageExtent =>
      (widget.controller.pageHeight + _pageGap) * widget.controller.zoom;

  void _onScroll() {
    final extent = _pageExtent;
    if (extent <= 0 || !_scrollController.hasClients) return;
    final page = (_scrollController.offset / extent).floor();
    widget.controller.setVisiblePage(page);
  }

  Future<void> _loadPage(int index) async {
    final version = widget.controller.displayVersion;
    final bytes = widget.controller.displayListForPage(index);
    if (bytes.isEmpty) {
      _pending.remove(index);
      return;
    }

    final snapshot = DisplayListSnapshot.fromBytes(bytes);
    ui.Image? atlas;
    if (_atlasImage == null && !_buildingAtlas && snapshot.hasPaintableGlyphs) {
      _buildingAtlas = true;
      atlas = await snapshot.buildAtlasImage();
    }
    final decodedImages = await snapshot.decodeImages(skip: _images.keys.toSet());

    if (!mounted || version != widget.controller.displayVersion) {
      atlas?.dispose();
      for (final image in decodedImages.values) {
        image.dispose();
      }
      _pending.remove(index);
      return;
    }

    setState(() {
      _snapshots[index] = snapshot;
      if (atlas != null) {
        if (_atlasImage == null) {
          _atlasImage = atlas;
        } else {
          atlas.dispose();
        }
        _buildingAtlas = false;
      }
      decodedImages.forEach((id, image) {
        // A concurrent page load may have decoded the same asset first.
        if (_images.containsKey(id)) {
          image.dispose();
        } else {
          _images[id] = image;
        }
      });
      _pending.remove(index);
    });
  }

  void _scheduleLoad(int index) {
    if (_snapshots.containsKey(index) || _pending.contains(index)) return;
    _pending.add(index);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _loadPage(index);
    });
  }

  void _scrollToPage(int page) {
    widget.controller.setCurrentPage(page);
    if (_scrollController.hasClients) {
      _scrollController.animateTo(
        page * _pageExtent,
        duration: const Duration(milliseconds: 250),
        curve: Curves.easeOut,
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final controller = widget.controller;
    return Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (controller.showNavigationPane)
          PageNavigator(
            controller: controller,
            currentPage: controller.currentPage,
            onPageSelected: _scrollToPage,
          ),
        Expanded(
          child: Container(
            color: WordTheme.canvasGray,
            child: controller.showRuler
                ? DocumentRulers(
                    controller: controller,
                    child: _buildCanvas(context, controller),
                  )
                : _buildCanvas(context, controller),
          ),
        ),
      ],
    );
  }

  Widget _buildCanvas(BuildContext context, EditorController controller) {
    return ListView.builder(
      controller: _scrollController,
      padding: const EdgeInsets.symmetric(vertical: _pageGap),
      itemCount: controller.pageCount,
      itemExtent: _pageExtent,
      itemBuilder: (context, index) {
        _scheduleLoad(index);
        // In text-fallback mode a single TextEditingController is shared with
        // the controller, so only the active page may host the editor.
        final readOnly =
            controller.preferTextRendering && index != controller.currentPage;
        return Center(
          child: Transform.scale(
            scale: controller.zoom,
            alignment: Alignment.topCenter,
            child: GestureDetector(
              onTap: readOnly ? () => controller.setCurrentPage(index) : null,
              child: _PageCanvas(
                key: ValueKey('page-$index-${controller.displayVersion}'),
                controller: controller,
                pageIndex: index,
                snapshot: _snapshots[index],
                atlasImage: _atlasImage,
                images: _images,
                readOnly: readOnly,
              ),
            ),
          ),
        );
      },
    );
  }
}

class _PageCanvas extends StatefulWidget {
  const _PageCanvas({
    super.key,
    required this.controller,
    required this.pageIndex,
    required this.snapshot,
    required this.atlasImage,
    required this.images,
    this.readOnly = false,
  });

  final EditorController controller;
  final int pageIndex;
  final DisplayListSnapshot? snapshot;
  final ui.Image? atlasImage;
  final Map<String, ui.Image> images;
  final bool readOnly;

  @override
  State<_PageCanvas> createState() => _PageCanvasState();
}

class _PageCanvasState extends State<_PageCanvas> {
  TextEditingController? _textController;
  FocusNode? _focusNode;
  bool _syncingFromController = false;

  bool get _canPaintDisplayList =>
      !widget.controller.preferTextRendering &&
      widget.snapshot != null &&
      widget.snapshot!.hasPaintableContent;

  bool get _useTextEditor => widget.controller.preferTextRendering;

  @override
  void initState() {
    super.initState();
    if (_useTextEditor) {
      _initTextEditor();
    }
  }

  void _initTextEditor() {
    _textController = TextEditingController(
      text: widget.controller.textForPage(widget.pageIndex),
    );
    _focusNode = FocusNode();
    _textController!.addListener(_onTextChanged);
    if (!widget.readOnly) {
      widget.controller.attachTextEditor(_textController!, _focusNode!);
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted && !widget.readOnly) _focusNode?.requestFocus();
    });
  }

  @override
  void didUpdateWidget(_PageCanvas oldWidget) {
    super.didUpdateWidget(oldWidget);
    final wasText = oldWidget.controller.preferTextRendering;
    final isText = widget.controller.preferTextRendering;
    if (wasText != isText) {
      if (isText && _textController == null) {
        _initTextEditor();
      } else if (!isText && _textController != null) {
        _disposeTextEditor();
      }
    }
    if (_textController != null &&
        (widget.pageIndex != oldWidget.pageIndex ||
            widget.controller.displayVersion != oldWidget.controller.displayVersion)) {
      final pageText = widget.controller.textForPage(widget.pageIndex);
      if (_textController!.text != pageText) {
        _syncingFromController = true;
        _textController!.text = pageText;
        _syncingFromController = false;
      }
    }
    if (_textController != null && !widget.readOnly) {
      widget.controller.attachTextEditor(_textController!, _focusNode!);
    }
  }

  void _disposeTextEditor() {
    _textController?.removeListener(_onTextChanged);
    if (!widget.readOnly && _textController != null) {
      widget.controller.detachTextEditor(_textController!);
    }
    _textController?.dispose();
    _focusNode?.dispose();
    _textController = null;
    _focusNode = null;
  }

  @override
  void dispose() {
    _disposeTextEditor();
    super.dispose();
  }

  void _onTextChanged() {
    if (_syncingFromController || _textController == null) return;
    widget.controller.replacePageText(widget.pageIndex, _textController!.text);
  }

  TextStyle get _textStyle {
    final c = widget.controller;
    var decoration = TextDecoration.none;
    if (c.underline) decoration = TextDecoration.underline;
    if (c.strikethrough) {
      decoration = decoration == TextDecoration.none
          ? TextDecoration.lineThrough
          : TextDecoration.combine([decoration, TextDecoration.lineThrough]);
    }
    return TextStyle(
      fontFamily: c.fontFamily,
      fontSize: c.fontSize,
      height: 1.4,
      fontWeight: c.bold ? FontWeight.bold : FontWeight.normal,
      fontStyle: c.italic ? FontStyle.italic : FontStyle.normal,
      decoration: decoration,
    );
  }

  @override
  Widget build(BuildContext context) {
    final pageText = widget.controller.textForPage(widget.pageIndex);

    return Container(
      width: widget.controller.pageWidth,
      height: widget.controller.pageHeight,
      decoration: BoxDecoration(
        color: Colors.white,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.15),
            blurRadius: 8,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Stack(
        children: [
          if (_canPaintDisplayList && !widget.readOnly && widget.snapshot != null)
            GlyphEditorSurface(
              controller: widget.controller,
              pageIndex: widget.pageIndex,
              snapshot: widget.snapshot!,
              atlasImage: widget.atlasImage,
              images: widget.images,
            )
          else if (_canPaintDisplayList && widget.snapshot != null)
            CustomPaint(
              size: Size(widget.controller.pageWidth, widget.controller.pageHeight),
              painter: DocumentPainter(
                snapshot: widget.snapshot!,
                atlasImage: widget.atlasImage,
                images: widget.images,
              ),
            )
          else if (_useTextEditor && !widget.readOnly && _textController != null)
            Padding(
              padding: const EdgeInsets.all(72),
              child: TextField(
                controller: _textController,
                focusNode: _focusNode,
                maxLines: null,
                style: _textStyle,
                textAlign: widget.controller.alignment,
                decoration: const InputDecoration(
                  border: InputBorder.none,
                  isCollapsed: true,
                  contentPadding: EdgeInsets.zero,
                ),
                cursorColor: Colors.black,
                selectionControls: materialTextSelectionControls,
              ),
            )
          else if (_useTextEditor && widget.readOnly)
            Padding(
              padding: const EdgeInsets.all(72),
              child: Text(
                pageText,
                style: _textStyle,
                textAlign: widget.controller.alignment,
              ),
            )
          else
            const Padding(
              padding: EdgeInsets.all(72),
              child: Text('Start typing…', style: TextStyle(fontSize: 12)),
            ),
        ],
      ),
    );
  }
}
