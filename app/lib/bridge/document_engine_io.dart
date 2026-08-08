import 'dart:typed_data';

import 'package:flutter/material.dart';
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
  String? fetchSectionFormat({String? caretRunId}) =>
      _inner.fetchSectionFormat(caretRunId: caretRunId);

  @override
  String? fetchDocumentOutline() => _inner.fetchDocumentOutline();

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
  Future<bool> insertSectionBreakAtAsync({String? caretRunId}) =>
      _inner.insertSectionBreakAtAsync(caretRunId: caretRunId);

  @override
  Future<bool> ensureHeaderFooterAsync({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) =>
      _inner.ensureHeaderFooterAsync(
        caretRunId: caretRunId,
        isHeader: isHeader,
        pageIndex: pageIndex,
      );

  @override
  String? fetchHeaderFooterSeedRun({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) =>
      _inner.fetchHeaderFooterSeedRun(
        caretRunId: caretRunId,
        isHeader: isHeader,
        pageIndex: pageIndex,
      );

  @override
  bool fetchEvenAndOddHeadersEnabled() => _inner.fetchEvenAndOddHeadersEnabled();

  @override
  Future<bool> setEvenAndOddHeadersAsync({required bool enabled}) =>
      _inner.setEvenAndOddHeadersAsync(enabled: enabled);

  @override
  bool fetchHeaderFooterLinked({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) =>
      _inner.fetchHeaderFooterLinked(
        caretRunId: caretRunId,
        isHeader: isHeader,
        pageIndex: pageIndex,
      );

  @override
  Future<bool> setHeaderFooterLinkAsync({
    String? caretRunId,
    required bool isHeader,
    required bool linked,
    int pageIndex = 0,
  }) =>
      _inner.setHeaderFooterLinkAsync(
        caretRunId: caretRunId,
        isHeader: isHeader,
        linked: linked,
        pageIndex: pageIndex,
      );

  @override
  Future<bool> insertFieldAsync({
    required String runId,
    required int offset,
    required String fieldType,
  }) =>
      _inner.insertFieldAsync(runId: runId, offset: offset, fieldType: fieldType);

  @override
  bool setCurrentPageIndex(int page) => _inner.setCurrentPageIndex(page);

  @override
  Future<bool> applyHeading1StyleAsync({String? caretRunId}) =>
      _inner.applyHeading1StyleAsync(caretRunId: caretRunId);

  @override
  Future<bool> applyNormalStyleAtAsync({String? caretRunId}) =>
      _inner.applyNormalStyleAtAsync(caretRunId: caretRunId);

  @override
  Future<bool> applyParagraphStyleAsync({
    String? caretRunId,
    required String styleName,
  }) =>
      _inner.applyParagraphStyleAsync(caretRunId: caretRunId, styleName: styleName);

  @override
  Future<bool> applyDocumentThemeAsync({required String themeName}) =>
      _inner.applyDocumentThemeAsync(themeName: themeName);

  @override
  Future<bool> applySectionFormatJsonAsync({
    required String formatJson,
    String? caretRunId,
  }) =>
      _inner.applySectionFormatJsonAsync(
        formatJson: formatJson,
        caretRunId: caretRunId,
      );

  @override
  Future<bool> applyBulletListStyleAsync({String? caretRunId}) =>
      _inner.applyBulletListStyleAsync(caretRunId: caretRunId);

  @override
  Future<bool> applyNumberedListStyleAsync({String? caretRunId}) =>
      _inner.applyNumberedListStyleAsync(caretRunId: caretRunId);

  @override
  Future<bool> adjustListLevelAsync({String? caretRunId, required int delta}) =>
      _inner.adjustListLevelAsync(caretRunId: caretRunId, delta: delta);

  @override
  Future<bool> restartNumberingAsync({String? caretRunId}) =>
      _inner.restartNumberingAsync(caretRunId: caretRunId);

  @override
  Future<bool> continueNumberingAsync({String? caretRunId}) =>
      _inner.continueNumberingAsync(caretRunId: caretRunId);

  @override
  Future<bool> insertTableBlockAsync(int rows, int cols) =>
      _inner.insertTableBlockAsync(rows, cols);

  @override
  Future<bool> deleteTableRowAsync({String? caretRunId}) =>
      _inner.deleteTableRowAsync(caretRunId: caretRunId);

  @override
  Future<bool> deleteTableColumnAsync({String? caretRunId}) =>
      _inner.deleteTableColumnAsync(caretRunId: caretRunId);

  @override
  Future<bool> mergeTableCellsAsync({String? caretRunId}) =>
      _inner.mergeTableCellsAsync(caretRunId: caretRunId);

  @override
  Future<bool> splitTableCellAsync({String? caretRunId}) =>
      _inner.splitTableCellAsync(caretRunId: caretRunId);

  @override
  Future<bool> setTableBorderAsync({
    String? caretRunId,
    required double width,
    required Color color,
  }) =>
      _inner.setTableBorderAsync(
        caretRunId: caretRunId,
        width: width,
        color: color,
      );

  @override
  Future<bool> setTableCellShadingAsync({
    String? caretRunId,
    Color? shading,
  }) =>
      _inner.setTableCellShadingAsync(caretRunId: caretRunId, shading: shading);

  @override
  Future<bool> resizeTableColumnAsync({
    String? caretRunId,
    required double width,
  }) =>
      _inner.resizeTableColumnAsync(caretRunId: caretRunId, width: width);

  @override
  Future<bool> autofitTableAsync({String? caretRunId}) =>
      _inner.autofitTableAsync(caretRunId: caretRunId);

  @override
  Future<bool> sortTableRowsAsync({String? caretRunId, required bool ascending}) =>
      _inner.sortTableRowsAsync(caretRunId: caretRunId, ascending: ascending);

  @override
  Future<bool> insertNestedTableAsync({
    String? caretRunId,
    required int rows,
    required int cols,
  }) =>
      _inner.insertNestedTableAsync(caretRunId: caretRunId, rows: rows, cols: cols);

  @override
  Future<bool> insertTableSumFieldAsync({String? caretRunId}) =>
      _inner.insertTableSumFieldAsync(caretRunId: caretRunId);

  @override
  Future<bool> insertImageBlockAsync(double width, double height) =>
      _inner.insertImageBlockAsync(width, height);

  @override
  Future<bool> insertShapeBlockAsync(int shapeType) =>
      _inner.insertShapeBlockAsync(shapeType);

  @override
  Future<bool> insertTextBoxAsync() => _inner.insertTextBoxAsync();

  @override
  Future<bool> insertWordArtAsync(String text) => _inner.insertWordArtAsync(text);

  @override
  Future<bool> insertDiagramAsync() => _inner.insertDiagramAsync();

  @override
  Future<bool> insertChartAsync() => _inner.insertChartAsync();

  @override
  Future<bool> insertImageBytesAsync(Uint8List bytes, String mimeType) =>
      _inner.insertImageBytesAsync(bytes, mimeType);

  @override
  Future<bool> setImageSizeAsync(String imageId, double width, double height) =>
      _inner.setImageSizeAsync(imageId, width, height);

  @override
  Future<bool> replaceImageBytesAsync(
    String imageId,
    Uint8List bytes,
    String mimeType,
  ) =>
      _inner.replaceImageBytesAsync(imageId, bytes, mimeType);

  @override
  Future<bool> setImageWrapAsync(String imageId, int wrap) =>
      _inner.setImageWrapAsync(imageId, wrap);

  @override
  Future<bool> setImageAnchorAsync(
    String imageId,
    double x,
    double y, {
    int originX = 0,
    int originY = 0,
  }) =>
      _inner.setImageAnchorAsync(
        imageId,
        x,
        y,
        originX: originX,
        originY: originY,
      );

  @override
  Future<bool> setImageTransformAsync(
    String imageId, {
    double rotationDeg = 0,
    double cropLeft = 0,
    double cropTop = 0,
    double cropRight = 0,
    double cropBottom = 0,
    double opacity = 1,
  }) =>
      _inner.setImageTransformAsync(
        imageId,
        rotationDeg: rotationDeg,
        cropLeft: cropLeft,
        cropTop: cropTop,
        cropRight: cropRight,
        cropBottom: cropBottom,
        opacity: opacity,
      );

  @override
  Future<bool> insertImageCaptionAsync(String imageId) =>
      _inner.insertImageCaptionAsync(imageId);

  @override
  Future<bool> compressImageAsync(String imageId, int quality) =>
      _inner.compressImageAsync(imageId, quality);

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
