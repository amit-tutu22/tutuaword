import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_painter.dart';
import 'package:tutuaword/editor/document_view_layout.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/editor/navigation_pane.dart';
import 'package:tutuaword/editor/accessibility_checker_pane.dart';
import 'package:tutuaword/editor/picture_inspector_pane.dart';
import 'package:tutuaword/editor/style_inspector_pane.dart';
import 'package:tutuaword/editor/rulers.dart';
import 'package:tutuaword/editor/web_glyph_text_input.dart';
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
  final ScrollController _splitScrollController = ScrollController();

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
    _splitScrollController.dispose();
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

  double get _pageGapEffective {
    final layout = widget.controller.viewLayout;
    if (layout == DocumentViewLayout.webLayout) return 4.0;
    if (layout == DocumentViewLayout.readMode) return 16.0;
    return _pageGap;
  }

  double get _pageExtent {
    final gap = _pageGapEffective;
    return (widget.controller.pageHeight + gap) * widget.controller.zoom;
  }

  int get _rowCount {
    final pages = widget.controller.pageCount;
    final columns = widget.controller.pageColumns.clamp(1, 3);
    return (pages + columns - 1) ~/ columns;
  }

  void _onScroll() {
    final extent = _pageExtent;
    if (extent <= 0 || !_scrollController.hasClients) return;
    final row = (_scrollController.offset / extent).floor();
    final columns = widget.controller.pageColumns.clamp(1, 3);
    widget.controller.setVisiblePage(row * columns);
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
        _loadedPageVersions[index] = pageVersion;
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
    // Avoid [EditorController.setCurrentPage] here — it refreshes from the
    // engine and can collapse an injected multi-page pageCount back to 1.
    widget.controller.selectPage(page);
    if (_scrollController.hasClients) {
      final columns = widget.controller.pageColumns.clamp(1, 3);
      final row = page.clamp(0, widget.controller.pageCount - 1) ~/ columns;
      final target = row * _pageExtent;
      _scrollController.animateTo(
        target,
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
        if (controller.showNavigationPane && !controller.isReadMode)
          NavigationPane(
            controller: controller,
            currentPage: controller.currentPage,
            onPageSelected: controller.jumpToPage,
            onOutlineSelected: controller.jumpToOutlineEntry,
          ),
        Expanded(
          child: LayoutBuilder(
            builder: (context, constraints) {
              controller.reportViewportSize(
                Size(constraints.maxWidth, constraints.maxHeight),
              );
              final canvasBg = controller.isWebLayout || controller.isReadMode
                  ? Colors.white
                  : WordTheme.canvasGray;
              final body = Container(
                color: canvasBg,
                child: controller.showRuler &&
                        !controller.isReadMode &&
                        !controller.isWebLayout
                    ? DocumentRulers(
                        controller: controller,
                        child: _buildSplitOrCanvas(context, controller),
                      )
                    : _buildSplitOrCanvas(context, controller),
              );
              if (!controller.isReadMode) return body;
              return Stack(
                children: [
                  body,
                  Positioned(
                    top: 8,
                    right: 8,
                    child: Material(
                      color: Colors.black.withValues(alpha: 0.7),
                      borderRadius: BorderRadius.circular(4),
                      child: TextButton(
                        key: const Key('exit_read_mode'),
                        style: TextButton.styleFrom(
                          foregroundColor: Colors.white,
                          padding: const EdgeInsets.symmetric(
                            horizontal: 12,
                            vertical: 8,
                          ),
                        ),
                        onPressed: controller.setPrintLayout,
                        child: const Text('Close Read Mode'),
                      ),
                    ),
                  ),
                ],
              );
            },
          ),
        ),
        if (controller.hasSelectedImage && !controller.isReadMode)
          PictureInspectorPane(controller: controller),
        if (controller.showAccessibilityChecker && !controller.isReadMode)
          AccessibilityCheckerPane(controller: controller),
        if (controller.showStyleInspector && !controller.isReadMode)
          StyleInspectorPane(controller: controller),
      ],
    );
  }

  Widget _buildSplitOrCanvas(BuildContext context, EditorController controller) {
    if (!controller.splitView) {
      return _buildCanvas(
        context,
        controller,
        scrollController: _scrollController,
        trackVisiblePage: true,
      );
    }
    return Column(
      children: [
        Expanded(
          child: _buildCanvas(
            context,
            controller,
            scrollController: _scrollController,
            trackVisiblePage: true,
          ),
        ),
        MouseRegion(
          cursor: SystemMouseCursors.resizeRow,
          child: GestureDetector(
            onTap: controller.toggleSplitView,
            child: Container(
              height: 6,
              color: WordTheme.groupDivider,
              alignment: Alignment.center,
              child: Container(
                width: 40,
                height: 2,
                color: WordTheme.ribbonTextDisabled,
              ),
            ),
          ),
        ),
        Expanded(
          child: _buildCanvas(
            context,
            controller,
            scrollController: _splitScrollController,
            trackVisiblePage: false,
            forceReadOnly: true,
          ),
        ),
      ],
    );
  }

  Widget _buildCanvas(
    BuildContext context,
    EditorController controller, {
    required ScrollController scrollController,
    required bool trackVisiblePage,
    bool forceReadOnly = false,
  }) {
    final columns = controller.pageColumns.clamp(1, 3);
    final gap = _pageGapEffective;
    return Stack(
      children: [
        ListView.builder(
          key: trackVisiblePage
              ? const ValueKey('document-page-list')
              : const ValueKey('document-page-list-split'),
          controller: scrollController,
          padding: EdgeInsets.symmetric(vertical: gap),
          itemCount: _rowCount,
          itemExtent: _pageExtent,
          itemBuilder: (context, rowIndex) {
            final children = <Widget>[];
            for (var col = 0; col < columns; col++) {
              final index = rowIndex * columns + col;
              if (index >= controller.pageCount) break;
              _scheduleLoad(index);
              final readOnly =
                  forceReadOnly || !controller.isPageEditable(index);
              children.add(
                Padding(
                  padding: EdgeInsets.symmetric(horizontal: gap / 2),
                  child: Transform.scale(
                    scale: controller.zoom,
                    alignment: Alignment.topCenter,
                    child: ClipRect(
                      clipBehavior: Clip.hardEdge,
                      child: SizedBox(
                        width: controller.pageWidth,
                        height: controller.pageHeight,
                        child: GestureDetector(
                          onTap: readOnly
                              ? () => controller.setCurrentPage(index)
                              : null,
                          child: _PageCanvas(
                            key: ValueKey(
                              'page-$index-${controller.pageDisplayVersion(index)}',
                            ),
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
                ),
              );
            }
            return LayoutBuilder(
              builder: (context, constraints) {
                return Center(
                  child: ConstrainedBox(
                    constraints: BoxConstraints(maxWidth: constraints.maxWidth),
                    child: FittedBox(
                      fit: BoxFit.scaleDown,
                      alignment: Alignment.topCenter,
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: children,
                      ),
                    ),
                  ),
                );
              },
            );
          },
        ),
        if (kIsWeb) WebGlyphTextInput(controller: controller),
      ],
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

  bool get _webChrome =>
      controller.isWebLayout || controller.isReadMode;

  @override
  Widget build(BuildContext context) {
    return ClipRect(
      clipBehavior: Clip.hardEdge,
      child: Container(
        width: controller.pageWidth,
        height: controller.pageHeight,
        decoration: BoxDecoration(
          color: Colors.white,
          boxShadow: _webChrome
              ? null
              : [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.15),
                    blurRadius: 8,
                    offset: const Offset(0, 2),
                  ),
                ],
          border: controller.printPreview
              ? Border.all(color: const Color(0xFFB4B4B4), width: 1)
              : _webChrome
                  ? Border(
                      bottom: BorderSide(color: Colors.grey.shade200, width: 1),
                    )
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
