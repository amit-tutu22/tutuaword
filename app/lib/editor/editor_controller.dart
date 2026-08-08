import 'dart:async';
import 'dart:convert';
import 'dart:io' if (dart.library.html) 'package:tutuaword/bridge/platform_stub.dart';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_loader.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/outline_entry.dart';
import 'package:tutuaword/editor/controllers/document_session_controller.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/controllers/formatting_controller.dart';
import 'package:tutuaword/editor/controllers/selection_controller.dart';
import 'package:tutuaword/editor/controllers/view_controller.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/editor/document_edit_zone.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/image_hit_test.dart';
import 'package:tutuaword/editor/shape_hit_test.dart';
import 'package:tutuaword/ui/paragraph_borders_dialog.dart';
import 'package:tutuaword/ui/paragraph_spacing_dialog.dart';
import 'package:tutuaword/ui/page_setup.dart';
import 'package:tutuaword/ui/table_design_dialog.dart';
import 'package:tutuaword/ui/paste_special_dialog.dart';
import 'package:tutuaword/ui/tab_stops_dialog.dart';

export 'package:tutuaword/bridge/engine_types.dart'
    show CaretGeometry, GlyphSelectionRect;
export 'package:tutuaword/editor/controllers/formatting_controller.dart'
    show LineSpacingMode;
export 'package:tutuaword/editor/doc_range.dart';

/// Clipboard payload read from the system pasteboard.
class EditorClipboardPayload {
  const EditorClipboardPayload({this.plainText, this.html, this.docxBytes});

  final String? plainText;
  final String? html;
  final Uint8List? docxBytes;

  bool get hasFormattedContent =>
      (html != null && html!.trim().isNotEmpty) ||
      (docxBytes != null && docxBytes!.isNotEmpty);
}

/// Thin composition of sub-controllers bridging Flutter UI to the Rust engine.
class EditorController extends ChangeNotifier {
  /// Test/document injection constructor — prefer [forTest] in unit tests.
  EditorController({
    DocumentEngine? engine,
    DocumentSessionStore? sessionStore,
    bool enableAutosave = true,
    Duration? autosaveInterval,
    bool useMockWhenEngineMissing = false,
  }) : _host = EngineHost(engine: engine ?? (useMockWhenEngineMissing ? MockDocumentEngine() : loadDocumentEngine())) {
    _view = ViewController();
    _selection = SelectionController(
      host: _host,
      onSelectionChanged: () => _formatting.syncFromCaret(),
    );
    _formatting = FormattingController(host: _host, selection: _selection);
    _session = DocumentSessionController(
      host: _host,
      selection: _selection,
      formatting: _formatting,
      view: _view,
      onSessionChanged: notifyListeners,
      sessionStore: sessionStore,
      enableAutosave: enableAutosave,
      autosaveInterval: autosaveInterval,
    );

    _bootStatus = _host.isConnected
        ? 'Rust engine connected'
        : 'Engine unavailable — build libtw_ffi';

    if (_host.isConnected) {
      _host.refreshFromEngine(full: true);
      _selection.ensureGlyphCaret();
      _formatting.syncFromCaret();
    }

    for (final sub in _subControllers) {
      sub.addListener(notifyListeners);
    }
  }

  /// [_host] is included so a coalesced display refresh repaints even when the
  /// edit that triggered it already notified optimistically.
  List<ChangeNotifier> get _subControllers =>
      [_host, _view, _selection, _formatting, _session];

  /// In-memory engine for widget/unit tests (R2.4).
  factory EditorController.forTest({MockDocumentEngine? engine}) {
    return EditorController(
      engine: engine ?? MockDocumentEngine(),
      enableAutosave: false,
    );
  }

  final EngineHost _host;
  late final ViewController _view;
  late final SelectionController _selection;
  late final FormattingController _formatting;
  late final DocumentSessionController _session;
  String _documentThemeName = 'Office';
  DocumentEditZone _editZone = DocumentEditZone.body;
  String? _selectedImageId;
  int? _selectedImagePage;
  Rect? _selectedImageRect;
  String? _selectedDiagramId;
  int? _selectedDiagramPage;
  Rect? _selectedDiagramRect;
  Rect? _previewImageRect;
  ImageResizeHandle? _activeImageHandle;
  Rect? _resizeStartRect;
  Offset? _moveStartPoint;
  Rect? _moveStartRect;
  double _selectedImageRotation = 0;

  String _bootStatus = '';

  // ── Sub-controller accessors (R2.4 decomposition) ─────────────────────────
  ViewController get view => _view;
  SelectionController get selectionController => _selection;
  FormattingController get formattingController => _formatting;
  DocumentSessionController get sessionController => _session;
  DocumentEditZone get editZone => _editZone;

  // ── Engine / rendering ────────────────────────────────────────────────────
  bool get isEngineConnected => _host.isConnected;
  bool get usesGlyphRendering => _host.isConnected;
  bool get preferTextRendering => false;
  Uint8List get displayListBytes => _host.displayListBytes;
  double get pageWidth => _host.pageWidth;
  double get pageHeight => _host.pageHeight;
  double get marginTop => _host.marginTop;
  double get marginBottom => _host.marginBottom;
  double get marginLeft => _host.marginLeft;
  double get marginRight => _host.marginRight;
  bool get isLandscape => pageWidth > pageHeight;
  String get marginPresetName =>
      PageSetupPresets.matchingMarginPreset(_currentSectionFormat()) ?? 'Normal';
  String get pageSizePresetName =>
      PageSetupPresets.matchingPageSizePreset(_currentSectionFormat()) ?? 'Letter';
  String get columnCountLabel =>
      PageSetupPresets.columnCountLabel(_currentSectionFormat());
  Color? get pageColor => PageSetupPresets.pageColor(_currentSectionFormat());
  String? get watermarkText => PageSetupPresets.watermarkText(_currentSectionFormat());
  bool get lineNumbersEnabled =>
      PageSetupPresets.lineNumbersEnabled(_currentSectionFormat());
  bool get differentFirstPage =>
      _currentSectionFormat()['different_first_page'] as bool? ?? false;
  bool get evenAndOddHeaders =>
      _host.engine?.fetchEvenAndOddHeadersEnabled() ?? false;
  bool get headerFooterLinked =>
      _host.engine?.fetchHeaderFooterLinked(
        caretRunId: _selection.defaultRunId(),
        isHeader: _editZone == DocumentEditZone.header,
        pageIndex: _selection.caretPage,
      ) ??
      false;
  bool get canLinkHeaderFooter => _editZone != DocumentEditZone.body;
  int get displayVersion => _host.displayVersion;
  int get atlasGeneration => _host.atlasGeneration;
  Uint8List get atlasPixels => _host.atlasPixels;
  int get atlasWidth => _host.atlasWidth;
  int get atlasHeight => _host.atlasHeight;
  int get pageCount => _host.pageCount;
  Uint8List displayListForPage(int page) => _host.displayListForPage(page);
  int pageDisplayVersion(int page) => _host.pageDisplayVersion(page);

  // ── Session ───────────────────────────────────────────────────────────────
  String get statusText {
    final base = _session.statusText.isNotEmpty ? _session.statusText : _bootStatus;
    final path = _session.currentPath == null
        ? ''
        : ' · ${_session.currentPath!.split(Platform.pathSeparator).last}';
    final preview = _view.printPreview ? ' · Print preview' : '';
    return '$base$path$preview · ${_host.documentText.length} chars · page ${_view.currentPage + 1}/$pageCount · v$displayVersion';
  }

  String get documentText => _host.documentText;
  String? get currentPath => _session.currentPath;
  bool get documentReadOnly => _session.documentReadOnly;
  DocumentProperties get documentProperties => _session.documentProperties;
  String? get infoMessage => _session.infoMessage;
  bool get trackChanges => _session.trackChanges;
  List<String> get spellMisspellings => _session.spellMisspellings;
  List<String> get recentDocuments => _session.recentDocuments;
  Duration get autosaveInterval => _session.autosaveInterval;
  String get documentTitle => _session.documentTitle;
  int get wordCount => _session.wordCount;

  @visibleForTesting
  int get nativeEditDepth => _host.nativeEditDepth;

  @visibleForTesting
  Future<void> ensureLayoutReady() => _host.ensureLayoutReady();

  // ── View ──────────────────────────────────────────────────────────────────
  int get currentPage => _view.currentPage;
  bool get printPreview => _view.printPreview;
  double get zoom => _view.zoom;
  bool get showRuler => _view.showRuler;
  bool get showNavigationPane => _view.showNavigationPane;
  bool get showStyleInspector => _view.showStyleInspector;
  String get styleInspectorSummary => _formatting.styleInspectorSummary;

  void setCurrentPage(int page) {
    _view.setCurrentPage(page, pageCount);
    _host.engine?.setCurrentPageIndex(_view.currentPage);
    _host.refreshFromEngine(dirtyPage: _view.currentPage);
    notifyListeners();
  }

  void setVisiblePage(int page) => _view.setVisiblePage(page, pageCount);
  void togglePrintPreview() {
    _view.togglePrintPreview();
    _session.setStatusText(_view.printPreview ? 'Print preview' : 'Print layout');
  }

  void setZoom(double value) => _view.setZoom(value);
  void zoomIn() => _view.zoomIn();
  void zoomOut() => _view.zoomOut();
  void toggleRuler() => _view.toggleRuler();
  void toggleNavigationPane() => _view.toggleNavigationPane();
  void toggleStyleInspector() => _view.toggleStyleInspector();

  List<DocumentOutlineEntry> get documentOutline {
    final json = _host.engine?.fetchDocumentOutline();
    if (json == null || json.isEmpty) return const [];
    final decoded = jsonDecode(json);
    if (decoded is! List) return const [];
    return decoded
        .whereType<Map>()
        .map((entry) => DocumentOutlineEntry.fromJson(Map<String, dynamic>.from(entry)))
        .toList();
  }

  void jumpToOutlineEntry(DocumentOutlineEntry entry) {
    _selection.setCaret(entry.runId, 0, page: entry.page);
    _view.setCurrentPage(entry.page, pageCount);
    _view.requestScrollToPage(entry.page);
    _host.engine?.setCurrentPageIndex(entry.page);
    notifyListeners();
  }

  bool isPageEditable(int pageIndex) {
    if (_session.documentReadOnly || _view.printPreview) return false;
    return _host.isConnected;
  }

  // ── Selection ─────────────────────────────────────────────────────────────
  CaretGeometry? get caretGeometry => _selection.caretGeometry;
  int get caretPage => _selection.caretPage;
  List<GlyphSelectionRect> get selectionRects => _selection.selectionRects;
  bool get hasGlyphSelection => _selection.hasGlyphSelection;
  DocRange? get selection => _selection.selection;
  String? get caretRunId => _selection.caretRunId;
  int get caretOffset => _selection.caretOffset;

  void ensureGlyphCaret() => _selection.ensureGlyphCaret();
  void hitTestAt(int pageIndex, double x, double y) => _selection.hitTestAt(pageIndex, x, y);
  void moveGlyphCaretByArrow(LogicalKeyboardKey key) => _selection.moveGlyphCaretByArrow(key);
  void beginGlyphSelection(int p, double x, double y) => _selection.beginGlyphSelection(p, x, y);
  void updateGlyphSelection(int p, double x, double y) => _selection.updateGlyphSelection(p, x, y);
  void endGlyphSelection(int p, double x, double y) => _selection.endGlyphSelection(p, x, y);
  bool isPointInGlyphSelection(int p, Offset pt) => _selection.isPointInGlyphSelection(p, pt);
  void beginGlyphDrag(int p) => _selection.beginGlyphDrag(p);
  void updateGlyphDragDropCaret(int p, double x, double y) =>
      _selection.updateGlyphDragDropCaret(p, x, y);
  void completeGlyphDrag(int p, double x, double y) => _completeGlyphDrag(p, x, y);
  void cancelGlyphDrag() => _selection.cancelGlyphDrag();
  bool get isGlyphDragActive => _selection.isGlyphDragActive;

  bool get hasSelectedImage => _selectedImageId != null;
  String? get selectedImageId => _selectedImageId;
  int? get selectedImagePage => _selectedImagePage;
  Rect? get selectedImageRect => _previewImageRect ?? _selectedImageRect;
  bool get hasSelectedDiagram => _selectedDiagramId != null;
  String? get selectedDiagramId => _selectedDiagramId;
  int? get selectedDiagramPage => _selectedDiagramPage;
  Rect? get selectedDiagramRect => _selectedDiagramRect;
  bool get isImageResizing => _activeImageHandle != null;

  void selectImage(int pageIndex, ImageBounds bounds) {
    clearDiagramSelection();
    _selectedImageId = bounds.imageId;
    _selectedImagePage = pageIndex;
    _selectedImageRect = bounds.rect;
    _previewImageRect = null;
    _selectedImageRotation = 0;
    notifyListeners();
  }

  void clearImageSelection() {
    if (_selectedImageId == null &&
        _selectedImageRect == null &&
        _previewImageRect == null) {
      return;
    }
    _selectedImageId = null;
    _selectedImagePage = null;
    _selectedImageRect = null;
    _previewImageRect = null;
    _selectedImageRotation = 0;
    _activeImageHandle = null;
    _resizeStartRect = null;
    notifyListeners();
  }

  bool trySelectImageAt(int pageIndex, Offset point, DisplayListSnapshot snapshot) {
    final hit = hitTestImage(snapshot, point);
    if (hit == null) {
      clearImageSelection();
      return false;
    }
    selectImage(pageIndex, hit);
    return true;
  }

  void selectDiagram(int pageIndex, ShapeBounds bounds) {
    clearImageSelection();
    _selectedDiagramId = bounds.shapeId;
    _selectedDiagramPage = pageIndex;
    _selectedDiagramRect = bounds.rect;
    notifyListeners();
  }

  void clearDiagramSelection() {
    if (_selectedDiagramId == null && _selectedDiagramRect == null) {
      return;
    }
    _selectedDiagramId = null;
    _selectedDiagramPage = null;
    _selectedDiagramRect = null;
    notifyListeners();
  }

  bool trySelectDiagramAt(int pageIndex, Offset point, DisplayListSnapshot snapshot) {
    final hit = hitTestShape(snapshot, point);
    if (hit == null) {
      clearDiagramSelection();
      return false;
    }
    selectDiagram(pageIndex, hit);
    return true;
  }

  ImageResizeHandle? imageHandleAt(Offset point) {
    final rect = selectedImageRect;
    if (rect == null) return null;
    return hitTestImageHandle(rect, point);
  }

  void beginImageResize(ImageResizeHandle handle) {
    final rect = _selectedImageRect;
    if (rect == null) return;
    _activeImageHandle = handle;
    _resizeStartRect = rect;
    _previewImageRect = rect;
    notifyListeners();
  }

  void updateImageResize(Offset current, {required bool lockAspectRatio}) {
    final start = _resizeStartRect;
    final handle = _activeImageHandle;
    if (start == null || handle == null) return;
    _previewImageRect = resizeImageWithHandle(
      start: start,
      handle: handle,
      current: current,
      origin: start.topLeft,
      lockAspectRatio: lockAspectRatio,
    );
    notifyListeners();
  }

  Future<void> commitImageResize() async {
    final id = _selectedImageId;
    final rect = _previewImageRect ?? _selectedImageRect;
    _activeImageHandle = null;
    _resizeStartRect = null;
    if (id == null || rect == null || !_host.isConnected) {
      _previewImageRect = null;
      notifyListeners();
      return;
    }
    await _session.applyEngineStyle(
      () => _host.engine!.setImageSizeAsync(id, rect.width, rect.height),
      'Image resized',
      full: true,
    );
    _selectedImageRect = rect;
    _previewImageRect = null;
    notifyListeners();
  }

  void cancelImageResize() {
    _activeImageHandle = null;
    _resizeStartRect = null;
    _previewImageRect = null;
    notifyListeners();
  }

  Future<void> replaceSelectedImage() async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    _session.setStatusText('Choose picture…');
    notifyListeners();
    try {
      final useInMemoryBytes =
          kIsWeb || (!kIsWeb && (Platform.isAndroid || Platform.isIOS));
      final result = await FilePicker.pickFiles(
        dialogTitle: 'Replace picture',
        type: FileType.custom,
        allowedExtensions: const ['png', 'jpg', 'jpeg', 'svg'],
        allowMultiple: false,
        withData: useInMemoryBytes,
      );
      if (result == null || result.files.isEmpty) {
        _session.setStatusText('Replace cancelled');
        return;
      }
      final file = result.files.single;
      Uint8List? bytes = file.bytes;
      if (bytes == null && file.path != null) {
        bytes = await File(file.path!).readAsBytes();
      }
      if (bytes == null || bytes.isEmpty) {
        _session.setStatusText('Replace failed: empty file');
        return;
      }
      final mime = _mimeForPicture(file.extension, file.name);
      await replaceSelectedImageBytes(bytes, mime);
    } catch (e) {
      _session.setStatusText('Replace failed: $e');
    }
  }

  Future<void> replaceSelectedImageBytes(Uint8List bytes, String mimeType) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.replaceImageBytesAsync(id, bytes, mimeType),
      'Picture replaced',
      full: true,
    );
    notifyListeners();
  }

  bool isPointOnSelectedImage(Offset point) {
    final rect = selectedImageRect;
    return rect != null && rect.contains(point);
  }

  void beginImageMove(Offset point) {
    final rect = _selectedImageRect;
    if (rect == null) return;
    _moveStartPoint = point;
    _moveStartRect = rect;
    _previewImageRect = rect;
    notifyListeners();
  }

  void updateImageMove(Offset current) {
    final start = _moveStartPoint;
    final origin = _moveStartRect;
    if (start == null || origin == null) return;
    _previewImageRect = origin.shift(current - start);
    notifyListeners();
  }

  Future<void> commitImageMove() async {
    final id = _selectedImageId;
    final rect = _previewImageRect ?? _selectedImageRect;
    _moveStartPoint = null;
    _moveStartRect = null;
    if (id == null || rect == null || !_host.isConnected) {
      _previewImageRect = null;
      notifyListeners();
      return;
    }
    await _session.applyEngineStyle(
      () => _host.engine!.setImageAnchorAsync(
        id,
        rect.left - marginLeft,
        rect.top - marginTop,
      ),
      'Image moved',
      full: true,
    );
    _selectedImageRect = rect;
    _previewImageRect = null;
    notifyListeners();
  }

  void cancelImageMove() {
    _moveStartPoint = null;
    _moveStartRect = null;
    _previewImageRect = null;
    notifyListeners();
  }

  Future<void> setSelectedImageWrap(int wrap) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    final label = switch (wrap) {
      0 => 'Inline with text',
      1 => 'Square wrap',
      3 => 'Behind text',
      _ => 'Wrap updated',
    };
    await _session.applyEngineStyle(
      () => _host.engine!.setImageWrapAsync(id, wrap),
      label,
      full: true,
    );
    notifyListeners();
  }

  Future<void> rotateSelectedImage({bool clockwise = true}) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    _selectedImageRotation =
        (_selectedImageRotation + (clockwise ? 90 : -90)) % 360;
    await _session.applyEngineStyle(
      () => _host.engine!.setImageTransformAsync(
        id,
        rotationDeg: _selectedImageRotation,
      ),
      'Picture rotated',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertSelectedImageCaption() async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertImageCaptionAsync(id),
      'Caption inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> compressSelectedImage({int quality = 75}) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.compressImageAsync(id, quality),
      'Picture compressed',
      full: true,
    );
    notifyListeners();
  }

  void selectGlyphWordAt(int p, double x, double y) => _selection.selectGlyphWordAt(p, x, y);
  Future<void> selectAll() => _selection.selectAll();

  // ── Formatting ────────────────────────────────────────────────────────────
  bool get bold => _formatting.bold;
  bool get italic => _formatting.italic;
  bool get underline => _formatting.underline;
  String get fontFamily => _formatting.fontFamily;
  double get fontSize => _formatting.fontSize;
  TextAlign get alignment => _formatting.alignment;
  bool get strikethrough => _formatting.strikethrough;
  bool get subscript => _formatting.subscript;
  bool get superscript => _formatting.superscript;
  bool get allCaps => _formatting.allCaps;
  bool get smallCaps => _formatting.smallCaps;
  bool get hidden => _formatting.hidden;
  bool get ligatures => _formatting.ligatures;
  Color get fontColor => _formatting.fontColor;
  Color? get highlightColor => _formatting.highlightColor;
  String get activeParagraphStyle => _formatting.activeParagraphStyle;
  String get documentThemeName => _documentThemeName;
  double get indentLeft => _formatting.indentLeft;
  LineSpacingMode get lineSpacing => _formatting.lineSpacing;
  double get exactLineSpacingPt => _formatting.exactLineSpacingPt;
  double get spaceBefore => _formatting.spaceBefore;
  double get spaceAfter => _formatting.spaceAfter;
  List<Map<String, dynamic>> get tabStops => _formatting.tabStops;

  void toggleBold() => _formatting.toggleBold();
  void toggleItalic() => _formatting.toggleItalic();
  void toggleUnderline() => _formatting.toggleUnderline();
  void setFontFamily(String f) => _formatting.setFontFamily(f);
  void setFontSize(double s) => _formatting.setFontSize(s);
  void increaseFontSize() => _formatting.increaseFontSize();
  void decreaseFontSize() => _formatting.decreaseFontSize();
  void setFontColor(Color c, {String? themeSlot, int? themeVariant}) =>
      _formatting.setFontColor(c, themeSlot: themeSlot, themeVariant: themeVariant);
  void clearFontColor() => _formatting.clearFontColor();
  void setHighlight(Color c) => _formatting.setHighlight(c);
  void clearHighlight() => _formatting.clearHighlight();
  void setAlignment(TextAlign a) => _formatting.setAlignment(a);
  void toggleStrikethrough() => _formatting.toggleStrikethrough();
  void toggleSubscript() => _formatting.toggleSubscript();
  void toggleSuperscript() => _formatting.toggleSuperscript();
  void toggleAllCaps() => _formatting.toggleAllCaps();
  void toggleSmallCaps() => _formatting.toggleSmallCaps();
  void toggleHidden() => _formatting.toggleHidden();
  void toggleLigatures() => _formatting.toggleLigatures();
  void clearFormatting() => unawaited(_formatting.clearFormatting());
  void increaseIndent() => _formatting.increaseIndent();
  void decreaseIndent() => _formatting.decreaseIndent();
  bool get isInList => _formatting.isInList;
  int get listLevel => _formatting.listLevel;
  void promoteListLevel() => _formatting.promoteListLevel();
  void demoteListLevel() => _formatting.demoteListLevel();
  void applySpacing({
    required LineSpacingMode lineSpacing,
    required double exactPoints,
    required double spaceBefore,
    required double spaceAfter,
    required bool keepTogether,
    required bool keepWithNext,
    required bool widowOrphanControl,
  }) =>
      _formatting.applySpacing(
        lineSpacing: lineSpacing,
        exactPoints: exactPoints,
        spaceBefore: spaceBefore,
        spaceAfter: spaceAfter,
        keepTogether: keepTogether,
        keepWithNext: keepWithNext,
        widowOrphanControl: widowOrphanControl,
      );

  bool get keepTogether => _formatting.keepTogether;
  bool get keepWithNext => _formatting.keepWithNext;
  bool get widowOrphanControl => _formatting.widowOrphanControl;
  Color? get paraShading => _formatting.paraShading;
  double get borderWidth => _formatting.borderWidth;

  void applyBordersAndShading({
    Color? shading,
    double borderWidth = 0,
    Color borderColor = Colors.black,
    bool clearShading = false,
    bool clearBorders = false,
  }) =>
      _formatting.applyBordersAndShading(
        shading: shading,
        borderWidth: borderWidth,
        borderColor: borderColor,
        clearShading: clearShading,
        clearBorders: clearBorders,
      );

  Future<void> showParagraphSpacingDialog(BuildContext context) async {
    final values = await ParagraphSpacingDialog.show(
      context,
      initial: ParagraphSpacingValues(
        lineSpacing: _formatting.lineSpacing,
        exactPoints: _formatting.exactLineSpacingPt,
        spaceBefore: _formatting.spaceBefore,
        spaceAfter: _formatting.spaceAfter,
        keepTogether: _formatting.keepTogether,
        keepWithNext: _formatting.keepWithNext,
        widowOrphanControl: _formatting.widowOrphanControl,
      ),
    );
    if (values == null) return;
    applySpacing(
      lineSpacing: values.lineSpacing,
      exactPoints: values.exactPoints,
      spaceBefore: values.spaceBefore,
      spaceAfter: values.spaceAfter,
      keepTogether: values.keepTogether,
      keepWithNext: values.keepWithNext,
      widowOrphanControl: values.widowOrphanControl,
    );
  }

  Future<void> showParagraphBordersDialog(BuildContext context) async {
    final values = await ParagraphBordersDialog.show(
      context,
      initial: ParagraphBordersValues(
        shading: _formatting.paraShading,
        borderWidth: _formatting.borderWidth,
      ),
    );
    if (values == null) return;
    applyBordersAndShading(
      shading: values.shading,
      borderWidth: values.borderWidth,
      borderColor: values.borderColor,
      clearShading: values.clearShading,
      clearBorders: values.clearBorders,
    );
  }

  void applyTabStops(List<Map<String, dynamic>> stops) =>
      _formatting.applyTabStops(stops);

  Future<void> showTabStopsDialog(BuildContext context) async {
    final initial = _formatting.tabStops
        .map(TabStopValue.fromJson)
        .whereType<TabStopValue>()
        .toList();
    final result = await TabStopsDialog.show(context, initial: initial);
    if (result == null) return;
    applyTabStops(result.map((s) => s.toJson()).toList());
  }

  // ── Clipboard ─────────────────────────────────────────────────────────────
  String get selectedText => _selection.selectedText();

  bool get canCutOrCopy => selectedText.isNotEmpty;

  Future<void> copySelection() async {
    final text = selectedText;
    if (text.isEmpty) return;
    await Clipboard.setData(ClipboardData(text: text));
  }

  Future<void> cutSelection() async {
    if (!canCutOrCopy) return;
    final text = selectedText;
    await Clipboard.setData(ClipboardData(text: text));
    await _selection.deleteGlyphSelection();
    _session.markDocumentDirty();
    notifyListeners();
  }

  static const _clipboardHtml = 'text/html';

  Future<EditorClipboardPayload> readClipboard() async {
    final plain = await Clipboard.getData(Clipboard.kTextPlain);
    final html = await Clipboard.getData(_clipboardHtml);
    return EditorClipboardPayload(plainText: plain?.text, html: html?.text);
  }

  Future<void> paste({bool plainText = false}) async {
    await pastePayload(await readClipboard(), plainText: plainText);
  }

  Future<void> showPasteSpecialDialog(BuildContext context) async {
    final payload = await readClipboard();
    final mode = await PasteSpecialDialog.show(
      context,
      hasFormattedContent: payload.hasFormattedContent,
    );
    if (mode == null) return;
    await pastePayload(payload, plainText: mode == PasteSpecialMode.plainText);
  }

  Future<void> pastePayload(EditorClipboardPayload payload, {required bool plainText}) async {
    if (!_host.isConnected) return;
    if (_selection.hasGlyphSelection) await _selection.deleteGlyphSelection();
    final runId = _selection.defaultRunId();
    if (runId == null) return;

    final pasted = !plainText && await _tryPasteFormatted(runId, payload) ||
        await _tryPastePlain(runId, payload.plainText);
    if (!pasted) return;

    _selection.collapseToCaret();
    _formatting.syncFromCaret();
    _session.markDocumentDirty();
    notifyListeners();
  }

  Future<void> deleteSelection() async {
    if (!canCutOrCopy) return;
    await _selection.deleteGlyphSelection();
    _session.markDocumentDirty();
    notifyListeners();
  }

  // ── Editing ───────────────────────────────────────────────────────────────
  void insertCharacter(String char) {
    if (_host.isConnected) {
      unawaited(insertGlyphCharacter(char));
      return;
    }
  }

  void deleteBackward() {
    if (_host.isConnected) unawaited(deleteGlyphBackward());
  }

  void deleteForward() {
    if (_host.isConnected) unawaited(deleteGlyphForward());
  }

  Future<void> insertGlyphCharacter(String char) async {
    if (_host.engine == null) return;
    if (char == '\n' || char == '\r') return;
    if (char != '\t' && char.codeUnitAt(0) < 0x20) return;
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final offset = _selection.caretOffset;
    final edit = _host.performNativeEdit(
      () => _host.engine!.tryInsertTextAsync(runId, offset, char),
      dirtyPage: _selection.caretPage,
    );
    // Logical caret advances before the worker acknowledges so the next
    // keystroke targets the right offset. Geometry must wait for the edit —
    // querying the engine earlier still sees the pre-insert layout.
    final optimistic = offset + char.length;
    _selection.afterInsert(runId, optimistic);
    _session.markDocumentDirty();
    notifyListeners();
    if (!await edit) {
      _rollbackCaret(runId, from: optimistic, to: offset);
    } else if (_selection.caretRunId == runId &&
        _selection.caretOffset == optimistic) {
      // Skip if a later keystroke (e.g. Enter) already moved the caret —
      // keyboard handlers fire inserts without awaiting them.
      _selection.syncCaretGeometry();
    }
    notifyListeners();
  }

  /// Undoes an optimistic caret advance, but only when nothing has moved the
  /// caret since — a later keystroke's position must win over an earlier
  /// failure's stale offset.
  void _rollbackCaret(String runId, {required int from, required int to}) {
    if (_selection.caretRunId != runId || _selection.caretOffset != from) return;
    _selection.afterInsert(runId, to);
    notifyListeners();
  }

  Future<void> insertGlyphParagraphBreak() async {
    if (_host.engine == null) return;
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    const margin = 72.0;
    final prevY = _selection.caretGeometry?.y ?? (margin + _formatting.fontSize);
    final edit = _host.performNativeEdit(
      () => _host.engine!.splitParagraphAsync(runId, _selection.caretOffset),
      dirtyPage: _selection.caretPage,
    );
    await edit;
    // Probe from the left margin on the line below the break. Using the
    // pre-break caret X lands on the trailing edge of the previous character
    // whenever hit-testing ignores Y (mocks) or the Y probe is still on the
    // same line. A split always leaves the caret at offset 0 of the new para.
    final nextY = prevY + _formatting.fontSize * 1.4;
    hitTestAt(_selection.caretPage, margin, nextY);
    final newRun = _selection.caretRunId ?? runId;
    _selection.afterInsert(newRun, 0);
    _selection.syncCaretGeometry();
    _session.markDocumentDirty();
    notifyListeners();
  }

  Future<void> deleteGlyphBackward() async {
    if (_host.engine == null) return;
    if (_selection.hasGlyphSelection) {
      await _selection.deleteGlyphSelection();
      _session.markDocumentDirty();
      return;
    }
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    if (_selection.caretOffset > 0) {
      final off = _selection.caretOffset;
      final edit = _host.performNativeEdit(
        () => _host.engine!.deleteRangeAsync(runId, off - 1, off),
        dirtyPage: _selection.caretPage,
      );
      final optimistic = off - 1;
      _selection.afterInsert(runId, optimistic);
      _session.markDocumentDirty();
      notifyListeners();
      if (!await edit) {
        _rollbackCaret(runId, from: optimistic, to: off);
      } else if (_selection.caretRunId == runId &&
          _selection.caretOffset == optimistic) {
        _selection.syncCaretGeometry();
      }
      notifyListeners();
      return;
    }
  }

  Future<void> deleteGlyphForward() async {
    if (_host.engine == null) return;
    if (_selection.hasGlyphSelection) {
      await _selection.deleteGlyphSelection();
      _session.markDocumentDirty();
      return;
    }
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final off = _selection.caretOffset;
    final edit = _host.performNativeEdit(
      () => _host.engine!.deleteRangeAsync(runId, off, off + 1),
      dirtyPage: _selection.caretPage,
    );
    _selection.collapseToCaret();
    notifyListeners();
    if (await edit) _session.markDocumentDirty();
  }

  Future<void> moveGlyphSelectionTo(int pageIndex, double x, double y) async {
    if (_host.engine == null || !_selection.hasGlyphSelection) return;
    final text = selectedText;
    if (text.isEmpty) return;
    await _selection.deleteGlyphSelection();
    final drop = _host.engine!.hitTestPage(pageIndex, x, y);
    if (drop == null) return;
    final edit = _host.performNativeEdit(
      () => _host.engine!.tryInsertTextAsync(drop.runId, drop.charOffset, text),
      dirtyPage: _selection.caretPage,
    );
    if (!await edit) return;
    _selection.afterInsert(drop.runId, drop.charOffset + text.length);
    _selection.syncCaretGeometry();
    _formatting.syncFromCaret();
    _session.markDocumentDirty();
    notifyListeners();
  }

  // ── Document lifecycle (delegated) ────────────────────────────────────────
  Future<void> newDocument() => _session.newDocument();
  Future<void> openDocument() => _session.openDocument();
  Future<void> openDocumentFromPath(String path) => _session.openDocumentFromPath(path);
  Future<void> openRecentDocument(String path) => _session.openRecentDocument(path);
  Future<void> saveDocument() => _session.saveDocument();
  Future<void> saveDocumentAs({required String extension}) =>
      _session.saveDocumentAs(extension: extension);
  Future<bool> saveDocumentToPath(String path, {String? formatExtension}) =>
      _session.saveDocumentToPath(path, formatExtension: formatExtension);
  Future<void> exportPdf() => _session.exportPdf();
  Future<bool> exportPdfToPath(String path) => _session.exportPdfToPath(path);
  Future<void> undo() => _session.undo();
  Future<void> redo() => _session.redo();
  Future<void> setAutosaveInterval(Duration d) => _session.setAutosaveInterval(d);
  Future<void> performAutosave() => _session.performAutosave();
  Future<bool> tryRecoverAutosave() => _session.tryRecoverAutosave();
  void toggleTrackChanges() => _session.toggleTrackChanges();
  void acceptAllRevisions() => _session.acceptAllRevisions();
  void rejectAllRevisions() => _session.rejectAllRevisions();
  Future<void> spellCheckDocument() => _session.spellCheckDocument();
  void clearInfoMessage() => _session.clearInfoMessage();

  void insertTable() => _session.applyEngineStyle(
        () => _host.engine!.insertTableBlockAsync(3, 3),
        'Table inserted (3×3)',
      );

  static const int shapeRectangle = 0;
  static const int shapeLine = 1;
  static const int shapeEllipse = 2;

  Future<void> insertShape(int shapeType) async {
    if (!_host.isConnected) return;
    final label = switch (shapeType) {
      shapeLine => 'Line inserted',
      shapeEllipse => 'Ellipse inserted',
      _ => 'Rectangle inserted',
    };
    await _session.applyEngineStyle(
      () => _host.engine!.insertShapeBlockAsync(shapeType),
      label,
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertTextBox() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertTextBoxAsync(),
      'Text box inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertWordArt(String text) async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertWordArtAsync(text),
      'WordArt inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertSmartArt() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertDiagramAsync(),
      'SmartArt inserted (read-only)',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertChart() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertChartAsync(),
      'Chart inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> deleteTableRow() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.deleteTableRowAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Table row deleted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> deleteTableColumn() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.deleteTableColumnAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Table column deleted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> mergeTableCells() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.mergeTableCellsAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Cells merged',
      full: true,
    );
    notifyListeners();
  }

  Future<void> splitTableCell() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.splitTableCellAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Cell split',
      full: true,
    );
    notifyListeners();
  }

  Future<void> showTableDesignDialog(BuildContext context) async {
    final values = await TableDesignDialog.show(
      context,
      initial: const TableDesignValues(columnWidth: 100),
    );
    if (values == null) return;
    await applyTableDesign(values);
  }

  Future<void> applyTableDesign(TableDesignValues values) async {
    if (!_host.isConnected) return;
    final caret = _selection.defaultRunId();

    if (values.autofitToWindow) {
      await _session.applyEngineStyle(
        () => _host.engine!.autofitTableAsync(caretRunId: caret),
        'Table autofit to window',
        full: true,
      );
      notifyListeners();
      return;
    }

    await _session.applyEngineStyle(
      () async {
        if (values.clearTableBorder) {
          if (!await _host.engine!.setTableBorderAsync(
            caretRunId: caret,
            width: 0,
            color: Colors.black,
          )) {
            return false;
          }
        } else if (values.tableBorderWidth > 0) {
          if (!await _host.engine!.setTableBorderAsync(
            caretRunId: caret,
            width: values.tableBorderWidth,
            color: values.tableBorderColor,
          )) {
            return false;
          }
        }

        if (values.clearCellShading) {
          if (!await _host.engine!.setTableCellShadingAsync(caretRunId: caret)) {
            return false;
          }
        } else if (values.cellShading != null) {
          if (!await _host.engine!.setTableCellShadingAsync(
            caretRunId: caret,
            shading: values.cellShading,
          )) {
            return false;
          }
        }

        if (values.columnWidth != null && values.columnWidth! > 0) {
          return _host.engine!.resizeTableColumnAsync(
            caretRunId: caret,
            width: values.columnWidth!,
          );
        }
        return true;
      },
      'Table design applied',
      full: true,
    );
    notifyListeners();
  }

  Future<void> sortTableAscending() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.sortTableRowsAsync(
        caretRunId: _selection.defaultRunId(),
        ascending: true,
      ),
      'Table sorted A→Z',
      full: true,
    );
    notifyListeners();
  }

  Future<void> sortTableDescending() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.sortTableRowsAsync(
        caretRunId: _selection.defaultRunId(),
        ascending: false,
      ),
      'Table sorted Z→A',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertNestedTable() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertNestedTableAsync(
        caretRunId: _selection.defaultRunId(),
        rows: 2,
        cols: 2,
      ),
      'Nested table inserted (2×2)',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertTableSumFormula() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertTableSumFieldAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'SUM formula inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertImageBytes(Uint8List bytes, String mimeType) async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertImageBytesAsync(bytes, mimeType),
      'Picture inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertImage() async {
    if (!_host.isConnected) return;
    _session.setStatusText('Choose picture…');
    notifyListeners();
    try {
      final useInMemoryBytes =
          kIsWeb || (!kIsWeb && (Platform.isAndroid || Platform.isIOS));
      final result = await FilePicker.pickFiles(
        dialogTitle: 'Insert picture',
        type: FileType.custom,
        allowedExtensions: const ['png', 'jpg', 'jpeg', 'svg'],
        allowMultiple: false,
        withData: useInMemoryBytes,
      );
      if (result == null || result.files.isEmpty) {
        _session.setStatusText('Insert cancelled');
        return;
      }
      final file = result.files.single;
      Uint8List? bytes = file.bytes;
      if (bytes == null && file.path != null) {
        bytes = await File(file.path!).readAsBytes();
      }
      if (bytes == null || bytes.isEmpty) {
        _session.setStatusText('Insert failed: empty file');
        return;
      }
      final mime = _mimeForPicture(file.extension, file.name);
      await insertImageBytes(bytes, mime);
    } catch (e) {
      _session.setStatusText('Insert failed: $e');
    }
  }

  String _mimeForPicture(String? extension, String name) {
    final ext = (extension ?? name.split('.').last).toLowerCase();
    return switch (ext) {
      'png' => 'image/png',
      'jpg' || 'jpeg' => 'image/jpeg',
      'svg' => 'image/svg+xml',
      _ => 'application/octet-stream',
    };
  }

  void applyHeading1() => applyParagraphStyle('Heading 1');

  void applyNormalStyle() => applyParagraphStyle('Normal');

  void applyParagraphStyle(String styleName) => _session.applyEngineStyle(
        () => _host.engine!.applyParagraphStyleAsync(
          styleName: styleName,
          caretRunId: _selection.defaultRunId(),
        ),
        '$styleName applied',
      ).then((_) => _formatting.setActiveParagraphStyle(styleName));

  void applyDocumentTheme(String themeName) => _session.applyEngineStyle(
        () => _host.engine!.applyDocumentThemeAsync(themeName: themeName),
        '$themeName theme applied',
      ).then((_) {
        _documentThemeName = themeName;
        notifyListeners();
      });

  Map<String, dynamic> _currentSectionFormat() {
    final json = _host.engine?.fetchSectionFormat(
      caretRunId: _selection.defaultRunId(),
    );
    if (json != null && json.isNotEmpty) {
      try {
        return jsonDecode(json) as Map<String, dynamic>;
      } catch (_) {}
    }
    return PageSetupPresets.defaultSectionFormat();
  }

  Future<void> _applySectionFormat(Map<String, dynamic> next, String status) =>
      _session.applyEngineStyle(
        () => _host.engine!.applySectionFormatJsonAsync(
          formatJson: jsonEncode(next),
          caretRunId: _selection.defaultRunId(),
        ),
        status,
        full: true,
      ).then((_) => notifyListeners());

  void applyMarginPreset(String name) => _applySectionFormat(
        PageSetupPresets.withMarginPreset(_currentSectionFormat(), name),
        '$name margins applied',
      );

  void setOrientation({required bool landscape}) => _applySectionFormat(
        PageSetupPresets.withOrientation(
          _currentSectionFormat(),
          landscape: landscape,
        ),
        landscape ? 'Landscape orientation applied' : 'Portrait orientation applied',
      );

  void applyPageSizePreset(String name) => _applySectionFormat(
        PageSetupPresets.withPageSizePreset(_currentSectionFormat(), name),
        '$name page size applied',
      );

  void applyColumnCount(int count) => _applySectionFormat(
        PageSetupPresets.withColumnCount(_currentSectionFormat(), count),
        switch (count.clamp(1, 3)) {
          2 => 'Two columns applied',
          3 => 'Three columns applied',
          _ => 'One column applied',
        },
      );

  void applyPageColor(Color? color) => _applySectionFormat(
        PageSetupPresets.withPageColor(_currentSectionFormat(), color),
        color == null ? 'Page color cleared' : 'Page color applied',
      );

  void applyWatermark(String text) => _applySectionFormat(
        PageSetupPresets.withWatermark(_currentSectionFormat(), text),
        'Watermark applied',
      );

  void clearWatermark() => _applySectionFormat(
        PageSetupPresets.withoutWatermark(_currentSectionFormat()),
        'Watermark removed',
      );

  void setLineNumbersEnabled(bool enabled) => _applySectionFormat(
        PageSetupPresets.withLineNumbers(
          _currentSectionFormat(),
          enabled: enabled,
        ),
        enabled ? 'Line numbers enabled' : 'Line numbers disabled',
      );

  void setDifferentFirstPage(bool enabled) => _applySectionFormat(
        PageSetupPresets.withDifferentFirstPage(_currentSectionFormat(), enabled),
        enabled ? 'Different first page on' : 'Different first page off',
      );

  Future<void> setEvenAndOddHeaders(bool enabled) async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.setEvenAndOddHeadersAsync(enabled: enabled),
      enabled ? 'Odd & even headers on' : 'Odd & even headers off',
      full: true,
    );
    notifyListeners();
  }

  Future<void> setHeaderFooterLinked(bool linked) async {
    if (!_host.isConnected || _editZone == DocumentEditZone.body) return;
    await _session.applyEngineStyle(
      () => _host.engine!.setHeaderFooterLinkAsync(
        caretRunId: _selection.defaultRunId(),
        isHeader: _editZone == DocumentEditZone.header,
        linked: linked,
        pageIndex: _selection.caretPage,
      ),
      linked ? 'Linked to previous' : 'Unlinked from previous',
      full: true,
    );
    if (!linked) {
      await _activateHeaderFooterEdit(
        isHeader: _editZone == DocumentEditZone.header,
      );
    }
    notifyListeners();
  }

  void applyNumberedList() => _session.applyEngineStyle(
        () => _host.engine!.applyNumberedListStyleAsync(caretRunId: _selection.defaultRunId()),
        'Numbered list applied',
      );

  void applyBulletList() => _session.applyEngineStyle(
        () => _host.engine!.applyBulletListStyleAsync(caretRunId: _selection.defaultRunId()),
        'Bullet list applied',
      );

  void restartNumbering() => _session.applyEngineStyle(
        () => _host.engine!.restartNumberingAsync(caretRunId: _selection.defaultRunId()),
        'Numbering restarted',
      );

  void continueNumbering() => _session.applyEngineStyle(
        () => _host.engine!.continueNumberingAsync(caretRunId: _selection.defaultRunId()),
        'Numbering continued',
      );

  void insertPageBreak() => _session.applyEngineStyle(
        () => _host.engine!.insertPageBreakAtAsync(caretRunId: _selection.defaultRunId()),
        'Page break inserted',
      );

  void insertSectionBreak() => _session.applyEngineStyle(
        () => _host.engine!.insertSectionBreakAtAsync(caretRunId: _selection.defaultRunId()),
        'Section break inserted',
        full: true,
      );

  Future<void> openHeaderEdit() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.ensureHeaderFooterAsync(
        isHeader: true,
        caretRunId: _selection.defaultRunId(),
        pageIndex: _selection.caretPage,
      ),
      'Editing header',
      full: true,
    );
    await _activateHeaderFooterEdit(isHeader: true);
  }

  Future<void> openFooterEdit() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.ensureHeaderFooterAsync(
        isHeader: false,
        caretRunId: _selection.defaultRunId(),
        pageIndex: _selection.caretPage,
      ),
      'Editing footer',
      full: true,
    );
    await _activateHeaderFooterEdit(isHeader: false);
  }

  Future<void> _activateHeaderFooterEdit({required bool isHeader}) async {
    final seed = _host.engine!.fetchHeaderFooterSeedRun(
      caretRunId: _selection.defaultRunId(),
      isHeader: isHeader,
      pageIndex: _selection.caretPage,
    );
    if (seed == null) return;
    _editZone = isHeader ? DocumentEditZone.header : DocumentEditZone.footer;
    _selection.setCaret(seed, 0, page: _selection.caretPage);
    _selection.syncCaretGeometry();
    _formatting.syncFromCaret();
    notifyListeners();
  }

  void closeHeaderFooterEdit() {
    _editZone = DocumentEditZone.body;
    _selection.ensureGlyphCaret();
    _selection.syncCaretGeometry();
    notifyListeners();
  }

  Future<void> insertPageNumberField() async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertFieldAsync(
        runId: runId,
        offset: _selection.caretOffset,
        fieldType: 'page',
      ),
      'Page number inserted',
      full: true,
    );
    final seed = _host.engine!.fetchHeaderFooterSeedRun(
      caretRunId: runId,
      isHeader: _editZone == DocumentEditZone.header,
    );
    final caret = seed ?? runId;
    _selection.setCaret(caret, 1, page: _selection.caretPage);
    _selection.syncCaretGeometry();
    notifyListeners();
  }

  Future<void> insertDateField() async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertFieldAsync(
        runId: runId,
        offset: _selection.caretOffset,
        fieldType: 'date',
      ),
      'Date inserted',
    );
    notifyListeners();
  }

  // ── Legacy stubs (removed TextField path) ─────────────────────────────────
  @Deprecated('TextField fallback removed in R2.4')
  void attachTextEditor(TextEditingController c, FocusNode f) {}

  @Deprecated('TextField fallback removed in R2.4')
  void detachTextEditor(TextEditingController c) {}

  @Deprecated('TextField fallback removed in R2.4')
  TextEditingController? get textController => null;

  @Deprecated('TextField fallback removed in R2.4')
  String textForPage(int pageIndex) => _host.documentText;

  @Deprecated('TextField fallback removed in R2.4')
  void replacePageText(int pageIndex, String text) {
    _host.setDocumentText(text);
    notifyListeners();
  }

  // ── Test hooks ────────────────────────────────────────────────────────────
  @visibleForTesting
  void setDocumentReadOnlyForTest(bool readOnly) =>
      _session.setDocumentReadOnlyForTest(readOnly);

  @visibleForTesting
  void setDisplayListForTest(Uint8List bytes, {bool preferTextRendering = false, int pageCount = 1}) {
    _host.injectDisplayListForTest(bytes, pageCount: pageCount);
    notifyListeners();
  }

  // ── Private helpers ───────────────────────────────────────────────────────
  Future<void> _completeGlyphDrag(int pageIndex, double x, double y) async {
    _selection.completeGlyphDrag(pageIndex, x, y);
    await moveGlyphSelectionTo(pageIndex, x, y);
  }

  Future<bool> _tryPasteFormatted(String runId, EditorClipboardPayload payload) async {
    if (payload.docxBytes != null && payload.docxBytes!.isNotEmpty) {
      final edit = _host.performNativeEdit(
        () => _host.engine!.tryPasteDocxAsync(runId, _selection.caretOffset, payload.docxBytes!),
        dirtyPage: _selection.caretPage,
      );
      if (await edit) {
        _selection.afterInsert(runId, _selection.caretOffset + (payload.plainText?.length ?? 0));
        return true;
      }
    }
    final html = payload.html?.trim();
    if (html != null && html.isNotEmpty) {
      final edit = _host.performNativeEdit(
        () => _host.engine!.tryPasteHtmlAsync(runId, _selection.caretOffset, html),
        dirtyPage: _selection.caretPage,
      );
      if (await edit) {
        _selection.afterInsert(runId, _selection.caretOffset + (payload.plainText?.length ?? 0));
        return true;
      }
    }
    return false;
  }

  Future<bool> _tryPastePlain(String runId, String? text) async {
    if (text == null || text.isEmpty) return false;
    final edit = _host.performNativeEdit(
      () => _host.engine!.tryInsertTextAsync(runId, _selection.caretOffset, text),
      dirtyPage: _selection.caretPage,
    );
    if (!await edit) return false;
    _selection.afterInsert(runId, _selection.caretOffset + text.length);
    return true;
  }

  @override
  void dispose() {
    _session.disposeSession();
    for (final sub in _subControllers) {
      sub.removeListener(notifyListeners);
    }
    super.dispose();
  }
}
