import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/command_codec.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/find_format_filter.dart';
import 'package:tutuaword/bridge/find_match.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/bridge/spell_issue.dart';
import 'package:tutuaword/bridge/wasm_interop.dart';
import 'package:tutuaword/editor/doc_range.dart';

const Duration kWasmEditCompletionTimeout = kEditCompletionTimeout;

/// Browser-hosted engine backed by `tw-wasm` (inline executor).
class WasmEngine {
  WasmEngine._(this._engine);

  static WasmEngine? _cached;

  final Object _engine;

  static Future<WasmEngine?> load() async {
    if (_cached != null) return _cached;
    try {
      await twWasmInit();
      final engine = createWasmEngine();
      final wasm = WasmEngine._(engine);
      NativeEventRouter.instance.attachPump(wasm._pump);
      _cached = wasm;
      return wasm;
    } catch (e) {
      debugPrint('WasmEngine.load failed: $e');
      return null;
    }
  }

  int _pump() {
    final count = callMethod(_engine, 'pump', []) as int;
    _drainEvents();
    return count;
  }

  Object _invoke(String method, List<Object?> args) =>
      callMethod(_engine, method, args);

  void _drainEvents() {
    while (true) {
      final json = callMethodOrNull(_engine, 'pop_event', []);
      if (json == null) break;
      _handleEventJson(json.toString());
    }
  }

  void _handleEventJson(String json) {
    try {
      final map = jsonDecode(json) as Map<String, dynamic>;
      final eventType = map['event_type'] as int;
      final requestId = map['request_id'] as int;
      NativeEventRouter.instance.onEvent(eventType, requestId);
    } catch (_) {}
  }

  bool registerFont(
    String family,
    Uint8List data, {
    bool bold = false,
    bool italic = false,
  }) {
    if (data.isEmpty) return false;
    try {
      callMethod(_engine, 'register_font', [family, bold, italic, data]);
      return true;
    } catch (_) {
      return false;
    }
  }

  int lastRequestId() =>
      (callMethod(_engine, 'last_request_id', []) as num).toInt();

  Future<bool> awaitEditCompletion({
    Duration timeout = kWasmEditCompletionTimeout,
  }) async {
    final requestId = lastRequestId();
    if (requestId == 0) return true;
    try {
      final eventType =
          await NativeEventRouter.instance.waitFor(requestId, timeout: timeout);
      return eventType != NativeEventTypes.error;
    } on TimeoutException {
      fetchDisplayList();
      return true;
    }
  }

  Future<bool> enqueueEdit(int Function() enqueue) async {
    final code = enqueue();
    if (code != 0) return false;
    return awaitEditCompletion();
  }

  int dispatchCommandBytes(Uint8List jsonBytes) {
    try {
      callMethod(_engine, 'dispatch', [jsonBytes]);
      _drainEvents();
      return 0;
    } catch (_) {
      return -1;
    }
  }

  int dispatchCommand(Map<String, dynamic> command) {
    return dispatchCommandBytes(CommandCodec.encode(command));
  }

  int _enqueueNamed(String name, List<Object?> args) {
    try {
      callMethod(_engine, name, args);
      _drainEvents();
      return 0;
    } catch (_) {
      return -1;
    }
  }

  DisplayListData? fetchDisplayList() {
    try {
      final bytes = callMethod(_engine, 'display_list_bytes', []) as Uint8List;
      return DisplayListData(
        bytes: bytes,
        version:
            (callMethod(_engine, 'display_list_version', []) as num)
                .toInt(),
        pageWidth: (callMethod(_engine, 'display_list_page_width', [])
            as num)
            .toDouble(),
        pageHeight:
            (callMethod(_engine, 'display_list_page_height', [])
                as num)
                .toDouble(),
        pageCount:
            (callMethod(_engine, 'display_list_page_count', []) as num)
                .toInt(),
      );
    } catch (_) {
      return null;
    }
  }

  PageDisplayListData? fetchPageDisplayList(int page) {
    try {
      final bytes = callMethod(
        _engine,
        'page_display_list_bytes',
        [page],
      );
      if (bytes == null) return null;
      return PageDisplayListData(
        bytes: bytes as Uint8List,
        version: (callMethod(
          _engine,
          'page_display_list_version',
          [page],
        ) as num)
            .toInt(),
        pageWidth: (callMethod(
          _engine,
          'page_display_list_page_width',
          [page],
        ) as num)
            .toDouble(),
        pageHeight: (callMethod(
          _engine,
          'page_display_list_page_height',
          [page],
        ) as num)
            .toDouble(),
      );
    } catch (_) {
      return null;
    }
  }

  AtlasData? fetchAtlas() {
    try {
      final bytes = _invoke('atlas_bytes', []) as Uint8List;
      return AtlasData(
        generation:
            (callMethod(_engine, 'atlas_generation', []) as num)
                .toInt(),
        bytes: bytes,
        width: (callMethod(_engine, 'atlas_width', []) as num)
            .toInt(),
        height: (callMethod(_engine, 'atlas_height', []) as num)
            .toInt(),
      );
    } catch (_) {
      return null;
    }
  }

  int? fetchAtlasGeneration() {
    try {
      return (callMethod(_engine, 'atlas_generation', []) as num)
          .toInt();
    } catch (_) {
      return null;
    }
  }

  String? fetchDocumentText() {
    try {
      return _invoke('text', []) as String;
    } catch (_) {
      return null;
    }
  }

  String? fetchTextRange(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) {
    try {
      return _invoke(
        'text_in_range',
        [startRunId, startOffset, endRunId, endOffset],
      ) as String?;
    } catch (_) {
      return null;
    }
  }

  String? fetchCaretFormat(String runId) {
    try {
      return _invoke('caret_format_json', [runId]) as String?;
    } catch (_) {
      return null;
    }
  }

  String? fetchSectionFormat({String? caretRunId}) {
    try {
      return _invoke('get_section_format_json', [caretRunId ?? '']) as String?;
    } catch (_) {
      return null;
    }
  }

  String? fetchChartDataJson(String shapeId) {
    try {
      return _invoke('get_chart_data_json', [shapeId]) as String?;
    } catch (_) {
      return null;
    }
  }

  String? latestChartId() {
    try {
      return _invoke('latest_chart_id', []) as String?;
    } catch (_) {
      return null;
    }
  }

  String? fetchOfficeMathXml(String runId) {
    try {
      return _invoke('get_office_math_xml', [runId]) as String?;
    } catch (_) {
      return null;
    }
  }

  String? latestOfficeMathRunId() {
    try {
      return _invoke('latest_office_math_run_id', []) as String?;
    } catch (_) {
      return null;
    }
  }

  String? fetchImageAltText(String imageId) {
    try {
      return _invoke('image_alt_text', [imageId]) as String?;
    } catch (_) {
      return null;
    }
  }

  String? fetchDocumentOutline() {
    try {
      return _invoke('document_outline_json', []) as String?;
    } catch (_) {
      return '[]';
    }
  }

  String? fetchBookmarks() {
    try {
      return _invoke('bookmarks_json', []) as String?;
    } catch (_) {
      return '[]';
    }
  }

  String? fetchHyperlinkAt(String runId) {
    try {
      final json = _invoke('hyperlink_at', [runId]) as String?;
      if (json == null || json.isEmpty) return null;
      return json;
    } catch (_) {
      return null;
    }
  }

  String? fetchParagraphNav(String runId) {
    try {
      final json = _invoke('paragraph_nav_json', [runId]) as String?;
      if (json == null || json.isEmpty) return null;
      return json;
    } catch (_) {
      return null;
    }
  }

  String? fetchSemanticTree() {
    try {
      return _invoke('semantic_tree_json', []) as String?;
    } catch (_) {
      return '[]';
    }
  }

  String? fetchAccessibilityIssues() {
    try {
      return _invoke('accessibility_issues_json', []) as String?;
    } catch (_) {
      return '[]';
    }
  }

  String? fetchDocumentInspect() {
    try {
      return _invoke('document_inspect_json', []) as String?;
    } catch (_) {
      return '[]';
    }
  }

  bool removeInspectFindings({
    bool comments = false,
    bool metadata = false,
    bool hiddenText = false,
  }) {
    try {
      _invoke('remove_inspect_findings', [comments, metadata, hiddenText]);
      return true;
    } catch (_) {
      return false;
    }
  }

  String? fetchDigitalSignatures() {
    try {
      return _invoke('digital_signatures_json', []) as String?;
    } catch (_) {
      return '[]';
    }
  }

  String? verifyDigitalSignatures() {
    try {
      return _invoke('verify_signatures_json', []) as String?;
    } catch (_) {
      return '[]';
    }
  }

  bool signDocument({
    required String name,
    String email = '',
    String? organization,
  }) {
    try {
      _invoke('sign_document', [name, email, organization]);
      return true;
    } catch (_) {
      return false;
    }
  }

  bool clearDigitalSignatures() {
    try {
      _invoke('clear_digital_signatures', []);
      return true;
    } catch (_) {
      return false;
    }
  }

  DocumentProperties fetchDocumentProperties() {
    try {
      final json = _invoke('document_properties_json', []) as String;
      if (json.isEmpty) return DocumentProperties.empty;
      return DocumentProperties.fromJson(
        jsonDecode(json) as Map<String, dynamic>,
      );
    } catch (_) {
      return DocumentProperties.empty;
    }
  }

  bool isDocumentReadOnly() {
    try {
      return _invoke('is_read_only', []) as bool;
    } catch (_) {
      return false;
    }
  }

  bool isPageStale(int page) {
    try {
      return _invoke('is_page_stale', [page]) as bool;
    } catch (_) {
      return false;
    }
  }

  String? getLastError() {
    try {
      final err = _invoke('last_error', []) as String;
      return err.isEmpty ? null : err;
    } catch (_) {
      return null;
    }
  }

  bool newDocument() {
    try {
      _invoke('new_document', []);
      _drainEvents();
      return true;
    } catch (_) {
      return false;
    }
  }

  int openDocumentBytes(Uint8List bytes, {String? path, String? password}) {
    try {
      final pw = password ?? '';
      // Blob URLs from the web file picker have no extension — ignore them so
      // format detection falls back to magic bytes.
      final hint = (path != null &&
              path.isNotEmpty &&
              !path.startsWith('blob:') &&
              !path.startsWith('data:'))
          ? path
          : '';
      if (pw.isNotEmpty || hint.isNotEmpty) {
        try {
          _invoke('open_document_with_password', [bytes, hint, pw]);
        } catch (_) {
          if (hint.isNotEmpty) {
            _invoke('open_document_with_path', [bytes, hint]);
          } else {
            _invoke('open_document', [bytes]);
          }
        }
      } else {
        _invoke('open_document', [bytes]);
      }
      _drainEvents();
      return 0;
    } catch (_) {
      return -1;
    }
  }

  Uint8List? saveDocumentBytes() {
    try {
      return _invoke('save_document', []) as Uint8List;
    } catch (_) {
      return null;
    }
  }

  Uint8List? saveDocumentAsBytes(String formatExtension) {
    try {
      return _invoke('save_document_as', [formatExtension]) as Uint8List;
    } catch (_) {
      return null;
    }
  }

  Uint8List? exportPdfBytes() {
    try {
      return _invoke('export_pdf', []) as Uint8List;
    } catch (_) {
      return null;
    }
  }

  Uint8List? exportPdfBytesForPrint([
    PrintLayoutSettings? layout,
    DocRange? selection,
  ]) {
    final settings = layout ?? PrintLayoutSettings.defaults;
    try {
      if (selection != null && !selection.isCollapsed && selection.isValid) {
        final (start, end) = selection.normalized();
        return _invoke('export_pdf_for_print_selection', [
          start.runId,
          start.offset,
          end.runId,
          end.offset,
          settings.scaleModeCode,
          settings.scalePercent,
          settings.marginLeft,
          settings.marginRight,
          settings.marginTop,
          settings.marginBottom,
          settings.duplexCode,
          settings.effectivePagesPerSheet,
          settings.booklet ? 1 : 0,
        ]) as Uint8List;
      }
      return _invoke('export_pdf_for_print', [
        settings.scaleModeCode,
        settings.scalePercent,
        settings.marginLeft,
        settings.marginRight,
        settings.marginTop,
        settings.marginBottom,
        settings.duplexCode,
        settings.effectivePagesPerSheet,
        settings.booklet ? 1 : 0,
      ]) as Uint8List;
    } catch (_) {
      return exportPdfBytes();
    }
  }

  HitTestResult? hitTestPage(int page, double x, double y) {
    try {
      final json = _invoke('hit_test', [page, x, y]) as String?;
      if (json == null) return null;
      final map = jsonDecode(json) as Map<String, dynamic>;
      return HitTestResult(
        runId: map['run_id'] as String,
        charOffset: map['offset'] as int,
      );
    } catch (_) {
      return null;
    }
  }

  HitTestResult? fetchDocumentTailHit(int page) {
    try {
      final json = _invoke('document_tail_hit', [page]) as String?;
      if (json == null) return null;
      final map = jsonDecode(json) as Map<String, dynamic>;
      return HitTestResult(
        runId: map['run_id'] as String,
        charOffset: map['offset'] as int,
      );
    } catch (_) {
      return null;
    }
  }

  CaretGeometry? caretGeometryAt(int page, double x, double y) {
    try {
      final json = _invoke('caret_geometry', [page, x, y]) as String?;
      if (json == null) return null;
      final map = jsonDecode(json) as Map<String, dynamic>;
      return CaretGeometry(
        x: (map['x'] as num).toDouble(),
        y: (map['y'] as num).toDouble(),
        height: (map['height'] as num).toDouble(),
      );
    } catch (_) {
      return null;
    }
  }

  CaretGeometry? caretAtPosition(int page, String runId, int charOffset) {
    try {
      final json = _invoke('caret_at', [page, runId, charOffset]) as String?;
      if (json == null) return null;
      final map = jsonDecode(json) as Map<String, dynamic>;
      return CaretGeometry(
        x: (map['x'] as num).toDouble(),
        y: (map['y'] as num).toDouble(),
        height: (map['height'] as num).toDouble(),
      );
    } catch (_) {
      return null;
    }
  }

  List<GlyphSelectionRect> selectionRectsOnPage(
    int page,
    double startX,
    double startY,
    double endX,
    double endY,
  ) {
    try {
      final values = _invoke(
        'selection_rects',
        [page, startX, startY, endX, endY],
      );
      final list = (values as List<Object?>).cast<num>();
      final rects = <GlyphSelectionRect>[];
      for (var i = 0; i + 3 < list.length; i += 4) {
        rects.add(
          GlyphSelectionRect(
            x: list[i].toDouble(),
            y: list[i + 1].toDouble(),
            width: list[i + 2].toDouble(),
            height: list[i + 3].toDouble(),
          ),
        );
      }
      return rects;
    } catch (_) {
      return const [];
    }
  }

  void insertText(String runId, int offset, String text) {
    dispatchCommand(CommandCodec.insertText(
      runId: runId,
      offset: offset,
      text: text,
    ));
  }

  bool tryInsertText(String runId, int offset, String text) {
    return dispatchCommand(CommandCodec.insertText(
          runId: runId,
          offset: offset,
          text: text,
        )) ==
        0;
  }

  Future<bool> tryInsertTextAsync(String runId, int offset, String text) =>
      enqueueEdit(() => dispatchCommand(CommandCodec.insertText(
            runId: runId,
            offset: offset,
            text: text,
          )));

  Future<bool> tryPasteHtmlAsync(String runId, int offset, String html) =>
      enqueueEdit(() => _enqueueNamed('paste_html', [runId, offset, html]));

  Future<bool> tryPasteDocxAsync(String runId, int offset, Uint8List bytes) =>
      enqueueEdit(() => _enqueueNamed('paste_docx', [runId, offset, bytes]));

  Future<bool> deleteRangeAsync(String runId, int start, int end) =>
      enqueueEdit(() => dispatchCommand(CommandCodec.deleteRange(
            runId: runId,
            start: start,
            end: end,
          )));

  /// Composes DeleteRange + InsertText until WASM exposes a transactional replace.
  Future<bool> replaceRangeAsync(
    String runId,
    int start,
    int end,
    String text,
  ) async {
    if (start < end) {
      final ok = await deleteRangeAsync(runId, start, end);
      if (!ok) return false;
    }
    if (text.isEmpty) return true;
    return tryInsertTextAsync(runId, start, text);
  }

  Future<bool> deleteDocRangeAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) =>
      enqueueEdit(() => dispatchCommand(CommandCodec.deleteDocRange(
            startRunId: startRunId,
            startOffset: startOffset,
            endRunId: endRunId,
            endOffset: endOffset,
          )));

  Future<HitTestResult?> splitParagraphAsync(String runId, int offset) async {
    final ok = await enqueueEdit(() => dispatchCommand(CommandCodec.splitParagraphAt(
          runId: runId,
          offset: offset,
        )));
    if (!ok) return null;
    return _fetchLastSplitCaret();
  }

  HitTestResult? _fetchLastSplitCaret() {
    try {
      final json = callMethodOrNull(_engine, 'last_split_caret', []);
      if (json == null) return null;
      final map = jsonDecode(json.toString()) as Map<String, dynamic>;
      return HitTestResult(
        runId: map['run_id'] as String,
        charOffset: map['offset'] as int,
      );
    } catch (_) {
      return null;
    }
  }

  Future<bool> applyCharFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) async {
    final patch = jsonDecode(formatJson) as Map<String, dynamic>;
    final commands = CommandCodec.charFormatPatchCommands(
      startRunId: startRunId,
      startOffset: startOffset,
      endRunId: endRunId,
      endOffset: endOffset,
      patch: patch,
    );
    for (final command in commands) {
      final code = dispatchCommand(command);
      if (code != 0) return false;
    }
    return awaitEditCompletion();
  }

  Future<bool> applyParaFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) =>
      enqueueEdit(() => dispatchCommand(CommandCodec.setParaFormatRange(
            startRunId: startRunId,
            startOffset: startOffset,
            endRunId: endRunId,
            endOffset: endOffset,
            format: jsonDecode(formatJson) as Map<String, dynamic>,
          )));

  Future<bool> clearFormatAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) =>
      enqueueEdit(() => _enqueueNamed('clear_format', [
            startRunId,
            startOffset,
            endRunId,
            endOffset,
          ]));

  Future<bool> insertPageBreakAtAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed(
            'insert_page_break',
            [caretRunId ?? ''],
          ));

  Future<bool> insertSectionBreakAtAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed(
            'insert_section_break',
            [caretRunId ?? ''],
          ));

  Future<bool> ensureHeaderFooterAsync({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) =>
      enqueueEdit(() => _enqueueNamed(
            'ensure_header_footer',
            [caretRunId ?? '', isHeader, pageIndex],
          ));

  String? fetchHeaderFooterSeedRun({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) {
    try {
      return _invoke('header_footer_seed_run', [
        caretRunId ?? '',
        isHeader,
        pageIndex,
      ]) as String?;
    } catch (_) {
      return null;
    }
  }

  bool fetchEvenAndOddHeadersEnabled() {
    try {
      return _invoke('even_and_odd_headers_enabled', const []) as bool? ?? false;
    } catch (_) {
      return false;
    }
  }

  Future<bool> setEvenAndOddHeadersAsync({required bool enabled}) =>
      enqueueEdit(() => _enqueueNamed('set_even_and_odd_headers', [enabled]));

  bool fetchHeaderFooterLinked({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) {
    try {
      return _invoke('header_footer_linked', [
        caretRunId ?? '',
        isHeader,
        pageIndex,
      ]) as bool? ??
          false;
    } catch (_) {
      return false;
    }
  }

  Future<bool> setHeaderFooterLinkAsync({
    String? caretRunId,
    required bool isHeader,
    required bool linked,
    int pageIndex = 0,
  }) =>
      enqueueEdit(() => _enqueueNamed('set_header_footer_link', [
            caretRunId ?? '',
            isHeader,
            pageIndex,
            linked,
          ]));

  Future<bool> insertFieldAsync({
    required String runId,
    required int offset,
    required String fieldType,
    String? mergeName,
  }) =>
      enqueueEdit(() => _enqueueNamed('insert_field', [
            runId,
            offset,
            mergeName == null ? fieldType : 'if:$mergeName',
          ]));

  Future<bool> applySpellReplacementAsync({
    required int plainStart,
    required int plainEnd,
    required String replacement,
  }) async =>
      false;

  Future<Uint8List?> exportSelectionDocxAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
  }) async =>
      null;

  String? getCommentsJson() => '[]';

  Future<bool> replyToCommentAsync({
    required int commentId,
    required String bodyText,
  }) async =>
      false;

  Future<bool> resolveCommentAsync({
    required int commentId,
    required bool resolved,
  }) async =>
      false;

  Future<bool> insertFootnoteAsync({
    required String runId,
    required int offset,
  }) =>
      enqueueEdit(() => _enqueueNamed('insert_footnote', [runId, offset]));

  Future<bool> insertEndnoteAsync({
    required String runId,
    required int offset,
  }) =>
      enqueueEdit(() => _enqueueNamed('insert_endnote', [runId, offset]));

  Future<bool> insertCommentAsync({
    required String runId,
    required int offset,
    String bodyText = '',
  }) =>
      enqueueEdit(
        () => _enqueueNamed('insert_comment', [runId, offset, bodyText]),
      );

  Future<bool> insertTableOfContentsAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('insert_table_of_contents', [caretRunId ?? '']));

  Future<bool> insertTableOfFiguresAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('insert_table_of_figures', [caretRunId ?? '']));

  Future<bool> addBibliographySourceAsync({
    required String key,
    required String author,
    required String title,
    required String year,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('add_bibliography_source', [key, author, title, year]),
      );

  Future<bool> insertCitationAsync({
    required String runId,
    required int offset,
    required String sourceKey,
  }) =>
      enqueueEdit(() => _enqueueNamed('insert_citation', [runId, offset, sourceKey]));

  Future<bool> insertBibliographyAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('insert_bibliography', [caretRunId ?? '']));

  Future<bool> insertBookmarkAsync({
    required String runId,
    required int offset,
    required String name,
  }) =>
      enqueueEdit(() => _enqueueNamed('insert_bookmark', [runId, offset, name]));

  Future<bool> insertHyperlinkAsync({
    required String runId,
    required int offset,
    required String url,
    required String text,
    String? tooltip,
  }) =>
      enqueueEdit(
        () => _enqueueNamed(
          'insert_hyperlink',
          [runId, offset, url, text, tooltip ?? ''],
        ),
      );

  Future<bool> insertFormFieldAsync({
    required String runId,
    required int offset,
    required String kind,
    String? name,
    String? initialValue,
  }) =>
      enqueueEdit(
        () => _enqueueNamed(
          'insert_form_field',
          [runId, offset, kind, name ?? '', initialValue ?? ''],
        ),
      );

  Future<bool> setFormFieldValueAsync({
    required String runId,
    required String value,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('set_form_field_value', [runId, value]),
      );

  Future<bool> insertMergeFieldAsync({
    required String runId,
    required int offset,
    required String name,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('insert_merge_field', [runId, offset, name]),
      );

  Future<bool> applyMailMergeRowAsync({
    required Map<String, String> values,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('apply_mail_merge_row', [jsonEncode(values)]),
      );

  Future<bool> insertCrossReferenceAsync({
    required String runId,
    required int offset,
    required String bookmarkName,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('insert_cross_reference', [runId, offset, bookmarkName]),
      );

  Future<bool> insertIndexAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('insert_index', [caretRunId ?? '']));

  bool setCurrentPageIndex(int page) =>
      _enqueueNamed('set_current_page', [page]) == 0;

  Future<bool> applyHeading1StyleAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('apply_heading1', [caretRunId ?? '']));

  Future<bool> applyNormalStyleAtAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('apply_normal_style', [caretRunId ?? '']));

  Future<bool> applyParagraphStyleAsync({
    String? caretRunId,
    required String styleName,
  }) =>
      enqueueEdit(() => _enqueueNamed('apply_paragraph_style', [caretRunId ?? '', styleName]));

  Future<bool> applyDocumentThemeAsync({required String themeName}) =>
      enqueueEdit(() => _enqueueNamed('apply_document_theme', [themeName]));

  Future<bool> applySectionFormatJsonAsync({
    required String formatJson,
    String? caretRunId,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('apply_section_format', [formatJson, caretRunId ?? '']),
      );

  Future<bool> applyBulletListStyleAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('apply_bullet_list', [caretRunId ?? '']));

  Future<bool> applyNumberedListStyleAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('apply_numbered_list', [caretRunId ?? '']));

  Future<bool> adjustListLevelAsync({String? caretRunId, required int delta}) =>
      enqueueEdit(() => _enqueueNamed('adjust_list_level', [caretRunId ?? '', delta]));

  Future<bool> moveBlockAsync({String? caretRunId, required int delta}) =>
      enqueueEdit(() => _enqueueNamed('move_block', [caretRunId ?? '', delta]));

  Future<bool> restartNumberingAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('restart_numbering', [caretRunId ?? '']));

  Future<bool> continueNumberingAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('continue_numbering', [caretRunId ?? '']));

  Future<bool> insertTableBlockAsync(int rows, int cols, {String? caretRunId}) =>
      enqueueEdit(
        () => _enqueueNamed('insert_table', [rows, cols, caretRunId ?? '']),
      );

  Future<bool> deleteTableRowAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('delete_table_row', [caretRunId ?? '']));

  Future<bool> deleteTableColumnAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('delete_table_column', [caretRunId ?? '']));

  Future<bool> mergeTableCellsAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('merge_table_cells', [caretRunId ?? '']));

  Future<bool> splitTableCellAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('split_table_cell', [caretRunId ?? '']));

  Future<bool> setTableBorderAsync({
    String? caretRunId,
    required double width,
    required Color color,
  }) =>
      enqueueEdit(() => _enqueueNamed('set_table_border', [
            caretRunId ?? '',
            width,
            color.red,
            color.green,
            color.blue,
            (color.a * 255).round(),
          ]));

  Future<bool> setTableCellShadingAsync({
    String? caretRunId,
    Color? shading,
  }) =>
      enqueueEdit(() => _enqueueNamed('set_table_cell_shading', [
            caretRunId ?? '',
            shading == null ? -1 : shading.red,
            shading?.green ?? 0,
            shading?.blue ?? 0,
            shading == null ? 0 : (shading.a * 255).round(),
          ]));

  Future<bool> resizeTableColumnAsync({
    String? caretRunId,
    required double width,
  }) =>
      enqueueEdit(() => _enqueueNamed('resize_table_column', [
            caretRunId ?? '',
            width,
          ]));

  Future<bool> autofitTableAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('autofit_table', [caretRunId ?? '']));

  Future<bool> sortTableRowsAsync({String? caretRunId, required bool ascending}) =>
      enqueueEdit(() => _enqueueNamed('sort_table_rows', [caretRunId ?? '', ascending]));

  Future<bool> insertNestedTableAsync({
    String? caretRunId,
    required int rows,
    required int cols,
  }) =>
      enqueueEdit(() => _enqueueNamed('insert_nested_table', [caretRunId ?? '', rows, cols]));

  Future<bool> insertTableSumFieldAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('insert_table_sum_field', [caretRunId ?? '']));

  Future<bool> insertImageBlockAsync(double width, double height) =>
      enqueueEdit(() => _enqueueNamed('insert_image', [width, height]));

  Future<bool> insertShapeBlockAsync(int shapeType) =>
      enqueueEdit(() => _enqueueNamed('insert_shape', [shapeType]));

  Future<bool> insertTextBoxAsync() =>
      enqueueEdit(() => _enqueueNamed('insert_text_box', []));

  Future<bool> insertWordArtAsync(String text) =>
      enqueueEdit(() => _enqueueNamed('insert_word_art', [text]));

  Future<bool> insertDiagramAsync({int diagramType = 0}) =>
      enqueueEdit(() => _enqueueNamed('insert_diagram', [diagramType]));

  Future<bool> insertChartAsync({int chartType = 0}) =>
      enqueueEdit(() => _enqueueNamed('insert_chart', [chartType]));

  Future<bool> setChartDataAsync(String shapeId, Map<String, dynamic> chartData) =>
      enqueueEdit(
        () => _enqueueNamed('set_chart_data_json', [shapeId, jsonEncode(chartData)]),
      );

  Future<bool> insertOfficeMathAsync({
    required String runId,
    required int offset,
    required String xml,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('insert_office_math', [runId, offset, xml]),
      );

  Future<bool> insertOfficeMathDisplayAsync({
    String? caretRunId,
    required String xml,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('insert_office_math_display', [caretRunId, xml]),
      );

  Future<bool> setOfficeMathAsync(String runId, String xml) =>
      enqueueEdit(() => _enqueueNamed('set_office_math_xml', [runId, xml]));

  Future<bool> deleteBlockAsync(String blockId) =>
      enqueueEdit(() => _enqueueNamed('delete_block', [blockId]));

  Future<bool> insertImageBytesAsync(Uint8List bytes, String mimeType) =>
      enqueueEdit(() => _enqueueNamed('insert_image_bytes', [bytes, mimeType]));

  Future<bool> setImageSizeAsync(String imageId, double width, double height) =>
      enqueueEdit(() => _enqueueNamed('set_image_size', [imageId, width, height]));

  Future<bool> replaceImageBytesAsync(
    String imageId,
    Uint8List bytes,
    String mimeType,
  ) =>
      enqueueEdit(() => _enqueueNamed('replace_image_bytes', [imageId, bytes, mimeType]));

  Future<bool> setImageWrapAsync(String imageId, int wrap) =>
      enqueueEdit(() => _enqueueNamed('set_image_wrap', [imageId, wrap]));

  Future<bool> setImageAnchorAsync(
    String imageId,
    double x,
    double y, {
    int originX = 0,
    int originY = 0,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('set_image_anchor', [imageId, x, y, originX, originY]),
      );

  Future<bool> setImageTransformAsync(
    String imageId, {
    double rotationDeg = 0,
    double cropLeft = 0,
    double cropTop = 0,
    double cropRight = 0,
    double cropBottom = 0,
    double opacity = 1,
  }) =>
      enqueueEdit(
        () => _enqueueNamed('set_image_transform', [
          imageId,
          rotationDeg,
          cropLeft,
          cropTop,
          cropRight,
          cropBottom,
          opacity,
        ]),
      );

  Future<bool> insertImageCaptionAsync(String imageId) =>
      enqueueEdit(() => _enqueueNamed('insert_image_caption', [imageId]));

  Future<bool> setImageAltTextAsync(String imageId, String? altText) =>
      enqueueEdit(() => _enqueueNamed('set_image_alt_text', [imageId, altText]));

  Future<bool> compressImageAsync(String imageId, int quality) =>
      enqueueEdit(() => _enqueueNamed('compress_image', [imageId, quality]));

  Future<bool> undoEditAsync() => enqueueEdit(() => _enqueueNamed('undo', []));

  Future<bool> redoEditAsync() => enqueueEdit(() => _enqueueNamed('redo', []));

  List<String>? spellCheckMisspellings() {
    final issues = spellCheckIssues();
    return issues?.map((issue) => issue.word).toList();
  }

  List<SpellIssue>? spellCheckIssues() {
    try {
      final text = _invoke('spell_check', []) as String;
      if (text.isEmpty) return [];
      if (text.trimLeft().startsWith('[')) {
        return SpellIssue.parseJsonList(text);
      }
      return text
          .split('\n')
          .where((w) => w.isNotEmpty)
          .map((word) => SpellIssue(word: word))
          .toList();
    } catch (_) {
      return null;
    }
  }

  List<String>? grammarCheckIssues() {
    try {
      final text = _invoke('grammar_check', []) as String;
      if (text.isEmpty) return [];
      return text.split('\n');
    } catch (_) {
      return null;
    }
  }

  List<FindMatch>? findMatches(
    String query,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
    FindFormatFilter formatFilter = FindFormatFilter.none,
  }) {
    try {
      final text = _invoke('find_matches', [
            query,
            matchCase,
            useRegex,
            useWildcards,
            formatFilter.isActive ? formatFilter.encode() : '',
          ]) as String? ??
          '[]';
      final decoded = jsonDecode(text);
      if (decoded is! List) return [];
      return decoded
          .whereType<Map>()
          .map((entry) => FindMatch.fromJson(Map<String, dynamic>.from(entry)))
          .toList();
    } catch (_) {
      return null;
    }
  }

  Future<int?> replaceAll(
    String find,
    String replace,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
  }) async {
    if (find.isEmpty) return null;
    final count = findMatches(
          find,
          matchCase,
          useRegex: useRegex,
          useWildcards: useWildcards,
        )?.length ??
        0;
    if (count == 0) return 0;
    final startJson = _invoke('hit_test', [0, 72.0, 83.0]) as String?;
    final tailJson = _invoke('document_tail_hit', [0]) as String?;
    if (startJson == null || tailJson == null) return null;
    final start = jsonDecode(startJson) as Map<String, dynamic>;
    final tail = jsonDecode(tailJson) as Map<String, dynamic>;
    final ok = await enqueueEdit(() => dispatchCommand(CommandCodec.findReplace(
          startRunId: start['run_id'] as String,
          startOffset: start['char_offset'] as int,
          endRunId: tail['run_id'] as String,
          endOffset: tail['char_offset'] as int,
          find: find,
          replace: replace,
          matchCase: matchCase,
          useRegex: useRegex,
          useWildcards: useWildcards,
        )));
    return ok ? count : null;
  }

  String? compareDocumentText(String otherText) {
    try {
      return _invoke('compare_document_text', [otherText]) as String?;
    } catch (_) {
      return null;
    }
  }

  bool setTrackChangesEnabled(bool enabled) =>
      _enqueueNamed('set_track_changes', [enabled]) == 0;

  bool setReadOnlyEnabled(bool enabled) =>
      _enqueueNamed('set_read_only', [enabled]) == 0;

  bool setEncryptionPassword(String? password) =>
      _enqueueNamed('set_encryption_password', [password ?? '']) == 0;

  bool acceptAllRevisions() =>
      _enqueueNamed('accept_all_revisions', []) == 0;

  bool rejectAllRevisions() =>
      _enqueueNamed('reject_all_revisions', []) == 0;

  bool acceptRevisionAtCaret({String? caretRunId}) {
    if (caretRunId == null) return false;
    return _enqueueNamed('accept_revision_at', [caretRunId]) == 0;
  }

  bool rejectRevisionAtCaret({String? caretRunId}) {
    if (caretRunId == null) return false;
    return _enqueueNamed('reject_revision_at', [caretRunId]) == 0;
  }

  String? adjacentRevisionRunId(String? caretRunId, {required bool forward}) {
    if (caretRunId == null) return null;
    try {
      return _invoke('adjacent_revision_run', [caretRunId, forward]) as String?;
    } catch (_) {
      return null;
    }
  }
}
