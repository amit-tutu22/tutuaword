import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/find_format_filter.dart';
import 'package:tutuaword/bridge/find_match.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/bridge/spell_issue.dart';
import 'package:tutuaword/bridge/wasm_engine_web.dart';
import 'package:tutuaword/editor/doc_range.dart';

/// Adapts [WasmEngine] to [DocumentEngine].
class WasmDocumentEngine implements DocumentEngine {
  WasmDocumentEngine(this._inner);

  final WasmEngine _inner;

  @override
  DisplayListData? fetchDisplayList() => _inner.fetchDisplayList();

  @override
  PageDisplayListData? fetchPageDisplayList(int page) =>
      _inner.fetchPageDisplayList(page);

  @override
  AtlasData? fetchAtlas() => _inner.fetchAtlas();

  @override
  int? fetchAtlasGeneration() => _inner.fetchAtlasGeneration();

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
  String? fetchChartDataJson(String shapeId) => _inner.fetchChartDataJson(shapeId);

  @override
  String? latestChartId() => _inner.latestChartId();

  @override
  String? fetchOfficeMathXml(String runId) => _inner.fetchOfficeMathXml(runId);

  @override
  String? latestOfficeMathRunId() => _inner.latestOfficeMathRunId();

  @override
  String? fetchImageAltText(String imageId) => _inner.fetchImageAltText(imageId);

  @override
  Uint8List? fetchImageAssetBytes(String assetId) =>
      _inner.fetchImageAssetBytes(assetId);

  @override
  String? fetchDocumentOutline() => _inner.fetchDocumentOutline();

  @override
  String? fetchBookmarks() => _inner.fetchBookmarks();

  @override
  String? fetchHyperlinkAt(String runId) => _inner.fetchHyperlinkAt(runId);

  @override
  String? fetchParagraphNav(String runId, int offset) =>
      _inner.fetchParagraphNav(runId);

  @override
  String? fetchSemanticTree() => _inner.fetchSemanticTree();

  @override
  String? fetchAccessibilityIssues() => _inner.fetchAccessibilityIssues();
  @override
  String? fetchRevisions() => _inner.fetchRevisions();

  @override
  String? fetchPluginList() => _inner.fetchPluginList();

  @override
  bool installSamplePluginNative({required bool grantEdit}) =>
      _inner.installSamplePluginNative(grantEdit: grantEdit);

  @override
  String? fetchDocumentInspect() => _inner.fetchDocumentInspect();

  @override
  bool removeInspectFindings({
    bool comments = false,
    bool metadata = false,
    bool hiddenText = false,
  }) =>
      _inner.removeInspectFindings(
        comments: comments,
        metadata: metadata,
        hiddenText: hiddenText,
      );

  @override
  String? fetchDigitalSignatures() => _inner.fetchDigitalSignatures();

  @override
  String? verifyDigitalSignatures() => _inner.verifyDigitalSignatures();

  @override
  bool signDocument({
    required String name,
    String email = '',
    String? organization,
  }) =>
      _inner.signDocument(
        name: name,
        email: email,
        organization: organization,
      );

  @override
  bool clearDigitalSignatures() => _inner.clearDigitalSignatures();

  @override
  DocumentProperties fetchDocumentProperties() =>
      _inner.fetchDocumentProperties();

  @override
  bool isDocumentReadOnly() => _inner.isDocumentReadOnly();

  @override
  bool isPageStale(int page) => _inner.isPageStale(page);

  @override
  String? getLastError() => _inner.getLastError();

  @override
  bool newDocument() => _inner.newDocument();

  @override
  int openDocumentBytes(Uint8List bytes, {String? path, String? password}) =>
      _inner.openDocumentBytes(bytes, path: path, password: password);

  @override
  Future<int> openDocumentBytesAsync(
    Uint8List bytes, {
    String? path,
    String? password,
    Duration timeout = const Duration(seconds: 120),
  }) =>
      _inner.openDocumentBytesAsync(
        bytes,
        path: path,
        password: password,
        timeout: timeout,
      );

  @override
  Uint8List? saveDocumentBytes() => _inner.saveDocumentBytes();

  @override
  Future<Uint8List?> saveDocumentBytesAsync({
    Duration timeout = const Duration(seconds: 120),
  }) =>
      _inner.saveDocumentBytesAsync(timeout: timeout);

  @override
  Uint8List? saveDocumentAsBytes(String formatExtension) =>
      _inner.saveDocumentAsBytes(formatExtension);

  @override
  Uint8List? exportPdfBytes() => _inner.exportPdfBytes();

  @override
  Uint8List? exportPdfBytesForPrint([
    PrintLayoutSettings? layout,
    DocRange? selection,
  ]) =>
      _inner.exportPdfBytesForPrint(layout, selection);

  @override
  HitTestResult? hitTestPage(int page, double x, double y) =>
      _inner.hitTestPage(page, x, y);

  @override
  HitTestResult? fetchDocumentTailHit(int page) =>
      _inner.fetchDocumentTailHit(page);

  @override
  HitTestResult? fetchLastSplitCaret() => _inner.fetchLastSplitCaret();

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
  Future<bool> replaceRangeAsync(
    String runId,
    int start,
    int end,
    String text,
  ) =>
      _inner.replaceRangeAsync(runId, start, end, text);

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
  Future<HitTestResult?> splitParagraphAsync(String runId, int offset) =>
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
    String? mergeName,
  }) =>
      _inner.insertFieldAsync(
        runId: runId,
        offset: offset,
        fieldType: fieldType,
        mergeName: mergeName,
      );

  @override
  Future<bool> applySpellReplacementAsync({
    required int plainStart,
    required int plainEnd,
    required String replacement,
  }) =>
      _inner.applySpellReplacementAsync(
        plainStart: plainStart,
        plainEnd: plainEnd,
        replacement: replacement,
      );

  @override
  Future<Uint8List?> exportSelectionDocxAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
  }) =>
      _inner.exportSelectionDocxAsync(
        startRunId: startRunId,
        startOffset: startOffset,
        endRunId: endRunId,
        endOffset: endOffset,
      );

  @override
  String? getCommentsJson() => _inner.getCommentsJson();

  @override
  Future<bool> replyToCommentAsync({
    required int commentId,
    required String bodyText,
  }) =>
      _inner.replyToCommentAsync(commentId: commentId, bodyText: bodyText);

  @override
  Future<bool> resolveCommentAsync({
    required int commentId,
    required bool resolved,
  }) =>
      _inner.resolveCommentAsync(commentId: commentId, resolved: resolved);

  @override
  Future<bool> insertFootnoteAsync({
    required String runId,
    required int offset,
  }) =>
      _inner.insertFootnoteAsync(runId: runId, offset: offset);

  @override
  Future<bool> insertEndnoteAsync({
    required String runId,
    required int offset,
  }) =>
      _inner.insertEndnoteAsync(runId: runId, offset: offset);

  @override
  Future<bool> insertCommentAsync({
    required String runId,
    required int offset,
    String bodyText = '',
  }) =>
      _inner.insertCommentAsync(
        runId: runId,
        offset: offset,
        bodyText: bodyText,
      );

  @override
  Future<bool> insertTableOfContentsAsync({String? caretRunId}) =>
      _inner.insertTableOfContentsAsync(caretRunId: caretRunId);

  @override
  Future<bool> insertTableOfFiguresAsync({String? caretRunId}) =>
      _inner.insertTableOfFiguresAsync(caretRunId: caretRunId);

  @override
  Future<bool> addBibliographySourceAsync({
    required String key,
    required String author,
    required String title,
    required String year,
  }) =>
      _inner.addBibliographySourceAsync(
        key: key,
        author: author,
        title: title,
        year: year,
      );

  @override
  Future<bool> insertCitationAsync({
    required String runId,
    required int offset,
    required String sourceKey,
  }) =>
      _inner.insertCitationAsync(
        runId: runId,
        offset: offset,
        sourceKey: sourceKey,
      );

  @override
  Future<bool> insertBibliographyAsync({String? caretRunId}) =>
      _inner.insertBibliographyAsync(caretRunId: caretRunId);

  @override
  Future<bool> insertBookmarkAsync({
    required String runId,
    required int offset,
    required String name,
  }) =>
      _inner.insertBookmarkAsync(runId: runId, offset: offset, name: name);

  @override
  Future<bool> insertHyperlinkAsync({
    required String runId,
    required int offset,
    required String url,
    required String text,
    String? tooltip,
  }) =>
      _inner.insertHyperlinkAsync(
        runId: runId,
        offset: offset,
        url: url,
        text: text,
        tooltip: tooltip,
      );

  @override
  Future<bool> insertFormFieldAsync({
    required String runId,
    required int offset,
    required String kind,
    String? name,
    String? initialValue,
  }) =>
      _inner.insertFormFieldAsync(
        runId: runId,
        offset: offset,
        kind: kind,
        name: name,
        initialValue: initialValue,
      );

  @override
  Future<bool> setFormFieldValueAsync({
    required String runId,
    required String value,
  }) =>
      _inner.setFormFieldValueAsync(runId: runId, value: value);

  @override
  Future<bool> insertMergeFieldAsync({
    required String runId,
    required int offset,
    required String name,
  }) =>
      _inner.insertMergeFieldAsync(runId: runId, offset: offset, name: name);

  @override
  Future<bool> applyMailMergeRowAsync({
    required Map<String, String> values,
  }) =>
      _inner.applyMailMergeRowAsync(values: values);

  @override
  Future<bool> insertCrossReferenceAsync({
    required String runId,
    required int offset,
    required String bookmarkName,
  }) =>
      _inner.insertCrossReferenceAsync(
        runId: runId,
        offset: offset,
        bookmarkName: bookmarkName,
      );

  @override
  Future<bool> insertIndexAsync({String? caretRunId}) =>
      _inner.insertIndexAsync(caretRunId: caretRunId);

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
  Future<bool> moveBlockAsync({
    String? caretRunId,
    int caretOffset = 0,
    required int delta,
  }) =>
      _inner.moveBlockAsync(caretRunId: caretRunId, delta: delta);

  @override
  Future<bool> restartNumberingAsync({String? caretRunId}) =>
      _inner.restartNumberingAsync(caretRunId: caretRunId);

  @override
  Future<bool> continueNumberingAsync({String? caretRunId}) =>
      _inner.continueNumberingAsync(caretRunId: caretRunId);

  @override
  Future<bool> insertTableBlockAsync(int rows, int cols, {String? caretRunId}) =>
      _inner.insertTableBlockAsync(rows, cols, caretRunId: caretRunId);

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
  Future<bool> insertShapeBlockAsync(int shapeType, {String? caretRunId}) =>
      _inner.insertShapeBlockAsync(shapeType, caretRunId: caretRunId);

  @override
  Future<bool> insertTextBoxAsync({String? caretRunId}) =>
      _inner.insertTextBoxAsync(caretRunId: caretRunId);

  @override
  Future<bool> insertWordArtAsync(String text, {String? caretRunId}) =>
      _inner.insertWordArtAsync(text, caretRunId: caretRunId);

  @override
  Future<bool> insertDiagramAsync({int diagramType = 0, String? caretRunId}) =>
      _inner.insertDiagramAsync(diagramType: diagramType, caretRunId: caretRunId);

  @override
  Future<bool> insertChartAsync({int chartType = 0, String? caretRunId}) =>
      _inner.insertChartAsync(chartType: chartType, caretRunId: caretRunId);

  @override
  Future<bool> setChartDataAsync(String shapeId, Map<String, dynamic> chartData) =>
      _inner.setChartDataAsync(shapeId, chartData);

  @override
  Future<bool> ensureShapeTextAsync(String shapeId) =>
      _inner.ensureShapeTextAsync(shapeId);

  @override
  Future<bool> insertOfficeMathAsync({
    required String runId,
    required int offset,
    required String xml,
  }) =>
      _inner.insertOfficeMathAsync(runId: runId, offset: offset, xml: xml);

  @override
  Future<bool> insertOfficeMathDisplayAsync({
    String? caretRunId,
    required String xml,
  }) =>
      _inner.insertOfficeMathDisplayAsync(caretRunId: caretRunId, xml: xml);

  @override
  Future<bool> setOfficeMathAsync(String runId, String xml) =>
      _inner.setOfficeMathAsync(runId, xml);

  @override
  Future<bool> deleteBlockAsync(String blockId) =>
      _inner.deleteBlockAsync(blockId);

  @override
  Future<bool> insertImageBytesAsync(
    Uint8List bytes,
    String mimeType, {
    String? caretRunId,
  }) =>
      _inner.insertImageBytesAsync(bytes, mimeType, caretRunId: caretRunId);

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
  Future<bool> setShapeAnchorAsync(
    String shapeId,
    double x,
    double y, {
    int originX = 0,
    int originY = 0,
  }) =>
      _inner.setShapeAnchorAsync(
        shapeId,
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
  Future<bool> setImageAltTextAsync(String imageId, String? altText) =>
      _inner.setImageAltTextAsync(imageId, altText);

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
  List<SpellIssue>? spellCheckIssues() => _inner.spellCheckIssues();

  @override
  List<String>? grammarCheckIssues() => _inner.grammarCheckIssues();

  @override
  List<FindMatch>? findMatches(
    String query,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
    FindFormatFilter formatFilter = FindFormatFilter.none,
  }) =>
      _inner.findMatches(
        query,
        matchCase,
        useRegex: useRegex,
        useWildcards: useWildcards,
        formatFilter: formatFilter,
      );

  @override
  Future<int?> replaceAll(
    String find,
    String replace,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
  }) =>
      _inner.replaceAll(
        find,
        replace,
        matchCase,
        useRegex: useRegex,
        useWildcards: useWildcards,
      );

  @override
  String? compareDocumentText(String otherText) =>
      _inner.compareDocumentText(otherText);

  @override
  bool setReadOnlyEnabled(bool enabled) => _inner.setReadOnlyEnabled(enabled);

  @override
  bool setEncryptionPassword(String? password) =>
      _inner.setEncryptionPassword(password);

  @override
  bool setTrackChangesEnabled(bool enabled) =>
      _inner.setTrackChangesEnabled(enabled);

  @override
  bool acceptAllRevisions() => _inner.acceptAllRevisions();

  @override
  bool rejectAllRevisions() => _inner.rejectAllRevisions();

  @override
  bool acceptRevisionAtCaret({String? caretRunId}) =>
      _inner.acceptRevisionAtCaret(caretRunId: caretRunId);

  @override
  bool rejectRevisionAtCaret({String? caretRunId}) =>
      _inner.rejectRevisionAtCaret(caretRunId: caretRunId);

  @override
  String? adjacentRevisionRunId(String? caretRunId, {required bool forward}) =>
      _inner.adjacentRevisionRunId(caretRunId, forward: forward);
}
