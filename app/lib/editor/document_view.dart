import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/editor/controllers/view_controller.dart';
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

/// Scroll offset that reveals [caretTop, caretBottom], or null if already visible.
double? scrollOffsetToRevealCaret({
  required double caretTop,
  required double caretBottom,
  required double viewportTop,
  required double viewportHeight,
  required double minExtent,
  required double maxExtent,
  double margin = 16,
}) {
  if (viewportHeight <= 0) return null;
  final viewBottom = viewportTop + viewportHeight;
  double? target;
  if (caretBottom > viewBottom - margin) {
    target = caretBottom + margin - viewportHeight;
  } else if (caretTop < viewportTop + margin) {
    target = caretTop - margin;
  }
  if (target == null) return null;
  return target.clamp(minExtent, maxExtent);
}

class _DocumentViewState extends State<DocumentView> {
  /// Per-page display lists. Every page of a layout version shares one atlas,
  /// so the decoded texture is kept separately and reused across pages.
  final Map<int, DisplayListSnapshot> _snapshots = {};
  final Map<int, int> _loadedPageVersions = {};
  final Set<int> _pending = {};
  ui.Image? _atlasImage;
  Future<void>? _atlasBuildFuture;
  int _loadedVersion = -1;
  int _loadedAtlasGeneration = -1;
  final Map<int, int> _emptyLoadRetries = {};
  final ScrollController _scrollController = ScrollController();
  final ScrollController _splitScrollController = ScrollController();
  double _viewportWidth = 0;

  bool _phoneNavSheetOpen = false;
  bool _phoneStyleSheetOpen = false;
  bool _phoneAccessibilitySheetOpen = false;
  bool _phonePictureSheetOpen = false;
  String? _dismissedPictureId;

  /// On phone/tablet, 100% zoom means the page fills the canvas width so the
  /// full line of text is on-screen. User zoom then scales that fit; going
  /// above 100% can pan horizontally. Desktop keeps 1:1 page points.
  double _effectiveScale(EditorController controller) {
    var scale = controller.zoom;
    if (mounted && WordTheme.mobileChrome(context)) {
      scale *= _mobileWidthFit(controller);
    }
    return scale;
  }

  double _mobileWidthFit(EditorController controller) {
    final columns = controller.pageColumns.clamp(1, 3);
    final rowWidth = columns * (controller.pageWidth + _pageGapEffective);
    if (_viewportWidth <= 1 || rowWidth <= 0) return 1.0;
    final fit = _viewportWidth / rowWidth;
    // Never upscale past 1:1 page points — only shrink so the page fits.
    return fit < 1.0 ? fit : 1.0;
  }

  double _pageExtentFor(EditorController controller) {
    final gap = _pageGapEffective;
    return (controller.pageHeight + gap) * _effectiveScale(controller);
  }

  double get _pageExtent => _pageExtentFor(widget.controller);

  /// Decoded document images shared across pages, keyed by asset id.
  final Map<String, ui.Image> _images = {};

  bool _pendingScrollArmed = false;
  /// True only while a caret-follow jump is driving the scroll position.
  bool _caretFollowScrollInFlight = false;

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
      _emptyLoadRetries.clear();
    }
    if (widget.controller.atlasGeneration != _loadedAtlasGeneration) {
      // Dispose the stale texture but do NOT mark the new generation as loaded
      // until pixels are decoded — otherwise page loads paint with a null atlas
      // (blank text until the next keystroke refresh).
      _atlasImage?.dispose();
      _atlasImage = null;
      _atlasBuildFuture = null;
      // Start decode immediately. Chrome often pauses rAF after the file-picker
      // dialog, so a post-frame-only kick would wait for an unrelated keypress.
      unawaited(_ensureAtlasTexture(widget.controller.displayVersion));
    }
    _armPendingScroll();
    if (mounted) setState(() {});
  }

  void _armPendingScroll() {
    if (_pendingScrollArmed) return;
    _pendingScrollArmed = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _pendingScrollArmed = false;
      if (!mounted) return;
      final caretScroll = widget.controller.view.takeCaretScrollRequest();
      if (caretScroll != null) {
        _scrollToKeepCaretVisible(caretScroll);
        return;
      }
      final scrollPage = widget.controller.view.takeScrollRequest();
      if (scrollPage != null) _scrollToPage(scrollPage);
    });
  }

  double get _pageGapEffective {
    final layout = widget.controller.viewLayout;
    if (layout == DocumentViewLayout.webLayout) return 4.0;
    if (layout == DocumentViewLayout.readMode) return 16.0;
    return _pageGap;
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
    final visiblePage = row * columns;
    // A caret-follow jump may leave a sliver of the previous page in view;
    // don't let that reset currentPage away from the caret's page. Manual
    // scrolling must still update it.
    if (_caretFollowScrollInFlight &&
        widget.controller.caretRunId != null &&
        visiblePage != widget.controller.caretPage) {
      return;
    }
    widget.controller.setVisiblePage(visiblePage);
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
      // Open can race the first engine read — retry a few times before caching
      // an empty snapshot (which would stick until the next version bump).
      final retries = _emptyLoadRetries[index] ?? 0;
      _pending.remove(index);
      if (retries < 5) {
        _emptyLoadRetries[index] = retries + 1;
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted) _scheduleLoad(index);
        });
        WidgetsBinding.instance.ensureVisualUpdate();
        WidgetsBinding.instance.scheduleFrame();
        return;
      }
      _emptyLoadRetries.remove(index);
      setState(() {
        _snapshots[index] = DisplayListSnapshot.empty();
        _loadedPageVersions[index] = pageVersion;
      });
      return;
    }
    _emptyLoadRetries.remove(index);

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
    if (controller.atlasPixels.isEmpty ||
        generation == 0 ||
        controller.atlasWidth == 0 ||
        controller.atlasHeight == 0) {
      return;
    }

    final inFlight = _atlasBuildFuture;
    if (inFlight != null) {
      await inFlight;
      return;
    }

    final build = _buildAtlasTexture(layoutVersion, generation);
    _atlasBuildFuture = build;
    try {
      await build;
    } finally {
      if (identical(_atlasBuildFuture, build)) {
        _atlasBuildFuture = null;
      }
    }
  }

  Future<void> _buildAtlasTexture(int layoutVersion, int generation) async {
    final controller = widget.controller;
    ui.Image? atlas;
    try {
      atlas = await DisplayListSnapshot.buildAtlasImageFromPixels(
        controller.atlasPixels,
        controller.atlasWidth,
        controller.atlasHeight,
      );
    } catch (e, st) {
      debugPrint('DocumentView: atlas decode failed: $e\n$st');
      return;
    }

    if (!mounted || layoutVersion != controller.displayVersion) {
      atlas?.dispose();
      return;
    }
    // A newer atlas generation arrived while we were decoding — drop this one.
    if (generation != controller.atlasGeneration) {
      atlas?.dispose();
      return;
    }

    setState(() {
      _atlasImage?.dispose();
      _atlasImage = atlas;
      _loadedAtlasGeneration = generation;
    });
    WidgetsBinding.instance.ensureVisualUpdate();
    WidgetsBinding.instance.scheduleFrame();
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
      if (mounted) unawaited(_loadPage(index));
    });
    // Wake Chrome after file-picker / dialog gaps where rAF is suspended.
    WidgetsBinding.instance.ensureVisualUpdate();
    WidgetsBinding.instance.scheduleFrame();
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

  double get _listPaddingTop {
    final gap = _pageGapEffective;
    if (WordTheme.mobileChrome(context)) {
      return gap * _effectiveScale(widget.controller);
    }
    return gap;
  }

  /// Maps a page-space caret Y into ListView scroll space.
  double get _caretYScale {
    final controller = widget.controller;
    if (WordTheme.mobileChrome(context)) {
      return _effectiveScale(controller);
    }
    final zoom = controller.zoom;
    final columns = controller.pageColumns.clamp(1, 3);
    final gap = _pageGapEffective;
    final rowWidth = columns * (controller.pageWidth + gap);
    final fit = _viewportWidth > 0 && rowWidth > _viewportWidth
        ? _viewportWidth / rowWidth
        : 1.0;
    return zoom * fit;
  }

  double _pageTopInScrollSpace(int page) {
    final columns = widget.controller.pageColumns.clamp(1, 3);
    final row = page.clamp(0, widget.controller.pageCount - 1) ~/ columns;
    return _listPaddingTop + row * _pageExtent;
  }

  void _scrollToKeepCaretVisible(CaretScrollRequest request) {
    if (!_scrollController.hasClients) return;
    final scale = _caretYScale;
    final pageTop = _pageTopInScrollSpace(request.page);
    final caretTop = pageTop + (request.y - request.height) * scale;
    final caretBottom = pageTop + request.y * scale;
    final position = _scrollController.position;
    final target = scrollOffsetToRevealCaret(
      caretTop: caretTop,
      caretBottom: caretBottom,
      viewportTop: position.pixels,
      viewportHeight: position.viewportDimension,
      minExtent: position.minScrollExtent,
      maxExtent: position.maxScrollExtent,
    );
    if (target == null) return;
    if ((target - position.pixels).abs() < 1) return;
    // Jump: key-repeat / IME bursts must not restart a 120ms animation.
    _caretFollowScrollInFlight = true;
    try {
      _scrollController.jumpTo(target);
    } finally {
      _caretFollowScrollInFlight = false;
    }
  }

  @override
  Widget build(BuildContext context) {
    final controller = widget.controller;
    final phone = WordTheme.phoneChrome(context);
    if (phone) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _syncPhoneSidePanes(context);
      });
    }
    return Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (!phone && controller.showNavigationPane && !controller.isReadMode)
          NavigationPane(
            controller: controller,
            currentPage: controller.currentPage,
            onPageSelected: controller.jumpToPage,
            onOutlineSelected: controller.jumpToOutlineEntry,
          ),
        Expanded(
          child: LayoutBuilder(
            builder: (context, constraints) {
              _viewportWidth = constraints.maxWidth;
              controller.reportViewportSize(
                Size(constraints.maxWidth, constraints.maxHeight),
              );
              final canvasBg = controller.isWebLayout || controller.isReadMode
                  ? Colors.white
                  : WordTheme.chrome(context).canvas;
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
        if (!phone && controller.hasSelectedImage && !controller.isReadMode)
          PictureInspectorPane(controller: controller),
        if (!phone && controller.showAccessibilityChecker && !controller.isReadMode)
          AccessibilityCheckerPane(controller: controller),
        if (!phone && controller.showStyleInspector && !controller.isReadMode)
          StyleInspectorPane(controller: controller),
      ],
    );
  }

  Future<void> _syncPhoneSidePanes(BuildContext context) async {
    final controller = widget.controller;
    if (!WordTheme.phoneChrome(context) || controller.isReadMode) return;

    if (controller.showNavigationPane && !_phoneNavSheetOpen) {
      _phoneNavSheetOpen = true;
      await _showPhonePaneSheet(
        context,
        title: 'Navigation',
        onDismiss: () {
          if (controller.showNavigationPane) {
            controller.toggleNavigationPane();
          }
        },
        child: NavigationPane(
          controller: controller,
          currentPage: controller.currentPage,
          onPageSelected: controller.jumpToPage,
          onOutlineSelected: controller.jumpToOutlineEntry,
          expanded: true,
        ),
      );
      _phoneNavSheetOpen = false;
    }

    if (controller.showStyleInspector && !_phoneStyleSheetOpen) {
      _phoneStyleSheetOpen = true;
      await _showPhonePaneSheet(
        context,
        title: 'Style Inspector',
        onDismiss: () {
          if (controller.showStyleInspector) {
            controller.toggleStyleInspector();
          }
        },
        child: StyleInspectorPane(controller: controller, expanded: true),
      );
      _phoneStyleSheetOpen = false;
    }

    if (controller.showAccessibilityChecker && !_phoneAccessibilitySheetOpen) {
      _phoneAccessibilitySheetOpen = true;
      await _showPhonePaneSheet(
        context,
        title: 'Accessibility',
        onDismiss: controller.hideAccessibilityChecker,
        child: AccessibilityCheckerPane(controller: controller, expanded: true),
      );
      _phoneAccessibilitySheetOpen = false;
    }

    if (!controller.hasSelectedImage) {
      _dismissedPictureId = null;
    } else if (controller.hasSelectedImage &&
        !_phonePictureSheetOpen &&
        controller.selectedImageId != _dismissedPictureId) {
      _phonePictureSheetOpen = true;
      final imageId = controller.selectedImageId;
      await _showPhonePaneSheet(
        context,
        title: 'Picture',
        onDismiss: () => _dismissedPictureId = imageId,
        child: PictureInspectorPane(controller: controller, expanded: true),
      );
      _phonePictureSheetOpen = false;
    }
  }

  Future<void> _showPhonePaneSheet(
    BuildContext context, {
    required String title,
    required VoidCallback onDismiss,
    required Widget child,
  }) {
    return showModalBottomSheet<void>(
      context: context,
      isScrollControlled: true,
      showDragHandle: true,
      builder: (sheetContext) {
        final height = MediaQuery.sizeOf(sheetContext).height * 0.55;
        return SafeArea(
          child: SizedBox(
            height: height,
            width: double.infinity,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 0, 8, 8),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          title,
                          style: const TextStyle(
                            fontSize: 14,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                      IconButton(
                        icon: const Icon(Icons.close, size: 20),
                        onPressed: () => Navigator.pop(sheetContext),
                        tooltip: 'Close',
                      ),
                    ],
                  ),
                ),
                const Divider(height: 1),
                Expanded(child: child),
              ],
            ),
          ),
        );
      },
    ).whenComplete(onDismiss);
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
              color: WordTheme.chrome(context).groupDivider,
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
    // Phone/tablet: page-width fit at 100% zoom (no sideways pan). Desktop/web
    // keeps Transform.scale + FittedBox.scaleDown.
    final mobile = WordTheme.mobileChrome(context);
    return mobile
        ? _buildPhoneCanvas(
            context,
            controller,
            scrollController: scrollController,
            trackVisiblePage: trackVisiblePage,
            forceReadOnly: forceReadOnly,
          )
        : _buildDesktopCanvas(
            context,
            controller,
            scrollController: scrollController,
            trackVisiblePage: trackVisiblePage,
            forceReadOnly: forceReadOnly,
          );
  }

  /// Desktop / web / tablet: scale-down to width and center the page row.
  Widget _buildDesktopCanvas(
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
          itemExtent: _pageExtentFor(controller),
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
        if (EditorController.usesSoftKeyboardGlyphInput)
          WebGlyphTextInput(controller: controller),
      ],
    );
  }

  /// Phone/tablet: layout size matches painted zoom. At 100% the page fits
  /// the canvas width; pinch/zoom above that can pan horizontally.
  Widget _buildPhoneCanvas(
    BuildContext context,
    EditorController controller, {
    required ScrollController scrollController,
    required bool trackVisiblePage,
    bool forceReadOnly = false,
  }) {
    final columns = controller.pageColumns.clamp(1, 3);
    final gap = _pageGapEffective;
    final scale = _effectiveScale(controller);
    final scaledPageW = controller.pageWidth * scale;
    final scaledPageH = controller.pageHeight * scale;
    final hPad = (gap * scale) / 2;
    final vPad = gap * scale;
    final contentWidth = columns * (scaledPageW + gap * scale);
    final pageList = ListView.builder(
      key: trackVisiblePage
          ? const ValueKey('document-page-list')
          : const ValueKey('document-page-list-split'),
      controller: scrollController,
      padding: EdgeInsets.symmetric(vertical: vPad),
      itemCount: _rowCount,
      itemExtent: _pageExtentFor(controller),
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
              padding: EdgeInsets.symmetric(horizontal: hPad),
              child: SizedBox(
                width: scaledPageW,
                height: scaledPageH,
                child: FittedBox(
                  fit: BoxFit.fill,
                  alignment: Alignment.topCenter,
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
        return Align(
          alignment: Alignment.topCenter,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: children,
          ),
        );
      },
    );

    return Stack(
      children: [
        LayoutBuilder(
          builder: (context, constraints) {
            if (contentWidth <= constraints.maxWidth + 0.5) {
              return pageList;
            }
            return SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: SizedBox(
                width: contentWidth,
                height: constraints.maxHeight,
                child: pageList,
              ),
            );
          },
        ),
        if (EditorController.usesSoftKeyboardGlyphInput)
          WebGlyphTextInput(controller: controller),
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
