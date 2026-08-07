import 'dart:typed_data';

import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';
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
  DocumentProperties fetchDocumentProperties();
  bool isDocumentReadOnly();

  /// True while [page] awaits background reflow, so [hitTestPage] on it returns
  /// null for "not laid out yet" rather than "nothing here".
  bool isPageStale(int page);
  String? getLastError();

  bool newDocument();
  int openDocumentBytes(Uint8List bytes, {String? path});
  Uint8List? saveDocumentBytes();
  Uint8List? saveDocumentAsBytes(String formatExtension);
  Uint8List? exportPdfBytes();

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
  Future<bool> deleteDocRangeAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  );
  Future<bool> splitParagraphAsync(String runId, int offset);
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
  bool setCurrentPageIndex(int page);
  Future<bool> applyHeading1StyleAsync({String? caretRunId});
  Future<bool> applyNormalStyleAtAsync({String? caretRunId});
  Future<bool> applyBulletListStyleAsync({String? caretRunId});
  Future<bool> applyNumberedListStyleAsync({String? caretRunId});
  Future<bool> insertTableBlockAsync(int rows, int cols);
  Future<bool> insertImageBlockAsync(double width, double height);
  Future<bool> undoEditAsync();
  Future<bool> redoEditAsync();

  List<String>? spellCheckMisspellings();
  bool setTrackChangesEnabled(bool enabled);
  bool acceptAllRevisions();
  bool rejectAllRevisions();
}

/// Resolve range → pixel rects via caret geometry (never pixel → range).
extension DocumentEngineSelection on DocumentEngine {
  List<GlyphSelectionRect> selectionRectsForRange(int page, DocRange range) {
    final anchorGeom =
        caretAtPosition(page, range.anchor.runId, range.anchor.offset);
    final focusGeom =
        caretAtPosition(page, range.focus.runId, range.focus.offset);
    if (anchorGeom == null || focusGeom == null) return const [];
    return selectionRectsOnPage(
      page,
      anchorGeom.x,
      anchorGeom.y,
      focusGeom.x,
      focusGeom.y,
    );
  }
}
