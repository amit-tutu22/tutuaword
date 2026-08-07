import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:tutuaword/bridge/command_codec.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/bridge/wasm_interop.dart';

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

  int _pump() => callMethod(_engine, 'pump', []) as int;

  Object _invoke(String method, List<Object?> args) =>
      callMethod(_engine, method, args);

  void _drainEvents() {
    while (true) {
      final json = callMethod(_engine, 'popEvent', []);
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

  int openDocumentBytes(Uint8List bytes, {String? path}) {
    try {
      if (path != null && path.isNotEmpty) {
        _invoke('open_document_with_path', [bytes, path]);
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

  Future<bool> splitParagraphAsync(String runId, int offset) =>
      enqueueEdit(() => dispatchCommand(CommandCodec.splitParagraphAt(
            runId: runId,
            offset: offset,
          )));

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

  bool setCurrentPageIndex(int page) =>
      _enqueueNamed('set_current_page', [page]) == 0;

  Future<bool> applyHeading1StyleAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('apply_heading1', [caretRunId ?? '']));

  Future<bool> applyNormalStyleAtAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('apply_normal_style', [caretRunId ?? '']));

  Future<bool> applyBulletListStyleAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('apply_bullet_list', [caretRunId ?? '']));

  Future<bool> applyNumberedListStyleAsync({String? caretRunId}) =>
      enqueueEdit(() => _enqueueNamed('apply_numbered_list', [caretRunId ?? '']));

  Future<bool> insertTableBlockAsync(int rows, int cols) =>
      enqueueEdit(() => _enqueueNamed('insert_table', [rows, cols]));

  Future<bool> insertImageBlockAsync(double width, double height) =>
      enqueueEdit(() => _enqueueNamed('insert_image', [width, height]));

  Future<bool> undoEditAsync() => enqueueEdit(() => _enqueueNamed('undo', []));

  Future<bool> redoEditAsync() => enqueueEdit(() => _enqueueNamed('redo', []));

  List<String>? spellCheckMisspellings() {
    try {
      final text = _invoke('spell_check', []) as String;
      if (text.isEmpty) return [];
      return text.split('\n');
    } catch (_) {
      return null;
    }
  }

  bool setTrackChangesEnabled(bool enabled) =>
      _enqueueNamed('set_track_changes', [enabled]) == 0;

  bool acceptAllRevisions() =>
      _enqueueNamed('accept_all_revisions', []) == 0;

  bool rejectAllRevisions() =>
      _enqueueNamed('reject_all_revisions', []) == 0;
}
