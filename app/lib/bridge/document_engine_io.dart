import 'dart:typed_data';

import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/native_engine.dart';

/// Adapts [NativeEngine] to [DocumentEngine].
class FfiDocumentEngine implements DocumentEngine {
  FfiDocumentEngine(this._inner);

  final NativeEngine _inner;

  @override
  DisplayListData? fetchDisplayList() => _inner.fetchDisplayList();

  @override
  PageDisplayListData? fetchPageDisplayList(int page) =>
      _inner.fetchPageDisplayList(page);

  @override
  AtlasData? fetchAtlas() => _inner.fetchAtlas();

  @override
  String? fetchDocumentText() => _inner.fetchDocumentText();

  @override
  String? fetchTextRange(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) =>
      _inner.fetchTextRange(startRunId, startOffset, endRunId, endOffset);

  @override
  String? fetchCaretFormat(String runId) => _inner.fetchCaretFormat(runId);

  @override
  DocumentProperties fetchDocumentProperties() =>
      _inner.fetchDocumentProperties();

  @override
  bool isDocumentReadOnly() => _inner.isDocumentReadOnly();

  @override
  bool isPageStale(int page) => _inner.isPageStale(page);

  @override
  int? fetchAtlasGeneration() => _inner.fetchAtlasGeneration();

  @override
  String? getLastError() => _inner.getLastError();

  @override
  bool newDocument() => _inner.newDocument();

  @override
  int openDocumentBytes(Uint8List bytes, {String? path}) =>
      _inner.openDocumentBytes(bytes, path: path);

  @override
  Uint8List? saveDocumentBytes() => _inner.saveDocumentBytes();

  @override
  Uint8List? saveDocumentAsBytes(String formatExtension) =>
      _inner.saveDocumentAsBytes(formatExtension);

  @override
  Uint8List? exportPdfBytes() => _inner.exportPdfBytes();

  @override
  HitTestResult? hitTestPage(int page, double x, double y) =>
      _inner.hitTestPage(page, x, y);

  @override
  HitTestResult? fetchDocumentTailHit(int page) =>
      _inner.fetchDocumentTailHit(page);

  @override
  CaretGeometry? caretGeometryAt(int page, double x, double y) =>
      _inner.caretGeometryAt(page, x, y);

  @override
  CaretGeometry? caretAtPosition(int page, String runId, int charOffset) =>
      _inner.caretAtPosition(page, runId, charOffset);

  @override
  List<GlyphSelectionRect> selectionRectsOnPage(
    int page,
    double startX,
    double startY,
    double endX,
    double endY,
  ) =>
      _inner.selectionRectsOnPage(page, startX, startY, endX, endY);

  @override
  void insertText(String runId, int offset, String text) =>
      _inner.insertText(runId, offset, text);

  @override
  bool tryInsertText(String runId, int offset, String text) =>
      _inner.tryInsertText(runId, offset, text);

  @override
  Future<bool> tryInsertTextAsync(String runId, int offset, String text) =>
      _inner.tryInsertTextAsync(runId, offset, text);

  @override
  Future<bool> tryPasteHtmlAsync(String runId, int offset, String html) =>
      _inner.tryPasteHtmlAsync(runId, offset, html);

  @override
  Future<bool> tryPasteDocxAsync(String runId, int offset, Uint8List bytes) =>
      _inner.tryPasteDocxAsync(runId, offset, bytes);

  @override
  Future<bool> deleteRangeAsync(String runId, int start, int end) =>
      _inner.deleteRangeAsync(runId, start, end);

  @override
  Future<bool> deleteDocRangeAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) =>
      _inner.deleteDocRangeAsync(
        startRunId,
        startOffset,
        endRunId,
        endOffset,
      );

  @override
  Future<bool> splitParagraphAsync(String runId, int offset) =>
      _inner.splitParagraphAsync(runId, offset);

  @override
  Future<bool> applyCharFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) =>
      _inner.applyCharFormatJsonAsync(
        startRunId: startRunId,
        startOffset: startOffset,
        endRunId: endRunId,
        endOffset: endOffset,
        formatJson: formatJson,
      );

  @override
  Future<bool> applyParaFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) =>
      _inner.applyParaFormatJsonAsync(
        startRunId: startRunId,
        startOffset: startOffset,
        endRunId: endRunId,
        endOffset: endOffset,
        formatJson: formatJson,
      );

  @override
  Future<bool> clearFormatAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) =>
      _inner.clearFormatAsync(startRunId, startOffset, endRunId, endOffset);

  @override
  Future<bool> insertPageBreakAtAsync({String? caretRunId}) =>
      _inner.insertPageBreakAtAsync(caretRunId: caretRunId);

  @override
  bool setCurrentPageIndex(int page) => _inner.setCurrentPageIndex(page);

  @override
  Future<bool> applyHeading1StyleAsync({String? caretRunId}) =>
      _inner.applyHeading1StyleAsync(caretRunId: caretRunId);

  @override
  Future<bool> applyNormalStyleAtAsync({String? caretRunId}) =>
      _inner.applyNormalStyleAtAsync(caretRunId: caretRunId);

  @override
  Future<bool> applyBulletListStyleAsync({String? caretRunId}) =>
      _inner.applyBulletListStyleAsync(caretRunId: caretRunId);

  @override
  Future<bool> applyNumberedListStyleAsync({String? caretRunId}) =>
      _inner.applyNumberedListStyleAsync(caretRunId: caretRunId);

  @override
  Future<bool> insertTableBlockAsync(int rows, int cols) =>
      _inner.insertTableBlockAsync(rows, cols);

  @override
  Future<bool> insertImageBlockAsync(double width, double height) =>
      _inner.insertImageBlockAsync(width, height);

  @override
  Future<bool> undoEditAsync() => _inner.undoEditAsync();

  @override
  Future<bool> redoEditAsync() => _inner.redoEditAsync();

  @override
  List<String>? spellCheckMisspellings() => _inner.spellCheckMisspellings();

  @override
  bool setTrackChangesEnabled(bool enabled) =>
      _inner.setTrackChangesEnabled(enabled);

  @override
  bool acceptAllRevisions() => _inner.acceptAllRevisions();

  @override
  bool rejectAllRevisions() => _inner.rejectAllRevisions();
}

DocumentEngine? loadDocumentEngine() {
  final ffi = NativeEngine.load();
  return ffi == null ? null : FfiDocumentEngine(ffi);
}
