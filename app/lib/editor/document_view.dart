import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_painter.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/editor/navigation_pane.dart';
import 'package:tutuaword/editor/style_inspector_pane.dart';
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
  final Map<int, int> _loadedPageVersions = {};
  final Set<int> _pending = {};
  ui.Image? _atlasImage;
  bool _buildingAtlas = false;
  int _loadedVersion = -1;
  int _loadedAtlasGeneration = -1;
  final ScrollController _scrollController = ScrollController();

  /// Decoded document images shared across pages, keyed by asset id.
  final Map<String, ui.Image> _images = {};

  @override
  void initState() {
    super.initState();
    widget.controller.addListener(_onControllerUpdate);
    _scrollController.addListener(_onScroll);
    _loadedVersion = widget.controller.displayVersion;
    _loadedAtlasGeneration = widget.controller.atlasGeneration;
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
      // Drop only pages whose per-page version changed (R1.3 lazy invalidation).
      _loadedPageVersions.removeWhere((page, version) {
        return widget.controller.pageDisplayVersion(page) != version;
      });
      _snapshots.removeWhere((page, _) => !_loadedPageVersions.containsKey(page));
      _pending.removeWhere((page) => !_loadedPageVersions.containsKey(page));
    }
    if (widget.controller.atlasGeneration != _loadedAtlasGeneration) {
      _loadedAtlasGeneration = widget.controller.atlasGeneration;
      _atlasImage?.dispose();
      _atlasImage = null;
      _buildingAtlas = false;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _ensureAtlasTexture(widget.controller.displayVersion);
      });
    }
    final scrollPage = widget.controller.view.takeScrollRequest();
    if (scrollPage != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _scrollToPage(scrollPage);
      });
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
    final pageVersion = widget.controller.pageDisplayVersion(index);
    if (_loadedPageVersions[index] == pageVersion &&
        _snapshots.containsKey(index)) {
      return;
    }
    final version = widget.controller.displayVersion;
    final bytes = widget.controller.displayListForPage(index);
    if (bytes.isEmpty) {
      if (!mounted || version != widget.controller.displayVersion) {
        _pending.remove(index);
        return;
      }
      // Cache an empty snapshot so itemBuilder stops scheduling loads every
      // frame (otherwise pumpAndSettle never completes in widget tests).
      setState(() {
        _snapshots[index] = DisplayListSnapshot.empty();
        _pending.remove(index);
      });
      return;
    }

    final snapshot = DisplayListSnapshot.fromBytes(bytes);
    await _ensureAtlasTexture(version);

    final decodedImages = await snapshot.decodeImages(skip: _images.keys.toSet());

    if (!mounted || version != widget.controller.displayVersion) {
      for (final image in decodedImages.values) {
        image.dispose();
      }
      _pending.remove(index);
      return;
    }

    setState(() {
      _snapshots[index] = snapshot;
      _loadedPageVersions[index] = pageVersion;
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

  Future<void> _ensureAtlasTexture(int layoutVersion) async {
    final controller = widget.controller;
    final generation = controller.atlasGeneration;
    if (_atlasImage != null && generation == _loadedAtlasGeneration) {
      return;
    }
    if (_buildingAtlas || controller.atlasPixels.isEmpty) {
      return;
    }
    if (generation == 0 || controller.atlasWidth == 0 || controller.atlasHeight == 0) {
      return;
    }

    _buildingAtlas = true;
    final atlas = await DisplayListSnapshot.buildAtlasImageFromPixels(
      controller.atlasPixels,
      controller.atlasWidth,
      controller.atlasHeight,
    );

    if (!mounted || layoutVersion != controller.displayVersion) {
      atlas?.dispose();
      _buildingAtlas = false;
      return;
    }

    setState(() {
      _atlasImage?.dispose();
      _atlasImage = atlas;
      _loadedAtlasGeneration = generation;
      _buildingAtlas = false;
    });
  }

  void _scheduleLoad(int index) {
    final pageVersion = widget.controller.pageDisplayVersion(index);
    if (_loadedPageVersions[index] == pageVersion &&
        _snapshots.containsKey(index)) {
      return;
    }
    if (_pending.contains(index)) return;
    // Drop a stale snapshot so the pending load is the source of truth.
    _snapshots.remove(index);
    _loadedPageVersions.remove(index);
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
          NavigationPane(
            controller: controller,
            currentPage: controller.currentPage,
            onPageSelected: _scrollToPage,
            onOutlineSelected: controller.jumpToOutlineEntry,
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
        if (controller.showStyleInspector)
          StyleInspectorPane(controller: controller),
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
        final readOnly = !controller.isPageEditable(index);
        return Center(
          child: Transform.scale(
            scale: controller.zoom,
            alignment: Alignment.topCenter,
            child: ClipRect(
              clipBehavior: Clip.hardEdge,
              child: SizedBox(
                width: controller.pageWidth,
                height: controller.pageHeight,
                child: GestureDetector(
                  onTap: readOnly ? () => controller.setCurrentPage(index) : null,
                  child: _PageCanvas(
                    key: ValueKey('page-$index-${controller.pageDisplayVersion(index)}'),
                    controller: controller,
                    pageIndex: index,
                    snapshot: _snapshots[index],
                    atlasImage: _atlasImage,
                    images: _images,
                    readOnly: readOnly,
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _PageCanvas extends StatelessWidget {
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

  bool get _wantGlyphEditor =>
      controller.isPageEditable(pageIndex) && !readOnly;

  bool get _canPaintDisplayList =>
      snapshot != null && snapshot!.hasPaintableContent;

  @override
  Widget build(BuildContext context) {
    return ClipRect(
      clipBehavior: Clip.hardEdge,
      child: Container(
        width: controller.pageWidth,
        height: controller.pageHeight,
        decoration: BoxDecoration(
          color: Colors.white,
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.15),
              blurRadius: 8,
              offset: const Offset(0, 2),
            ),
          ],
          border: controller.printPreview
              ? Border.all(color: const Color(0xFFB4B4B4), width: 1)
              : null,
        ),
        child: Stack(
          children: [
            if (controller.printPreview)
              Positioned(
                top: 6,
                right: 8,
                child: Text(
                  'Page ${pageIndex + 1}',
                  style: TextStyle(
                    fontSize: 10,
                    color: Colors.grey.shade600,
                  ),
                ),
              ),
            if (_wantGlyphEditor)
              GlyphEditorSurface(
                controller: controller,
                pageIndex: pageIndex,
                snapshot: snapshot ?? DisplayListSnapshot.empty(),
                atlasImage: atlasImage,
                images: images,
              )
            else if (_canPaintDisplayList && snapshot != null)
              CustomPaint(
                size: Size(controller.pageWidth, controller.pageHeight),
                painter: DocumentPainter(
                  snapshot: snapshot!,
                  atlasImage: atlasImage,
                  images: images,
                ),
              )
            else if (controller.printPreview)
              Center(
                child: Text(
                  'Page ${pageIndex + 1}',
                  style: TextStyle(color: Colors.grey.shade500, fontSize: 12),
                ),
              )
            else if (!controller.isEngineConnected)
              const Padding(
                padding: EdgeInsets.all(72),
                child: Text(
                  'Rust engine unavailable.\nRun scripts/build-ffi.sh to enable editing.',
                  style: TextStyle(fontSize: 12, color: Colors.grey),
                ),
              )
            else
              const Padding(
                padding: EdgeInsets.all(72),
                child: Text('Start typing…', style: TextStyle(fontSize: 12)),
              ),
          ],
        ),
      ),
    );
  }
}
