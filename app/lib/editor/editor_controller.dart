import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/native_engine.dart';
import 'package:tutuaword/editor/controllers/document_session_controller.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/controllers/formatting_controller.dart';
import 'package:tutuaword/editor/controllers/selection_controller.dart';
import 'package:tutuaword/editor/controllers/view_controller.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/ui/paste_special_dialog.dart';

export 'package:tutuaword/bridge/native_engine.dart' show CaretGeometry, GlyphSelectionRect;
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

  String _bootStatus = '';

  // ── Sub-controller accessors (R2.4 decomposition) ─────────────────────────
  ViewController get view => _view;
  SelectionController get selectionController => _selection;
  FormattingController get formattingController => _formatting;
  DocumentSessionController get sessionController => _session;

  // ── Engine / rendering ────────────────────────────────────────────────────
  bool get isEngineConnected => _host.isConnected;
  bool get usesGlyphRendering => _host.isConnected;
  bool get preferTextRendering => false;
  Uint8List get displayListBytes => _host.displayListBytes;
  double get pageWidth => _host.pageWidth;
  double get pageHeight => _host.pageHeight;
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
  double get indentLeft => _formatting.indentLeft;

  void toggleBold() => _formatting.toggleBold();
  void toggleItalic() => _formatting.toggleItalic();
  void toggleUnderline() => _formatting.toggleUnderline();
  void setFontFamily(String f) => _formatting.setFontFamily(f);
  void setFontSize(double s) => _formatting.setFontSize(s);
  void increaseFontSize() => _formatting.increaseFontSize();
  void decreaseFontSize() => _formatting.decreaseFontSize();
  void setFontColor(Color c) => _formatting.setFontColor(c);
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
    // Caret advances before the worker acknowledges, so the next keystroke
    // targets the right offset and the display refresh arrives with the event.
    final optimistic = offset + char.length;
    _selection.afterInsert(runId, optimistic);
    _session.markDocumentDirty();
    notifyListeners();
    if (!await edit) {
      _rollbackCaret(runId, from: optimistic, to: offset);
    }
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
    final prevX = _selection.caretGeometry?.x ?? margin;
    final edit = _host.performNativeEdit(
      () => _host.engine!.splitParagraphAsync(runId, _selection.caretOffset),
      dirtyPage: _selection.caretPage,
    );
    await edit;
    final nextY = prevY + _formatting.fontSize * 1.4;
    hitTestAt(_selection.caretPage, prevX.clamp(margin, pageWidth - margin), nextY);
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
      _selection.afterInsert(runId, off - 1);
      _session.markDocumentDirty();
      notifyListeners();
      if (!await edit) {
        _rollbackCaret(runId, from: off - 1, to: off);
      }
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

  void insertImage() => _session.applyEngineStyle(
        () => _host.engine!.insertImageBlockAsync(200, 150),
        'Image placeholder inserted',
      );

  void applyHeading1() => _session.applyEngineStyle(
        () => _host.engine!.applyHeading1StyleAsync(caretRunId: _selection.defaultRunId()),
        'Heading 1 applied',
      ).then((_) => _formatting.setActiveParagraphStyle('Heading 1'));

  void applyNormalStyle() => _session.applyEngineStyle(
        () => _host.engine!.applyNormalStyleAtAsync(caretRunId: _selection.defaultRunId()),
        'Normal style applied',
      ).then((_) => _formatting.setActiveParagraphStyle('Normal'));

  void applyNumberedList() => _session.applyEngineStyle(
        () => _host.engine!.applyNumberedListStyleAsync(caretRunId: _selection.defaultRunId()),
        'Numbered list applied',
      );

  void applyBulletList() => _session.applyEngineStyle(
        () => _host.engine!.applyBulletListStyleAsync(caretRunId: _selection.defaultRunId()),
        'Bullet list applied',
      );

  void insertPageBreak() => _session.applyEngineStyle(
        () => _host.engine!.insertPageBreakAtAsync(caretRunId: _selection.defaultRunId()),
        'Page break inserted',
      );

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
