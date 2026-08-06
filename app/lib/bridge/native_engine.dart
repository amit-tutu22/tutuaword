import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as p;
import 'package:tutuaword/bridge/document_properties.dart';

typedef TwInitNative = Int32 Function(Pointer<NativeFunction<Int32 Function(Uint32, Pointer<Uint8>, IntPtr)>>);
typedef TwInitDart = int Function(Pointer<NativeFunction<Int32 Function(Uint32, Pointer<Uint8>, IntPtr)>>);

typedef TwApplyInsertTextNative = Int32 Function(Pointer<Utf8>, Uint32, Pointer<Utf8>);
typedef TwApplyInsertTextDart = int Function(Pointer<Utf8>, int, Pointer<Utf8>);

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
  Pointer<Float>,
  Pointer<Float>,
);
typedef TwGetPageDisplayListDart = int Function(
  int,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
  Pointer<Float>,
  Pointer<Float>,
);

typedef TwGetDocumentTextNative = Int32 Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwGetDocumentTextDart = int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);

typedef TwGetLastErrorNative = Int32 Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwGetLastErrorDart = int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);

typedef TwGetDocumentPropertiesJsonNative = Int32 Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwGetDocumentPropertiesJsonDart = int Function(Pointer<Pointer<Uint8>>, Pointer<IntPtr>);

typedef TwIsDocumentReadOnlyNative = Int32 Function();
typedef TwIsDocumentReadOnlyDart = int Function();

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

typedef TwApplyCharFormatNative = Int32 Function(
  Pointer<Utf8>,
  Uint32,
  Pointer<Utf8>,
  Uint32,
  Pointer<Utf8>,
);
typedef TwApplyCharFormatDart = int Function(
  Pointer<Utf8>,
  int,
  Pointer<Utf8>,
  int,
  Pointer<Utf8>,
);

typedef TwApplyParaFormatNative = Int32 Function(
  Pointer<Utf8>,
  Uint32,
  Pointer<Utf8>,
  Uint32,
  Pointer<Utf8>,
);
typedef TwApplyParaFormatDart = int Function(
  Pointer<Utf8>,
  int,
  Pointer<Utf8>,
  int,
  Pointer<Utf8>,
);

typedef TwApplyDeleteRangeNative = Int32 Function(Pointer<Utf8>, Uint32, Uint32);
typedef TwApplyDeleteRangeDart = int Function(Pointer<Utf8>, int, int);

typedef TwApplyDeleteDocRangeNative = Int32 Function(
  Pointer<Utf8>,
  Uint32,
  Pointer<Utf8>,
  Uint32,
);
typedef TwApplyDeleteDocRangeDart = int Function(
  Pointer<Utf8>,
  int,
  Pointer<Utf8>,
  int,
);

typedef TwApplySplitParagraphNative = Int32 Function(Pointer<Utf8>, Uint32);
typedef TwApplySplitParagraphDart = int Function(Pointer<Utf8>, int);

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

typedef TwWaitForLayoutNative = Int32 Function();
typedef TwWaitForLayoutDart = int Function();

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

typedef TwFreeBufferNative = Void Function(Pointer<Uint8>, IntPtr);
typedef TwFreeBufferDart = void Function(Pointer<Uint8>, int);

typedef TwShutdownNative = Void Function();
typedef TwShutdownDart = void Function();

class NativeEngine {
  NativeEngine._(this._lib);

  static NativeEngine? _cached;

  final DynamicLibrary _lib;
  late final TwApplyInsertTextDart applyInsertText;
  late final TwApplyPasteHtmlDart applyPasteHtml;
  late final TwApplyPasteDocxDart applyPasteDocx;
  late final TwApplyCharFormatDart applyCharFormat;
  late final TwApplyParaFormatDart applyParaFormat;
  late final TwApplyDeleteRangeDart applyDeleteRange;
  late final TwApplyDeleteDocRangeDart applyDeleteDocRange;
  late final TwApplySplitParagraphDart applySplitParagraph;
  late final TwGetDisplayListDart getDisplayList;
  late final TwGetPageDisplayListDart getPageDisplayList;
  late final TwGetDocumentTextDart getDocumentText;
  late final TwGetLastErrorDart getLastErrorNative;
  late final TwGetDocumentPropertiesJsonDart getDocumentPropertiesJson;
  late final TwIsDocumentReadOnlyDart isDocumentReadOnlyNative;
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
  late final TwWaitForLayoutDart waitForLayoutNative;
  late final TwCaretGeometryDart caretGeometry;
  late final TwCaretAtPositionDart caretAtPositionNative;
  late final TwSelectionRectsDart selectionRects;
  late final TwFreeBufferDart freeBuffer;

  static NativeEngine? load() {
    if (_cached != null) return _cached;
    try {
      final lib = _openLibrary();
      final engine = NativeEngine._(lib);
      lib.lookupFunction<TwInitNative, TwInitDart>('tw_init')(
        Pointer.fromFunction(_noopCallback, 0),
      );
      engine.applyInsertText =
          lib.lookupFunction<TwApplyInsertTextNative, TwApplyInsertTextDart>('tw_apply_insert_text');
      engine.applyPasteHtml =
          lib.lookupFunction<TwApplyPasteHtmlNative, TwApplyPasteHtmlDart>('tw_apply_paste_html');
      engine.applyPasteDocx =
          lib.lookupFunction<TwApplyPasteDocxNative, TwApplyPasteDocxDart>('tw_apply_paste_docx');
      engine.applyCharFormat =
          lib.lookupFunction<TwApplyCharFormatNative, TwApplyCharFormatDart>('tw_apply_char_format');
      engine.applyParaFormat =
          lib.lookupFunction<TwApplyParaFormatNative, TwApplyParaFormatDart>('tw_apply_para_format');
      engine.applyDeleteRange =
          lib.lookupFunction<TwApplyDeleteRangeNative, TwApplyDeleteRangeDart>('tw_apply_delete_range');
      engine.applyDeleteDocRange = lib.lookupFunction<TwApplyDeleteDocRangeNative,
          TwApplyDeleteDocRangeDart>('tw_apply_delete_doc_range');
      engine.applySplitParagraph = lib.lookupFunction<TwApplySplitParagraphNative,
          TwApplySplitParagraphDart>('tw_apply_split_paragraph');
      engine.getDisplayList =
          lib.lookupFunction<TwGetDisplayListNative, TwGetDisplayListDart>('tw_get_display_list');
      engine.getPageDisplayList = lib.lookupFunction<TwGetPageDisplayListNative,
          TwGetPageDisplayListDart>('tw_get_page_display_list');
      engine.getDocumentText =
          lib.lookupFunction<TwGetDocumentTextNative, TwGetDocumentTextDart>('tw_get_document_text');
      engine.getLastErrorNative =
          lib.lookupFunction<TwGetLastErrorNative, TwGetLastErrorDart>('tw_get_last_error');
      engine.getDocumentPropertiesJson = lib.lookupFunction<TwGetDocumentPropertiesJsonNative,
          TwGetDocumentPropertiesJsonDart>('tw_get_document_properties_json');
      engine.isDocumentReadOnlyNative =
          lib.lookupFunction<TwIsDocumentReadOnlyNative, TwIsDocumentReadOnlyDart>(
              'tw_is_document_read_only');
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
      engine.waitForLayoutNative =
          lib.lookupFunction<TwWaitForLayoutNative, TwWaitForLayoutDart>('tw_wait_for_layout');
      engine.caretGeometry =
          lib.lookupFunction<TwCaretGeometryNative, TwCaretGeometryDart>('tw_caret_geometry');
      engine.caretAtPositionNative = lib.lookupFunction<TwCaretAtPositionNative, TwCaretAtPositionDart>(
          'tw_caret_at_position');
      engine.selectionRects =
          lib.lookupFunction<TwSelectionRectsNative, TwSelectionRectsDart>('tw_selection_rects');
      engine.freeBuffer = lib.lookupFunction<TwFreeBufferNative, TwFreeBufferDart>('tw_free_buffer');
      _cached = engine;
      return engine;
    } catch (_) {
      return null;
    }
  }

  /// Release the native session — for tests and app shutdown.
  static void shutdown() {
    final engine = _cached;
    if (engine == null) return;
    engine._lib.lookupFunction<TwShutdownNative, TwShutdownDart>('tw_shutdown')();
    _cached = null;
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

  static int _noopCallback(int _, Pointer<Uint8> __, int ___) => 0;
}

class DisplayListData {
  DisplayListData({
    required this.bytes,
    required this.version,
    required this.pageWidth,
    required this.pageHeight,
    required this.pageCount,
    required this.documentText,
  });

  final Uint8List bytes;
  final int version;
  final double pageWidth;
  final double pageHeight;
  final int pageCount;
  final String documentText;
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
        documentText: fetchDocumentText() ?? '',
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
  Uint8List? fetchPageDisplayList(int page) {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    final outWidth = calloc<Float>();
    final outHeight = calloc<Float>();

    try {
      final result = getPageDisplayList(page, outPtr, outLen, outWidth, outHeight);
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
    final runPtr = runId.toNativeUtf8();
    final textPtr = text.toNativeUtf8();
    try {
      if (applyInsertText(runPtr, offset, textPtr) != 0) {
        throw StateError('insertText failed for run $runId at $offset');
      }
    } finally {
      calloc.free(runPtr);
      calloc.free(textPtr);
    }
  }

  /// Like [insertText] but returns false instead of throwing on failure.
  bool tryInsertText(String runId, int offset, String text) {
    final runPtr = runId.toNativeUtf8();
    final textPtr = text.toNativeUtf8();
    try {
      return applyInsertText(runPtr, offset, textPtr) == 0;
    } finally {
      calloc.free(runPtr);
      calloc.free(textPtr);
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
    final startPtr = startRunId.toNativeUtf8();
    final endPtr = endRunId.toNativeUtf8();
    final jsonPtr = formatJson.toNativeUtf8();
    try {
      return applyCharFormat(startPtr, startOffset, endPtr, endOffset, jsonPtr) == 0;
    } finally {
      calloc.free(startPtr);
      calloc.free(endPtr);
      calloc.free(jsonPtr);
    }
  }

  /// Apply a partial ParaFormat JSON delta over paragraphs touched by the range.
  bool applyParaFormatJson({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) {
    final startPtr = startRunId.toNativeUtf8();
    final endPtr = endRunId.toNativeUtf8();
    final jsonPtr = formatJson.toNativeUtf8();
    try {
      return applyParaFormat(startPtr, startOffset, endPtr, endOffset, jsonPtr) == 0;
    } finally {
      calloc.free(startPtr);
      calloc.free(endPtr);
      calloc.free(jsonPtr);
    }
  }

  bool deleteRange(String runId, int start, int end) {
    final runPtr = runId.toNativeUtf8();
    try {
      return applyDeleteRange(runPtr, start, end) == 0;
    } finally {
      calloc.free(runPtr);
    }
  }

  bool deleteDocRange(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) {
    final startPtr = startRunId.toNativeUtf8();
    final endPtr = endRunId.toNativeUtf8();
    try {
      return applyDeleteDocRange(startPtr, startOffset, endPtr, endOffset) == 0;
    } finally {
      calloc.free(startPtr);
      calloc.free(endPtr);
    }
  }

  bool splitParagraphAt(String runId, int offset) {
    final runPtr = runId.toNativeUtf8();
    try {
      return applySplitParagraph(runPtr, offset) == 0;
    } finally {
      calloc.free(runPtr);
    }
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

  void waitForLayoutSync() {
    waitForLayoutNative();
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
