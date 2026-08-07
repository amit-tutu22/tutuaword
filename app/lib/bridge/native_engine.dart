import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;
import 'package:tutuaword/bridge/command_codec.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/native_event_router.dart';

typedef TwEventCallbackNative = Void Function(Uint32, Uint64, Pointer<Uint8>, IntPtr);
typedef TwEventCallbackDart = void Function(int, int, Pointer<Uint8>, int);

typedef TwInitNative = Int32 Function(Pointer<NativeFunction<TwEventCallbackNative>>);
typedef TwInitDart = int Function(Pointer<NativeFunction<TwEventCallbackNative>>);

typedef TwDispatchNative = Int32 Function(Pointer<Uint8>, IntPtr);
typedef TwDispatchDart = int Function(Pointer<Uint8>, int);

typedef TwApplyPasteHtmlNative = Int32 Function(Pointer<Utf8>, Uint32, Pointer<Utf8>);
typedef TwApplyPasteHtmlDart = int Function(Pointer<Utf8>, int, Pointer<Utf8>);

typedef TwApplyPasteDocxNative = Int32 Function(Pointer<Utf8>, Uint32, Pointer<Uint8>, IntPtr);
typedef TwApplyPasteDocxDart = int Function(Pointer<Utf8>, int, Pointer<Uint8>, int);

typedef TwGetDisplayListNative = Int32 Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
  Pointer<Uint64>,
  Pointer<Float>,
  Pointer<Float>,
  Pointer<Uint32>,
);
typedef TwGetDisplayListDart = int Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
  Pointer<Uint64>,
  Pointer<Float>,
  Pointer<Float>,
  Pointer<Uint32>,
);

typedef TwGetPageDisplayListNative = Int32 Function(
  Uint32,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
  Pointer<Uint64>,
  Pointer<Float>,
  Pointer<Float>,
);
typedef TwGetPageDisplayListDart = int Function(
  int,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
  Pointer<Uint64>,
  Pointer<Float>,
  Pointer<Float>,
);

typedef TwGetAtlasNative = Int32 Function(
  Pointer<Uint64>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
  Pointer<Uint32>,
  Pointer<Uint32>,
);
typedef TwGetAtlasDart = int Function(
  Pointer<Uint64>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
  Pointer<Uint32>,
  Pointer<Uint32>,
);

typedef TwGetDocumentTextNative = Int32 Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwGetDocumentTextDart = int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);

typedef TwGetLastErrorNative = Int32 Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwGetLastErrorDart = int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);

typedef TwGetDocumentPropertiesJsonNative = Int32 Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwGetDocumentPropertiesJsonDart = int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);

typedef TwIsDocumentReadOnlyNative = Int32 Function();
typedef TwIsDocumentReadOnlyDart = int Function();

typedef TwIsPageStaleNative = Int32 Function(Uint32);
typedef TwIsPageStaleDart = int Function(int);

typedef TwPumpEventsNative = Int32 Function();
typedef TwPumpEventsDart = int Function();

typedef TwGetAtlasGenerationNative = Int32 Function(Pointer<Uint64>);
typedef TwGetAtlasGenerationDart = int Function(Pointer<Uint64>);

typedef TwOpenDocumentWithPathNative = Int32 Function(
  Pointer<Uint8>,
  IntPtr,
  Pointer<Utf8>,
);
typedef TwOpenDocumentWithPathDart = int Function(
  Pointer<Uint8>,
  int,
  Pointer<Utf8>,
);

typedef TwSaveDocumentNative = Int32 Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwSaveDocumentDart = int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);

typedef TwNewDocumentNative = Int32 Function();
typedef TwNewDocumentDart = int Function();

typedef TwSetCurrentPageNative = Int32 Function(Uint32);
typedef TwSetCurrentPageDart = int Function(int);

typedef TwApplyHeading1Native = Int32 Function(Pointer<Utf8>);
typedef TwApplyHeading1Dart = int Function(Pointer<Utf8>);

typedef TwApplyNormalStyleNative = Int32 Function(Pointer<Utf8>);
typedef TwApplyNormalStyleDart = int Function(Pointer<Utf8>);

typedef TwGetTextRangeNative = Int32 Function(
  Pointer<Utf8>,
  Uint32,
  Pointer<Utf8>,
  Uint32,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwGetTextRangeDart = int Function(
  Pointer<Utf8>,
  int,
  Pointer<Utf8>,
  int,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);

typedef TwGetCaretFormatNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwGetCaretFormatDart = int Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);

typedef TwClearFormatNative = Int32 Function(
  Pointer<Utf8>,
  Uint32,
  Pointer<Utf8>,
  Uint32,
);
typedef TwClearFormatDart = int Function(
  Pointer<Utf8>,
  int,
  Pointer<Utf8>,
  int,
);

typedef TwInsertPageBreakNative = Int32 Function(Pointer<Utf8>);
typedef TwInsertPageBreakDart = int Function(Pointer<Utf8>);

typedef TwApplyBulletListNative = Int32 Function(Pointer<Utf8>);
typedef TwApplyBulletListDart = int Function(Pointer<Utf8>);

typedef TwApplyNumberedListNative = Int32 Function(Pointer<Utf8>);
typedef TwApplyNumberedListDart = int Function(Pointer<Utf8>);

typedef TwInsertTableNative = Int32 Function(Uint32, Uint32);
typedef TwInsertTableDart = int Function(int, int);

typedef TwInsertImageNative = Int32 Function(Float, Float);
typedef TwInsertImageDart = int Function(double, double);

typedef TwExportPdfNative = Int32 Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwExportPdfDart = int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);

typedef TwUndoNative = Int32 Function();
typedef TwUndoDart = int Function();

typedef TwRedoNative = Int32 Function();
typedef TwRedoDart = int Function();

typedef TwSaveDocumentAsNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwSaveDocumentAsDart = int Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);

typedef TwSpellCheckDocumentNative = Int32 Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwSpellCheckDocumentDart = int Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);

typedef TwSetTrackChangesNative = Int32 Function(Int32);
typedef TwSetTrackChangesDart = int Function(int);
typedef TwAcceptAllRevisionsNative = Int32 Function();
typedef TwAcceptAllRevisionsDart = int Function();
typedef TwRejectAllRevisionsNative = Int32 Function();
typedef TwRejectAllRevisionsDart = int Function();

typedef TwHitTestNative = Int32 Function(Uint32, Float, Float, Pointer<Utf8>, IntPtr, Pointer<Uint32>);
typedef TwHitTestDart = int Function(int, double, double, Pointer<Utf8>, int, Pointer<Uint32>);

typedef TwDocumentTailHitNative = Int32 Function(Uint32, Pointer<Utf8>, IntPtr, Pointer<Uint32>);
typedef TwDocumentTailHitDart = int Function(int, Pointer<Utf8>, int, Pointer<Uint32>);

typedef TwCaretGeometryNative = Int32 Function(Uint32, Float, Float, Pointer<Float>, Pointer<Float>, Pointer<Float>);
typedef TwCaretGeometryDart = int Function(int, double, double, Pointer<Float>, Pointer<Float>, Pointer<Float>);

typedef TwCaretAtPositionNative = Int32 Function(
  Uint32,
  Pointer<Utf8>,
  Uint32,
  Pointer<Float>,
  Pointer<Float>,
  Pointer<Float>,
);
typedef TwCaretAtPositionDart = int Function(
  int,
  Pointer<Utf8>,
  int,
  Pointer<Float>,
  Pointer<Float>,
  Pointer<Float>,
);

typedef TwSelectionRectsNative = Int32 Function(Uint32, Float, Float, Float, Float, Pointer<Float>, Uint32, Pointer<Uint32>);
typedef TwSelectionRectsDart = int Function(int, double, double, double, double, Pointer<Float>, int, Pointer<Uint32>);

typedef TwLastRequestIdNative = Uint64 Function();
typedef TwLastRequestIdDart = int Function();

typedef TwFreeBufferNative = Void Function(Pointer<Uint8>, IntPtr);
typedef TwFreeBufferDart = void Function(Pointer<Uint8>, int);

typedef TwShutdownNative = Void Function();
typedef TwShutdownDart = void Function();

/// Budget for an edit's worker round-trip. Typing never blocks on this — the
/// caret advances optimistically and the repaint arrives with the event — so
/// this only bounds how long a dropped or coalesced event can stall an
/// operation that genuinely needs settled layout before the sync fallback.
const Duration kEditCompletionTimeout = Duration(milliseconds: 1500);

class NativeEngine {
  NativeEngine._(this._lib);

  static NativeEngine? _cached;
  static NativeCallable<TwEventCallbackNative>? _eventCallable;
  static Future<void>? _shutdownInFlight;

  final DynamicLibrary _lib;
  late final TwDispatchDart dispatch;
  late final TwApplyPasteHtmlDart applyPasteHtml;
  late final TwApplyPasteDocxDart applyPasteDocx;
  late final TwGetDisplayListDart getDisplayList;
  late final TwGetPageDisplayListDart getPageDisplayList;
  late final TwGetAtlasDart getAtlas;
  late final TwGetDocumentTextDart getDocumentText;
  late final TwGetLastErrorDart getLastErrorNative;
  late final TwGetDocumentPropertiesJsonDart getDocumentPropertiesJson;
  late final TwIsDocumentReadOnlyDart isDocumentReadOnlyNative;

  /// Absent in engine builds that predate `tw_is_page_stale`; a missing symbol
  /// degrades to "never stale", which is the pre-background-reflow behaviour.
  TwIsPageStaleDart? isPageStaleNative;
  TwPumpEventsDart? pumpEventsNative;
  TwGetAtlasGenerationDart? getAtlasGenerationNative;
  late final TwGetTextRangeDart getTextRange;
  late final TwGetCaretFormatDart getCaretFormat;
  late final TwClearFormatDart clearFormatNative;
  late final TwInsertPageBreakDart insertPageBreak;
  late final TwOpenDocumentWithPathDart openDocumentWithPath;
  late final TwNewDocumentDart newDocumentNative;
  late final TwSaveDocumentDart saveDocument;
  late final TwSetCurrentPageDart setCurrentPage;
  late final TwApplyHeading1Dart applyHeading1;
  late final TwApplyNormalStyleDart applyNormalStyle;
  late final TwApplyBulletListDart applyBulletList;
  late final TwApplyNumberedListDart applyNumberedList;
  late final TwInsertTableDart insertTable;
  late final TwInsertImageDart insertImage;
  late final TwExportPdfDart exportPdf;
  late final TwUndoDart undo;
  late final TwRedoDart redo;
  late final TwSaveDocumentAsDart saveDocumentAs;
  late final TwSpellCheckDocumentDart spellCheckDocument;
  late final TwSetTrackChangesDart setTrackChanges;
  late final TwAcceptAllRevisionsDart acceptAllRevisionsNative;
  late final TwRejectAllRevisionsDart rejectAllRevisionsNative;
  late final TwHitTestDart hitTest;
  late final TwDocumentTailHitDart documentTailHit;
  late final TwLastRequestIdDart lastRequestIdNative;
  late final TwCaretGeometryDart caretGeometry;
  late final TwCaretAtPositionDart caretAtPositionNative;
  late final TwSelectionRectsDart selectionRects;
  late final TwFreeBufferDart freeBuffer;

  static NativeEngine? load() {
    if (_cached != null) return _cached;
    try {
      final lib = _openLibrary();
      final engine = NativeEngine._(lib);
      _eventCallable ??= NativeCallable<TwEventCallbackNative>.listener(_eventCallback);
      lib.lookupFunction<TwInitNative, TwInitDart>('tw_init')(
        _eventCallable!.nativeFunction,
      );
      engine.dispatch = lib.lookupFunction<TwDispatchNative, TwDispatchDart>('tw_dispatch');
      engine.applyPasteHtml =
          lib.lookupFunction<TwApplyPasteHtmlNative, TwApplyPasteHtmlDart>('tw_apply_paste_html');
      engine.applyPasteDocx =
          lib.lookupFunction<TwApplyPasteDocxNative, TwApplyPasteDocxDart>('tw_apply_paste_docx');
      engine.getDisplayList =
          lib.lookupFunction<TwGetDisplayListNative, TwGetDisplayListDart>('tw_get_display_list');
      engine.getPageDisplayList = lib.lookupFunction<TwGetPageDisplayListNative,
          TwGetPageDisplayListDart>('tw_get_page_display_list');
      engine.getAtlas =
          lib.lookupFunction<TwGetAtlasNative, TwGetAtlasDart>('tw_get_atlas');
      engine.getDocumentText =
          lib.lookupFunction<TwGetDocumentTextNative, TwGetDocumentTextDart>('tw_get_document_text');
      engine.getLastErrorNative =
          lib.lookupFunction<TwGetLastErrorNative, TwGetLastErrorDart>('tw_get_last_error');
      engine.getDocumentPropertiesJson = lib.lookupFunction<TwGetDocumentPropertiesJsonNative,
          TwGetDocumentPropertiesJsonDart>('tw_get_document_properties_json');
      engine.isDocumentReadOnlyNative =
          lib.lookupFunction<TwIsDocumentReadOnlyNative, TwIsDocumentReadOnlyDart>(
              'tw_is_document_read_only');
      // Optional exports: an older engine build simply lacks them, and each
      // has a defined degraded behaviour rather than failing the whole load.
      try {
        engine.isPageStaleNative =
            lib.lookupFunction<TwIsPageStaleNative, TwIsPageStaleDart>('tw_is_page_stale');
      } on ArgumentError {
        engine.isPageStaleNative = null;
      }
      try {
        engine.pumpEventsNative =
            lib.lookupFunction<TwPumpEventsNative, TwPumpEventsDart>('tw_pump_events');
      } on ArgumentError {
        engine.pumpEventsNative = null;
      }
      try {
        engine.getAtlasGenerationNative =
            lib.lookupFunction<TwGetAtlasGenerationNative, TwGetAtlasGenerationDart>(
                'tw_get_atlas_generation');
      } on ArgumentError {
        engine.getAtlasGenerationNative = null;
      }
      engine.getTextRange =
          lib.lookupFunction<TwGetTextRangeNative, TwGetTextRangeDart>('tw_get_text_range');
      engine.getCaretFormat = lib.lookupFunction<TwGetCaretFormatNative, TwGetCaretFormatDart>(
          'tw_get_caret_format');
      engine.clearFormatNative =
          lib.lookupFunction<TwClearFormatNative, TwClearFormatDart>('tw_clear_format');
      engine.insertPageBreak = lib.lookupFunction<TwInsertPageBreakNative, TwInsertPageBreakDart>(
          'tw_insert_page_break');
      engine.openDocumentWithPath = lib.lookupFunction<TwOpenDocumentWithPathNative,
          TwOpenDocumentWithPathDart>('tw_open_document_with_path');
      engine.newDocumentNative =
          lib.lookupFunction<TwNewDocumentNative, TwNewDocumentDart>('tw_new_document');
      engine.saveDocument =
          lib.lookupFunction<TwSaveDocumentNative, TwSaveDocumentDart>('tw_save_document');
      engine.setCurrentPage =
          lib.lookupFunction<TwSetCurrentPageNative, TwSetCurrentPageDart>('tw_set_current_page');
      engine.applyHeading1 =
          lib.lookupFunction<TwApplyHeading1Native, TwApplyHeading1Dart>('tw_apply_heading1');
      engine.applyNormalStyle = lib.lookupFunction<TwApplyNormalStyleNative, TwApplyNormalStyleDart>(
          'tw_apply_normal_style');
      engine.applyBulletList = lib.lookupFunction<TwApplyBulletListNative, TwApplyBulletListDart>(
          'tw_apply_bullet_list');
      engine.applyNumberedList = lib.lookupFunction<TwApplyNumberedListNative, TwApplyNumberedListDart>(
          'tw_apply_numbered_list');
      engine.insertTable =
          lib.lookupFunction<TwInsertTableNative, TwInsertTableDart>('tw_insert_table');
      engine.insertImage =
          lib.lookupFunction<TwInsertImageNative, TwInsertImageDart>('tw_insert_image');
      engine.exportPdf =
          lib.lookupFunction<TwExportPdfNative, TwExportPdfDart>('tw_export_pdf');
      engine.undo = lib.lookupFunction<TwUndoNative, TwUndoDart>('tw_undo');
      engine.redo = lib.lookupFunction<TwRedoNative, TwRedoDart>('tw_redo');
      engine.saveDocumentAs = lib.lookupFunction<TwSaveDocumentAsNative, TwSaveDocumentAsDart>(
          'tw_save_document_as');
      engine.spellCheckDocument = lib.lookupFunction<TwSpellCheckDocumentNative,
          TwSpellCheckDocumentDart>('tw_spell_check_document');
      engine.setTrackChanges =
          lib.lookupFunction<TwSetTrackChangesNative, TwSetTrackChangesDart>('tw_set_track_changes');
      engine.acceptAllRevisionsNative = lib.lookupFunction<TwAcceptAllRevisionsNative,
          TwAcceptAllRevisionsDart>('tw_accept_all_revisions');
      engine.rejectAllRevisionsNative = lib.lookupFunction<TwRejectAllRevisionsNative,
          TwRejectAllRevisionsDart>('tw_reject_all_revisions');
      engine.hitTest = lib.lookupFunction<TwHitTestNative, TwHitTestDart>('tw_hit_test');
      engine.documentTailHit =
          lib.lookupFunction<TwDocumentTailHitNative, TwDocumentTailHitDart>('tw_document_tail_hit');
      engine.lastRequestIdNative =
          lib.lookupFunction<TwLastRequestIdNative, TwLastRequestIdDart>('tw_last_request_id');
      engine.caretGeometry =
          lib.lookupFunction<TwCaretGeometryNative, TwCaretGeometryDart>('tw_caret_geometry');
      engine.caretAtPositionNative = lib.lookupFunction<TwCaretAtPositionNative, TwCaretAtPositionDart>(
          'tw_caret_at_position');
      engine.selectionRects =
          lib.lookupFunction<TwSelectionRectsNative, TwSelectionRectsDart>('tw_selection_rects');
      engine.freeBuffer = lib.lookupFunction<TwFreeBufferNative, TwFreeBufferDart>('tw_free_buffer');
      final pump = engine.pumpEventsNative;
      if (pump != null) NativeEventRouter.instance.attachPump(pump);
      _cached = engine;
      return engine;
    } catch (_) {
      return null;
    }
  }

  /// Release the native session — for tests and app shutdown.
  static void shutdown() {
    // ignore: discarded_futures
    shutdownAsync();
  }

  static Future<void> shutdownAsync() {
    return _shutdownInFlight ??= _shutdownImpl().whenComplete(() {
      _shutdownInFlight = null;
    });
  }

  static Future<void> _shutdownImpl() async {
    final engine = _cached;
    if (engine == null) return;
    engine._lib.lookupFunction<TwShutdownNative, TwShutdownDart>('tw_shutdown')();
    _cached = null;
    NativeEventRouter.instance.reset();
    NativeEventRouter.instance.detachPump();
    // Keep _eventCallable alive until process exit — closing it races worker callbacks.
  }

  static DynamicLibrary _openLibrary() {
    if (Platform.isMacOS) {
      final exeDir = p.dirname(Platform.resolvedExecutable);
      for (final path in [
        p.normalize(p.join(exeDir, '../Frameworks/libtw_ffi.dylib')),
        p.normalize(p.join(exeDir, 'libtw_ffi.dylib')),
        'libtw_ffi.dylib',
        '../target/release/libtw_ffi.dylib',
        '../../target/release/libtw_ffi.dylib',
        '../../../target/release/libtw_ffi.dylib',
      ]) {
        try {
          return DynamicLibrary.open(path);
        } catch (_) {}
      }
      throw StateError('libtw_ffi.dylib not found');
    }
    if (Platform.isLinux) {
      return DynamicLibrary.open('libtw_ffi.so');
    }
    if (Platform.isWindows) {
      return DynamicLibrary.open('tw_ffi.dll');
    }
    throw UnsupportedError('Platform not supported');
  }

  /// [payload] repeats the type and request id, but only stays alive for the
  /// duration of the native call — and this callback is deferred to a later turn
  /// of the event loop, so it must correlate purely on the by-value arguments.
  static void _eventCallback(
    int eventType,
    int requestId,
    Pointer<Uint8> payload,
    int payloadLen,
  ) {
    NativeEventRouter.instance.onEvent(eventType, requestId);
  }
}

class DisplayListData {
  DisplayListData({
    required this.bytes,
    required this.version,
    required this.pageWidth,
    required this.pageHeight,
    required this.pageCount,
  });

  final Uint8List bytes;
  final int version;
  final double pageWidth;
  final double pageHeight;
  final int pageCount;
}

class PageDisplayListData {
  PageDisplayListData({
    required this.bytes,
    required this.version,
    required this.pageWidth,
    required this.pageHeight,
  });

  final Uint8List bytes;
  final int version;
  final double pageWidth;
  final double pageHeight;
}

class AtlasData {
  AtlasData({
    required this.generation,
    required this.bytes,
    required this.width,
    required this.height,
  });

  final int generation;
  final Uint8List bytes;
  final int width;
  final int height;
}

class HitTestResult {
  HitTestResult({required this.runId, required this.charOffset});

  final String runId;
  final int charOffset;
}

class CaretGeometry {
  CaretGeometry({required this.x, required this.y, required this.height});

  final double x;
  final double y;
  final double height;
}

class GlyphSelectionRect {
  GlyphSelectionRect({required this.x, required this.y, required this.width, required this.height});

  final double x;
  final double y;
  final double width;
  final double height;
}

extension NativeEngineOps on NativeEngine {
  int lastRequestId() => lastRequestIdNative();

  /// Enqueue a JSON [`Command`] via `tw_dispatch` (R2.5).
  int dispatchCommandBytes(Uint8List jsonBytes) {
    final ptr = calloc<Uint8>(jsonBytes.length);
    try {
      ptr.asTypedList(jsonBytes.length).setAll(0, jsonBytes);
      return dispatch(ptr, jsonBytes.length);
    } finally {
      calloc.free(ptr);
    }
  }

  int dispatchCommand(Map<String, dynamic> command) {
    return dispatchCommandBytes(CommandCodec.encode(command));
  }

  int _dispatchCommands(List<Map<String, dynamic>> commands) {
    for (final command in commands) {
      final code = dispatchCommand(command);
      if (code != 0) return code;
    }
    return 0;
  }

  /// Await the worker response for the most recently enqueued edit (R1.4).
  Future<bool> awaitEditCompletion({Duration timeout = kEditCompletionTimeout}) async {
    final requestId = lastRequestId();
    if (requestId == 0) return true;
    try {
      final eventType = await NativeEventRouter.instance.waitFor(requestId, timeout: timeout);
      return eventType != NativeEventTypes.error;
    } on TimeoutException {
      assert(() {
        debugPrint(
          'awaitEditCompletion: timed out waiting for requestId=$requestId; '
          'falling back to sync fetchDisplayList',
        );
        return true;
      }());
      fetchDisplayList();
      return true;
    }
  }

  /// Enqueue an edit FFI call, then await its correlated worker event.
  Future<bool> enqueueEdit(int Function() ffiCall) async {
    final code = ffiCall();
    if (code != 0) return false;
    return awaitEditCompletion();
  }

  DisplayListData? fetchDisplayList() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    final outVersion = calloc<Uint64>();
    final outWidth = calloc<Float>();
    final outHeight = calloc<Float>();
    final outPageCount = calloc<Uint32>();

    try {
      final result = getDisplayList(
        outPtr,
        outLen,
        outVersion,
        outWidth,
        outHeight,
        outPageCount,
      );
      if (result != 0) return null;

      final len = outLen.value;
      final ptr = outPtr.value;
      Uint8List bytes;
      if (ptr == nullptr || len == 0) {
        bytes = Uint8List(0);
      } else {
        bytes = ptr.asTypedList(len).sublist(0);
        freeBuffer(ptr, len);
      }

      return DisplayListData(
        bytes: bytes,
        version: outVersion.value,
        pageWidth: outWidth.value,
        pageHeight: outHeight.value,
        pageCount: outPageCount.value,
      );
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
      calloc.free(outVersion);
      calloc.free(outWidth);
      calloc.free(outHeight);
      calloc.free(outPageCount);
    }
  }

  /// Display list for one page, without changing the session's current page.
  PageDisplayListData? fetchPageDisplayList(int page) {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    final outVersion = calloc<Uint64>();
    final outWidth = calloc<Float>();
    final outHeight = calloc<Float>();

    try {
      final result = getPageDisplayList(
        page,
        outPtr,
        outLen,
        outVersion,
        outWidth,
        outHeight,
      );
      if (result != 0) return null;

      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) {
        return PageDisplayListData(
          bytes: Uint8List(0),
          version: outVersion.value,
          pageWidth: outWidth.value,
          pageHeight: outHeight.value,
        );
      }
      final bytes = ptr.asTypedList(len).sublist(0);
      freeBuffer(ptr, len);
      return PageDisplayListData(
        bytes: bytes,
        version: outVersion.value,
        pageWidth: outWidth.value,
        pageHeight: outHeight.value,
      );
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
      calloc.free(outVersion);
      calloc.free(outWidth);
      calloc.free(outHeight);
    }
  }

  /// Session glyph atlas, versioned independently from page display lists.
  AtlasData? fetchAtlas() {
    final outGeneration = calloc<Uint64>();
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    final outWidth = calloc<Uint32>();
    final outHeight = calloc<Uint32>();

    try {
      final result = getAtlas(outGeneration, outPtr, outLen, outWidth, outHeight);
      if (result != 0) return null;

      final len = outLen.value;
      final ptr = outPtr.value;
      Uint8List bytes;
      if (ptr == nullptr || len == 0) {
        bytes = Uint8List(0);
      } else {
        bytes = ptr.asTypedList(len).sublist(0);
        freeBuffer(ptr, len);
      }

      return AtlasData(
        generation: outGeneration.value,
        bytes: bytes,
        width: outWidth.value,
        height: outHeight.value,
      );
    } finally {
      calloc.free(outGeneration);
      calloc.free(outPtr);
      calloc.free(outLen);
      calloc.free(outWidth);
      calloc.free(outHeight);
    }
  }

  String? fetchDocumentText() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = getDocumentText(outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return '';
      final text = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return text;
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  String? fetchTextRange(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) {
    final startPtr = startRunId.toNativeUtf8();
    final endPtr = endRunId.toNativeUtf8();
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = getTextRange(
        startPtr,
        startOffset,
        endPtr,
        endOffset,
        outPtr,
        outLen,
      );
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return '';
      final text = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return text;
    } finally {
      calloc.free(startPtr);
      calloc.free(endPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  String? fetchCaretFormat(String runId) {
    final runPtr = runId.toNativeUtf8();
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = getCaretFormat(runPtr, outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final json = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return json;
    } finally {
      calloc.free(runPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  bool clearFormat(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) {
    final startPtr = startRunId.toNativeUtf8();
    final endPtr = endRunId.toNativeUtf8();
    try {
      return clearFormatNative(startPtr, startOffset, endPtr, endOffset) == 0;
    } finally {
      calloc.free(startPtr);
      calloc.free(endPtr);
    }
  }

  bool insertPageBreakAt({String? caretRunId}) {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return insertPageBreak(ptr) == 0;
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  bool newDocument() => newDocumentNative() == 0;

  /// Blocks the calling isolate: `tw_open_document_with_path` enqueues the open
  /// and then parks on `wait_for_open` (30 s cap) before returning, so there is
  /// no request id for Dart to correlate against the `DocumentOpened` event.
  /// Making this awaitable needs the export split into an enqueue that returns
  /// the request id plus a result getter (Rust-side change).
  int openDocumentBytes(Uint8List bytes, {String? path}) {
    final ptr = calloc<Uint8>(bytes.length);
    final pathPtr = path?.toNativeUtf8();
    try {
      ptr.asTypedList(bytes.length).setAll(0, bytes);
      return openDocumentWithPath(
        ptr,
        bytes.length,
        pathPtr ?? nullptr.cast<Utf8>(),
      );
    } finally {
      if (pathPtr != null) calloc.free(pathPtr);
      calloc.free(ptr);
    }
  }

  String? getLastError() => _readNativeString(getLastErrorNative);

  DocumentProperties fetchDocumentProperties() {
    final json = _readNativeString(getDocumentPropertiesJson);
    if (json == null || json.isEmpty) return DocumentProperties.empty;
    try {
      return DocumentProperties.fromJson(jsonDecode(json) as Map<String, dynamic>);
    } catch (_) {
      return DocumentProperties.empty;
    }
  }

  bool isDocumentReadOnly() => isDocumentReadOnlyNative() == 1;

  /// True while [page] still carries pre-edit geometry because the background
  /// forward reflow has not reached it. Hit tests on such a page return no
  /// result, which callers must not confuse with an empty page.
  bool isPageStale(int page) => (isPageStaleNative?.call(page) ?? 0) == 1;

  /// Atlas generation without copying the pixel buffer. Null when the engine
  /// build lacks the query, which forces callers back to a full fetch.
  int? fetchAtlasGeneration() {
    final query = getAtlasGenerationNative;
    if (query == null) return null;
    final out = calloc<Uint64>();
    try {
      return query(out) == 0 ? out.value : null;
    } finally {
      calloc.free(out);
    }
  }

  String? _readNativeString(int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>) reader) {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      if (reader(outPtr, outLen) != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final bytes = ptr.asTypedList(len).sublist(0);
      freeBuffer(ptr, len);
      return utf8.decode(bytes);
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  /// Blocks the calling isolate — see [openDocumentBytes]; `tw_save_document`
  /// returns the serialized bytes only after `wait_for_document_saved`.
  Uint8List? saveDocumentBytes() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = saveDocument(outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return Uint8List(0);
      final bytes = ptr.asTypedList(len).sublist(0);
      freeBuffer(ptr, len);
      return bytes;
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  void insertText(String runId, int offset, String text) {
    final code = dispatchCommand(CommandCodec.insertText(
      runId: runId,
      offset: offset,
      text: text,
    ));
    if (code != 0) {
      throw StateError('insertText failed for run $runId at $offset');
    }
  }

  /// Fire-and-forget insert; use [tryInsertTextAsync] to await layout.
  bool tryInsertText(String runId, int offset, String text) {
    return dispatchCommand(CommandCodec.insertText(
          runId: runId,
          offset: offset,
          text: text,
        )) ==
        0;
  }

  Future<bool> tryInsertTextAsync(String runId, int offset, String text) async {
    return enqueueEdit(() => dispatchCommand(CommandCodec.insertText(
          runId: runId,
          offset: offset,
          text: text,
        )));
  }

  Future<bool> tryPasteHtmlAsync(String runId, int offset, String html) async {
    final runPtr = runId.toNativeUtf8();
    final htmlPtr = html.toNativeUtf8();
    try {
      return enqueueEdit(() => applyPasteHtml(runPtr, offset, htmlPtr));
    } finally {
      calloc.free(runPtr);
      calloc.free(htmlPtr);
    }
  }

  Future<bool> tryPasteDocxAsync(String runId, int offset, Uint8List bytes) async {
    final runPtr = runId.toNativeUtf8();
    final dataPtr = calloc<Uint8>(bytes.length);
    try {
      dataPtr.asTypedList(bytes.length).setAll(0, bytes);
      return enqueueEdit(() => applyPasteDocx(runPtr, offset, dataPtr, bytes.length));
    } finally {
      calloc.free(runPtr);
      calloc.free(dataPtr);
    }
  }

  bool tryPasteHtml(String runId, int offset, String html) {
    final runPtr = runId.toNativeUtf8();
    final htmlPtr = html.toNativeUtf8();
    try {
      return applyPasteHtml(runPtr, offset, htmlPtr) == 0;
    } finally {
      calloc.free(runPtr);
      calloc.free(htmlPtr);
    }
  }

  bool tryPasteDocx(String runId, int offset, Uint8List bytes) {
    final runPtr = runId.toNativeUtf8();
    final dataPtr = calloc<Uint8>(bytes.length);
    try {
      dataPtr.asTypedList(bytes.length).setAll(0, bytes);
      return applyPasteDocx(runPtr, offset, dataPtr, bytes.length) == 0;
    } finally {
      calloc.free(runPtr);
      calloc.free(dataPtr);
    }
  }

  /// Apply a partial CharFormat JSON delta over a document range.
  bool applyCharFormatJson({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) {
    final patch = jsonDecode(formatJson) as Map<String, dynamic>;
    return _dispatchCommands(CommandCodec.charFormatPatchCommands(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
          patch: patch,
        )) ==
        0;
  }

  /// Apply a partial ParaFormat JSON delta over paragraphs touched by the range.
  bool applyParaFormatJson({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) {
    final format = jsonDecode(formatJson) as Map<String, dynamic>;
    return dispatchCommand(CommandCodec.setParaFormatRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
          format: format,
        )) ==
        0;
  }

  Future<bool> deleteRangeAsync(String runId, int start, int end) async {
    return enqueueEdit(() => dispatchCommand(CommandCodec.deleteRange(
          runId: runId,
          start: start,
          end: end,
        )));
  }

  Future<bool> deleteDocRangeAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) async {
    return enqueueEdit(() => dispatchCommand(CommandCodec.deleteDocRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
        )));
  }

  Future<bool> splitParagraphAsync(String runId, int offset) async {
    return enqueueEdit(() => dispatchCommand(CommandCodec.splitParagraphAt(
          runId: runId,
          offset: offset,
        )));
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
    return enqueueEdit(() => _dispatchCommands(commands));
  }

  Future<bool> applyParaFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) async {
    final format = jsonDecode(formatJson) as Map<String, dynamic>;
    return enqueueEdit(() => dispatchCommand(CommandCodec.setParaFormatRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
          format: format,
        )));
  }

  Future<bool> clearFormatAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) async {
    final startPtr = startRunId.toNativeUtf8();
    final endPtr = endRunId.toNativeUtf8();
    try {
      return enqueueEdit(() => clearFormatNative(startPtr, startOffset, endPtr, endOffset));
    } finally {
      calloc.free(startPtr);
      calloc.free(endPtr);
    }
  }

  Future<bool> insertPageBreakAtAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => insertPageBreak(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> setCurrentPageIndexAsync(int page) => enqueueEdit(() => setCurrentPage(page));

  Future<bool> applyHeading1StyleAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => applyHeading1(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> applyNormalStyleAtAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => applyNormalStyle(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> applyBulletListStyleAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => applyBulletList(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> applyNumberedListStyleAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => applyNumberedList(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> insertTableBlockAsync(int rows, int cols) =>
      enqueueEdit(() => insertTable(rows, cols));

  Future<bool> insertImageBlockAsync(double width, double height) =>
      enqueueEdit(() => insertImage(width, height));

  Future<bool> undoEditAsync() => enqueueEdit(undo);

  Future<bool> redoEditAsync() => enqueueEdit(redo);

  bool deleteRange(String runId, int start, int end) {
    return dispatchCommand(CommandCodec.deleteRange(
          runId: runId,
          start: start,
          end: end,
        )) ==
        0;
  }

  bool deleteDocRange(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) {
    return dispatchCommand(CommandCodec.deleteDocRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
        )) ==
        0;
  }

  bool splitParagraphAt(String runId, int offset) {
    return dispatchCommand(CommandCodec.splitParagraphAt(
          runId: runId,
          offset: offset,
        )) ==
        0;
  }

  bool setCurrentPageIndex(int page) => setCurrentPage(page) == 0;

  bool applyHeading1Style({String? caretRunId}) {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return applyHeading1(ptr) == 0;
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  bool applyNormalStyleAt({String? caretRunId}) {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return applyNormalStyle(ptr) == 0;
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  bool applyBulletListStyle({String? caretRunId}) {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return applyBulletList(ptr) == 0;
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  bool applyNumberedListStyle({String? caretRunId}) {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return applyNumberedList(ptr) == 0;
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  bool insertTableBlock(int rows, int cols) => insertTable(rows, cols) == 0;

  bool insertImageBlock(double width, double height) => insertImage(width, height) == 0;

  bool undoEdit() => undo() == 0;

  bool redoEdit() => redo() == 0;

  Uint8List? exportPdfBytes() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = exportPdf(outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return Uint8List(0);
      final bytes = ptr.asTypedList(len).sublist(0);
      freeBuffer(ptr, len);
      return bytes;
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  /// Blocks the calling isolate — see [openDocumentBytes].
  Uint8List? saveDocumentAsBytes(String formatExtension) {
    final formatPtr = formatExtension.toNativeUtf8();
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = saveDocumentAs(formatPtr, outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return Uint8List(0);
      final bytes = ptr.asTypedList(len).sublist(0);
      freeBuffer(ptr, len);
      return bytes;
    } finally {
      calloc.free(formatPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  /// Blocks the calling isolate — see [openDocumentBytes]; the whole-document
  /// spell pass runs before `tw_spell_check_document` returns.
  List<String>? spellCheckMisspellings() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = spellCheckDocument(outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return [];
      final text = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      if (text.isEmpty) return [];
      return text.split('\n');
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  bool setTrackChangesEnabled(bool enabled) => setTrackChanges(enabled ? 1 : 0) == 0;

  bool acceptAllRevisions() => acceptAllRevisionsNative() == 0;

  bool rejectAllRevisions() => rejectAllRevisionsNative() == 0;

  HitTestResult? hitTestPage(int page, double x, double y) {
    final runIdBuf = calloc<Uint8>(64);
    final offsetOut = calloc<Uint32>();
    try {
      final result = hitTest(page, x, y, runIdBuf.cast<Utf8>(), 64, offsetOut);
      if (result != 0) return null;
      final runId = runIdBuf.cast<Utf8>().toDartString();
      return HitTestResult(runId: runId, charOffset: offsetOut.value);
    } finally {
      calloc.free(runIdBuf);
      calloc.free(offsetOut);
    }
  }

  HitTestResult? fetchDocumentTailHit(int page) {
    final runIdBuf = calloc<Uint8>(64);
    final offsetOut = calloc<Uint32>();
    try {
      final result = documentTailHit(page, runIdBuf.cast<Utf8>(), 64, offsetOut);
      if (result != 0) return null;
      final runId = runIdBuf.cast<Utf8>().toDartString();
      return HitTestResult(runId: runId, charOffset: offsetOut.value);
    } finally {
      calloc.free(runIdBuf);
      calloc.free(offsetOut);
    }
  }

  CaretGeometry? caretGeometryAt(int page, double x, double y) {
    final outX = calloc<Float>();
    final outY = calloc<Float>();
    final outH = calloc<Float>();
    try {
      final result = caretGeometry(page, x, y, outX, outY, outH);
      if (result != 0) return null;
      return CaretGeometry(x: outX.value, y: outY.value, height: outH.value);
    } finally {
      calloc.free(outX);
      calloc.free(outY);
      calloc.free(outH);
    }
  }

  CaretGeometry? caretAtPosition(int page, String runId, int charOffset) {
    final runPtr = runId.toNativeUtf8();
    final outX = calloc<Float>();
    final outY = calloc<Float>();
    final outH = calloc<Float>();
    try {
      final result = caretAtPositionNative(page, runPtr, charOffset, outX, outY, outH);
      if (result != 0) return null;
      return CaretGeometry(x: outX.value, y: outY.value, height: outH.value);
    } finally {
      calloc.free(runPtr);
      calloc.free(outX);
      calloc.free(outY);
      calloc.free(outH);
    }
  }

  List<GlyphSelectionRect> selectionRectsOnPage(int page, double startX, double startY, double endX, double endY) {
    const maxRects = 64;
    final buf = calloc<Float>(maxRects * 4);
    final countOut = calloc<Uint32>();
    try {
      final result = selectionRects(page, startX, startY, endX, endY, buf, maxRects * 4, countOut);
      if (result != 0) return const [];
      final count = countOut.value;
      final rects = <GlyphSelectionRect>[];
      for (var i = 0; i < count; i++) {
        final base = i * 4;
        rects.add(GlyphSelectionRect(
          x: buf[base],
          y: buf[base + 1],
          width: buf[base + 2],
          height: buf[base + 3],
        ));
      }
      return rects;
    } finally {
      calloc.free(buf);
      calloc.free(countOut);
    }
  }
}
