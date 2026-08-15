import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/find_format_filter.dart';
import 'package:tutuaword/bridge/find_match.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/bridge/spell_issue.dart';
import 'package:tutuaword/editor/doc_range.dart';

/// Minimal engine surface used by editor controllers (FFI or mock).
abstract class DocumentEngine {
  DisplayListData? fetchDisplayList();
  PageDisplayListData? fetchPageDisplayList(int page);
  AtlasData? fetchAtlas();

  /// Atlas generation without the pixel copy, or null when unavailable — a null
  /// forces the caller to fall back to [fetchAtlas] to detect a change.
  int? fetchAtlasGeneration();
  String? fetchDocumentText();
  String? fetchTextRange(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  );
  String? fetchCaretFormat(String runId);
  String? fetchSectionFormat({String? caretRunId});
  String? fetchDocumentOutline();
  /// JSON array of bookmarks: `{ name, run_id, paragraph_id, page }` (F19.S4).
  String? fetchBookmarks();
  /// JSON semantic accessibility tree (F21.S1).
  String? fetchSemanticTree();
  /// JSON accessibility checker issues (F21.S4).
  String? fetchAccessibilityIssues();
  /// JSON Document Inspector findings (F22.S3).
  String? fetchDocumentInspect();
  /// Remove selected Document Inspector categories (F22.S3).
  bool removeInspectFindings({
    bool comments = false,
    bool metadata = false,
    bool hiddenText = false,
  });
  /// JSON digital signatures list (F22.S4).
  String? fetchDigitalSignatures();
  /// JSON verification results for all signatures (F22.S4).
  String? verifyDigitalSignatures();
  /// Sign the document with the given identity (F22.S4).
  bool signDocument({
    required String name,
    String email = '',
    String? organization,
  });
  /// Remove all digital signatures (F22.S4).
  bool clearDigitalSignatures();
  DocumentProperties fetchDocumentProperties();
  bool isDocumentReadOnly();

  /// JSON [`ChartData`] for [shapeId], or null when missing / not a chart.
  String? fetchChartDataJson(String shapeId);

  /// UUID of the last chart block in document order, if any.
  String? latestChartId();

  /// OMML XML for [runId], or null when missing / not an equation run.
  String? fetchOfficeMathXml(String runId);

  /// UUID of the last equation run in document order, if any.
  String? latestOfficeMathRunId();

  /// Alternative text for [imageId], or null when the image is missing (F21.S3).
  /// Empty string means the image has no alt text set.
  String? fetchImageAltText(String imageId);

  /// True while [page] awaits background reflow, so [hitTestPage] on it returns
  /// null for "not laid out yet" rather than "nothing here".
  bool isPageStale(int page);
  String? getLastError();

  bool newDocument();
  int openDocumentBytes(Uint8List bytes, {String? path, String? password});
  Uint8List? saveDocumentBytes();
  Uint8List? saveDocumentAsBytes(String formatExtension);
  Uint8List? exportPdfBytes();

  /// Print-ready PDF (VisualMatch with structural fallback) for File→Print (F25.S1–S3).
  ///
  /// When [selection] is set, only that body range is printed (F25.S3).
  Uint8List? exportPdfBytesForPrint([
    PrintLayoutSettings? layout,
    DocRange? selection,
  ]);

  HitTestResult? hitTestPage(int page, double x, double y);
  HitTestResult? fetchDocumentTailHit(int page);
  CaretGeometry? caretGeometryAt(int page, double x, double y);
  CaretGeometry? caretAtPosition(int page, String runId, int charOffset);
  List<GlyphSelectionRect> selectionRectsOnPage(
    int page,
    double startX,
    double startY,
    double endX,
    double endY,
  );

  void insertText(String runId, int offset, String text);
  bool tryInsertText(String runId, int offset, String text);
  Future<bool> tryInsertTextAsync(String runId, int offset, String text);
  Future<bool> tryPasteHtmlAsync(String runId, int offset, String html);
  Future<bool> tryPasteDocxAsync(String runId, int offset, Uint8List bytes);
  Future<bool> deleteRangeAsync(String runId, int start, int end);
  /// Replace `[start, end)` in a run with [text] (F28.S2; prefer single undo).
  Future<bool> replaceRangeAsync(
    String runId,
    int start,
    int end,
    String text,
  );
  Future<bool> deleteDocRangeAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  );
  Future<HitTestResult?> splitParagraphAsync(String runId, int offset);
  Future<bool> applyCharFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  });
  Future<bool> applyParaFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  });
  Future<bool> clearFormatAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  );
  Future<bool> insertPageBreakAtAsync({String? caretRunId});
  Future<bool> insertSectionBreakAtAsync({String? caretRunId});
  Future<bool> ensureHeaderFooterAsync({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  });
  String? fetchHeaderFooterSeedRun({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  });
  bool fetchEvenAndOddHeadersEnabled();
  Future<bool> setEvenAndOddHeadersAsync({required bool enabled});
  bool fetchHeaderFooterLinked({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  });
  Future<bool> setHeaderFooterLinkAsync({
    String? caretRunId,
    required bool isHeader,
    required bool linked,
    int pageIndex = 0,
  });
  Future<bool> insertFieldAsync({
    required String runId,
    required int offset,
    required String fieldType,
    String? mergeName,
  });
  Future<bool> applySpellReplacementAsync({
    required int plainStart,
    required int plainEnd,
    required String replacement,
  });
  Future<Uint8List?> exportSelectionDocxAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
  });
  String? getCommentsJson();
  Future<bool> replyToCommentAsync({
    required int commentId,
    required String bodyText,
  });
  Future<bool> resolveCommentAsync({
    required int commentId,
    required bool resolved,
  });
  Future<bool> insertFootnoteAsync({
    required String runId,
    required int offset,
  });
  Future<bool> insertEndnoteAsync({
    required String runId,
    required int offset,
  });
  Future<bool> insertCommentAsync({
    required String runId,
    required int offset,
    String bodyText = '',
  });
  Future<bool> insertTableOfContentsAsync({String? caretRunId});
  Future<bool> insertTableOfFiguresAsync({String? caretRunId});
  Future<bool> addBibliographySourceAsync({
    required String key,
    required String author,
    required String title,
    required String year,
  });
  Future<bool> insertCitationAsync({
    required String runId,
    required int offset,
    required String sourceKey,
  });
  Future<bool> insertBibliographyAsync({String? caretRunId});
  Future<bool> insertBookmarkAsync({
    required String runId,
    required int offset,
    required String name,
  });
  Future<bool> insertHyperlinkAsync({
    required String runId,
    required int offset,
    required String url,
    required String text,
    String? tooltip,
  });
  /// Insert a plain-text or checkbox form field (F26.S1).
  ///
  /// [kind] is `"text"` / `"formtext"` or `"checkbox"` / `"formcheckbox"`.
  Future<bool> insertFormFieldAsync({
    required String runId,
    required int offset,
    required String kind,
    String? name,
    String? initialValue,
  });
  /// Update / toggle an existing form field value (F26.S1).
  Future<bool> setFormFieldValueAsync({
    required String runId,
    required String value,
  });
  /// Insert a mail-merge field (`MERGEFIELD Name`) (F26.S2).
  Future<bool> insertMergeFieldAsync({
    required String runId,
    required int offset,
    required String name,
  });
  /// Replace merge fields using one CSV row (column → value) (F26.S2).
  Future<bool> applyMailMergeRowAsync({
    required Map<String, String> values,
  });
  Future<bool> insertCrossReferenceAsync({
    required String runId,
    required int offset,
    required String bookmarkName,
  });
  Future<bool> insertIndexAsync({String? caretRunId});
  bool setCurrentPageIndex(int page);
  Future<bool> applyHeading1StyleAsync({String? caretRunId});
  Future<bool> applyNormalStyleAtAsync({String? caretRunId});
  Future<bool> applyParagraphStyleAsync({String? caretRunId, required String styleName});
  Future<bool> applyDocumentThemeAsync({required String themeName});
  Future<bool> applySectionFormatJsonAsync({
    required String formatJson,
    String? caretRunId,
  });
  Future<bool> applyBulletListStyleAsync({String? caretRunId});
  Future<bool> applyNumberedListStyleAsync({String? caretRunId});
  Future<bool> adjustListLevelAsync({String? caretRunId, required int delta});
  Future<bool> restartNumberingAsync({String? caretRunId});
  Future<bool> continueNumberingAsync({String? caretRunId});
  Future<bool> insertTableBlockAsync(int rows, int cols, {String? caretRunId});
  Future<bool> deleteTableRowAsync({String? caretRunId});
  Future<bool> deleteTableColumnAsync({String? caretRunId});
  Future<bool> mergeTableCellsAsync({String? caretRunId});
  Future<bool> splitTableCellAsync({String? caretRunId});
  Future<bool> setTableBorderAsync({
    String? caretRunId,
    required double width,
    required Color color,
  });
  Future<bool> setTableCellShadingAsync({
    String? caretRunId,
    Color? shading,
  });
  Future<bool> resizeTableColumnAsync({
    String? caretRunId,
    required double width,
  });
  Future<bool> autofitTableAsync({String? caretRunId});
  Future<bool> sortTableRowsAsync({String? caretRunId, required bool ascending});
  Future<bool> insertNestedTableAsync({
    String? caretRunId,
    required int rows,
    required int cols,
  });
  Future<bool> insertTableSumFieldAsync({String? caretRunId});
  Future<bool> insertImageBlockAsync(double width, double height);
  Future<bool> insertImageBytesAsync(Uint8List bytes, String mimeType);
  Future<bool> insertShapeBlockAsync(int shapeType);
  Future<bool> insertTextBoxAsync();
  Future<bool> insertWordArtAsync(String text);
  Future<bool> insertDiagramAsync({int diagramType = 0});
  Future<bool> insertChartAsync({int chartType = 0});
  Future<bool> setChartDataAsync(String shapeId, Map<String, dynamic> chartData);
  Future<bool> insertOfficeMathAsync({
    required String runId,
    required int offset,
    required String xml,
  });
  Future<bool> insertOfficeMathDisplayAsync({
    String? caretRunId,
    required String xml,
  });
  Future<bool> setOfficeMathAsync(String runId, String xml);
  Future<bool> deleteBlockAsync(String blockId);
  Future<bool> setImageSizeAsync(String imageId, double width, double height);
  Future<bool> replaceImageBytesAsync(
    String imageId,
    Uint8List bytes,
    String mimeType,
  );
  Future<bool> setImageWrapAsync(String imageId, int wrap);
  Future<bool> setImageAnchorAsync(
    String imageId,
    double x,
    double y, {
    int originX = 0,
    int originY = 0,
  });
  Future<bool> setImageTransformAsync(
    String imageId, {
    double rotationDeg = 0,
    double cropLeft = 0,
    double cropTop = 0,
    double cropRight = 0,
    double cropBottom = 0,
    double opacity = 1,
  });
  Future<bool> insertImageCaptionAsync(String imageId);
  /// Set or clear image alt text (null/empty clears) — F21.S3.
  Future<bool> setImageAltTextAsync(String imageId, String? altText);
  Future<bool> compressImageAsync(String imageId, int quality);
  Future<bool> undoEditAsync();
  Future<bool> redoEditAsync();

  List<String>? spellCheckMisspellings();
  List<SpellIssue>? spellCheckIssues();
  List<String>? grammarCheckIssues();
  List<FindMatch>? findMatches(
    String query,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
    FindFormatFilter formatFilter = FindFormatFilter.none,
  });
  Future<int?> replaceAll(
    String find,
    String replace,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
  });
  String? compareDocumentText(String otherText);
  bool setReadOnlyEnabled(bool enabled);
  /// Set or clear the password used to encrypt DOCX on save (F22.S2).
  /// Pass null or empty to clear.
  bool setEncryptionPassword(String? password);
  bool setTrackChangesEnabled(bool enabled);
  bool acceptAllRevisions();
  bool rejectAllRevisions();

  bool acceptRevisionAtCaret({String? caretRunId});
  bool rejectRevisionAtCaret({String? caretRunId});
  String? adjacentRevisionRunId(String? caretRunId, {required bool forward});
}

/// Resolve range → pixel rects via caret geometry (never pixel → range).
extension DocumentEngineSelection on DocumentEngine {
  /// Find caret geometry for [runId]/[offset], trying [hintPage] first then
  /// scanning outward. Needed when a selection spans pages and one endpoint
  /// does not live on the page being painted.
  (int, CaretGeometry)? caretPageAndGeometry(
    String runId,
    int offset, {
    int hintPage = 0,
  }) {
    final hint = hintPage < 0 ? 0 : hintPage;
    final direct = caretAtPosition(hint, runId, offset);
    if (direct != null) return (hint, direct);
    for (var delta = 1; delta < 64; delta++) {
      final earlier = hint - delta;
      if (earlier >= 0) {
        final geom = caretAtPosition(earlier, runId, offset);
        if (geom != null) return (earlier, geom);
      }
      final later = hint + delta;
      final geom = caretAtPosition(later, runId, offset);
      if (geom != null) return (later, geom);
    }
    return null;
  }

  /// Project normalized selection endpoints onto [page] for [selectionRectsOnPage].
  /// Returns null when [page] is outside the selected page span.
  static (double, double, double, double)? projectSelectionOntoPage({
    required int page,
    required int startPage,
    required CaretGeometry startGeom,
    required int endPage,
    required CaretGeometry endGeom,
  }) {
    if (page < startPage || page > endPage) return null;
    if (page == startPage && page == endPage) {
      return (startGeom.x, startGeom.y, endGeom.x, endGeom.y);
    }
    if (page == startPage) {
      // Through the bottom of this page.
      return (startGeom.x, startGeom.y, startGeom.x + 1e6, 1e9);
    }
    if (page == endPage) {
      // From the top of this page through the end caret.
      return (0, 0, endGeom.x, endGeom.y);
    }
    // Middle page fully covered by a multi-page selection.
    return (0, 0, 1e6, 1e9);
  }

  List<GlyphSelectionRect> selectionRectsForRange(int page, DocRange range) {
    final anchor = caretPageAndGeometry(
      range.anchor.runId,
      range.anchor.offset,
      hintPage: page,
    );
    final focus = caretPageAndGeometry(
      range.focus.runId,
      range.focus.offset,
      hintPage: page,
    );
    if (anchor == null || focus == null) return const [];

    final (anchorPage, anchorGeom) = anchor;
    final (focusPage, focusGeom) = focus;
    final forward = anchorPage < focusPage ||
        (anchorPage == focusPage &&
            (anchorGeom.y < focusGeom.y - 0.5 ||
                ((anchorGeom.y - focusGeom.y).abs() <= 0.5 &&
                    anchorGeom.x <= focusGeom.x)));
    final startPage = forward ? anchorPage : focusPage;
    final endPage = forward ? focusPage : anchorPage;
    final startGeom = forward ? anchorGeom : focusGeom;
    final endGeom = forward ? focusGeom : anchorGeom;

    final projected = projectSelectionOntoPage(
      page: page,
      startPage: startPage,
      startGeom: startGeom,
      endPage: endPage,
      endGeom: endGeom,
    );
    if (projected == null) return const [];
    final (startX, startY, endX, endY) = projected;
    return selectionRectsOnPage(page, startX, startY, endX, endY);
  }
}
