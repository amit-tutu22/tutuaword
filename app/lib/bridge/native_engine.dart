import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:tutuaword/bridge/command_codec.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/find_format_filter.dart';
import 'package:tutuaword/bridge/find_match.dart';
import 'package:tutuaword/bridge/native_event_router.dart';

typedef TwEventCallbackNative = Void Function(Uint32, Uint64, Pointer<Uint8>, IntPtr);
typedef TwEventCallbackDart = void Function(int, int, Pointer<Uint8>, int);

typedef TwInitNative = Int32 Function(Pointer<NativeFunction<TwEventCallbackNative>>);
typedef TwInitDart = int Function(Pointer<NativeFunction<TwEventCallbackNative>>);

typedef TwAwaitStartupNative = Int32 Function(Uint32);
typedef TwAwaitStartupDart = int Function(int);

typedef TwRegisterFontNative = Int32 Function(Pointer<Utf8>, Bool, Bool, Pointer<Uint8>, IntPtr);
typedef TwRegisterFontDart = int Function(Pointer<Utf8>, bool, bool, Pointer<Uint8>, int);

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

typedef TwApplyParagraphStyleNative = Int32 Function(Pointer<Utf8>, Pointer<Utf8>);
typedef TwApplyParagraphStyleDart = int Function(Pointer<Utf8>, Pointer<Utf8>);

typedef TwApplyDocumentThemeNative = Int32 Function(Pointer<Utf8>);
typedef TwApplyDocumentThemeDart = int Function(Pointer<Utf8>);

typedef TwApplySectionFormatJsonNative = Int32 Function(Pointer<Utf8>, Pointer<Utf8>);
typedef TwApplySectionFormatJsonDart = int Function(Pointer<Utf8>, Pointer<Utf8>);

typedef TwGetSectionFormatJsonNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwGetSectionFormatJsonDart = int Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);

typedef TwInsertSectionBreakNative = Int32 Function(Pointer<Utf8>);
typedef TwInsertSectionBreakDart = int Function(Pointer<Utf8>);
typedef TwEnsureHeaderFooterNative = Int32 Function(Pointer<Utf8>, Int32, Int32);
typedef TwEnsureHeaderFooterDart = int Function(Pointer<Utf8>, int, int);
typedef TwSetEvenAndOddHeadersNative = Int32 Function(Int32);
typedef TwSetEvenAndOddHeadersDart = int Function(int);
typedef TwEvenAndOddHeadersEnabledNative = Int32 Function();
typedef TwEvenAndOddHeadersEnabledDart = int Function();
typedef TwHeaderFooterLinkedNative = Int32 Function(Pointer<Utf8>, Int32, Int32);
typedef TwHeaderFooterLinkedDart = int Function(Pointer<Utf8>, int, int);
typedef TwSetHeaderFooterLinkNative = Int32 Function(Pointer<Utf8>, Int32, Int32, Int32);
typedef TwSetHeaderFooterLinkDart = int Function(Pointer<Utf8>, int, int, int);
typedef TwHeaderFooterSeedRunNative = Int32 Function(
    Pointer<Utf8>, Int32, Int32, Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwHeaderFooterSeedRunDart = int Function(
    Pointer<Utf8>, int, int, Pointer<Pointer<Uint8>>, Pointer<IntPtr>);
typedef TwInsertFieldNative = Int32 Function(
    Pointer<Utf8>, Int32, Pointer<Utf8>);
typedef TwInsertFieldDart = int Function(Pointer<Utf8>, int, Pointer<Utf8>);
typedef TwInsertFootnoteNative = Int32 Function(Pointer<Utf8>, Int32);
typedef TwInsertFootnoteDart = int Function(Pointer<Utf8>, int);
typedef TwInsertCommentNative = Int32 Function(Pointer<Utf8>, Int32, Pointer<Utf8>);
typedef TwInsertCommentDart = int Function(Pointer<Utf8>, int, Pointer<Utf8>);
typedef TwInsertTableOfContentsNative = Int32 Function(Pointer<Utf8>);
typedef TwInsertTableOfContentsDart = int Function(Pointer<Utf8>);
typedef TwAddBibliographySourceNative = Int32 Function(
  Pointer<Utf8>, Pointer<Utf8>, Pointer<Utf8>, Pointer<Utf8>);
typedef TwAddBibliographySourceDart = int Function(
  Pointer<Utf8>, Pointer<Utf8>, Pointer<Utf8>, Pointer<Utf8>);
typedef TwInsertCitationNative = Int32 Function(Pointer<Utf8>, Int32, Pointer<Utf8>);
typedef TwInsertCitationDart = int Function(Pointer<Utf8>, int, Pointer<Utf8>);
typedef TwInsertBibliographyNative = Int32 Function(Pointer<Utf8>);
typedef TwInsertBibliographyDart = int Function(Pointer<Utf8>);
typedef TwInsertBookmarkNative = Int32 Function(Pointer<Utf8>, Int32, Pointer<Utf8>);
typedef TwInsertBookmarkDart = int Function(Pointer<Utf8>, int, Pointer<Utf8>);
typedef TwInsertCrossReferenceNative = Int32 Function(Pointer<Utf8>, Int32, Pointer<Utf8>);
typedef TwInsertCrossReferenceDart = int Function(Pointer<Utf8>, int, Pointer<Utf8>);
typedef TwInsertIndexNative = Int32 Function(Pointer<Utf8>);
typedef TwInsertIndexDart = int Function(Pointer<Utf8>);

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

typedef TwGetDocumentOutlineNative = Int32 Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwGetDocumentOutlineDart = int Function(
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

typedef TwAdjustListLevelNative = Int32 Function(Pointer<Utf8>, Int32);
typedef TwAdjustListLevelDart = int Function(Pointer<Utf8>, int);

typedef TwRestartNumberingNative = Int32 Function(Pointer<Utf8>);
typedef TwRestartNumberingDart = int Function(Pointer<Utf8>);

typedef TwContinueNumberingNative = Int32 Function(Pointer<Utf8>);
typedef TwContinueNumberingDart = int Function(Pointer<Utf8>);

typedef TwInsertTableNative = Int32 Function(Uint32, Uint32, Pointer<Utf8>);
typedef TwInsertTableDart = int Function(int, int, Pointer<Utf8>);
typedef TwDeleteTableRowNative = Int32 Function(Pointer<Utf8>);
typedef TwDeleteTableRowDart = int Function(Pointer<Utf8>);
typedef TwDeleteTableColumnNative = Int32 Function(Pointer<Utf8>);
typedef TwDeleteTableColumnDart = int Function(Pointer<Utf8>);
typedef TwMergeTableCellsNative = Int32 Function(Pointer<Utf8>);
typedef TwMergeTableCellsDart = int Function(Pointer<Utf8>);
typedef TwSplitTableCellNative = Int32 Function(Pointer<Utf8>);
typedef TwSplitTableCellDart = int Function(Pointer<Utf8>);

typedef TwSetTableBorderNative = Int32 Function(Pointer<Utf8>, Float, Uint8, Uint8, Uint8, Uint8);
typedef TwSetTableBorderDart = int Function(Pointer<Utf8>, double, int, int, int, int);

typedef TwSetTableCellShadingNative = Int32 Function(Pointer<Utf8>, Int32, Uint8, Uint8, Uint8);
typedef TwSetTableCellShadingDart = int Function(Pointer<Utf8>, int, int, int, int);

typedef TwResizeTableColumnNative = Int32 Function(Pointer<Utf8>, Float);
typedef TwResizeTableColumnDart = int Function(Pointer<Utf8>, double);

typedef TwAutofitTableNative = Int32 Function(Pointer<Utf8>);
typedef TwAutofitTableDart = int Function(Pointer<Utf8>);

typedef TwSortTableRowsNative = Int32 Function(Pointer<Utf8>, Bool);
typedef TwSortTableRowsDart = int Function(Pointer<Utf8>, bool);

typedef TwInsertNestedTableNative = Int32 Function(Pointer<Utf8>, Uint32, Uint32);
typedef TwInsertNestedTableDart = int Function(Pointer<Utf8>, int, int);

typedef TwInsertTableSumFieldNative = Int32 Function(Pointer<Utf8>);
typedef TwInsertTableSumFieldDart = int Function(Pointer<Utf8>);

typedef TwInsertImageNative = Int32 Function(Float, Float);
typedef TwInsertImageDart = int Function(double, double);
typedef TwInsertShapeNative = Int32 Function(Int32);
typedef TwInsertShapeDart = int Function(int);
typedef TwInsertTextBoxNative = Int32 Function();
typedef TwInsertTextBoxDart = int Function();
typedef TwInsertWordArtNative = Int32 Function(Pointer<Utf8>);
typedef TwInsertWordArtDart = int Function(Pointer<Utf8>);
typedef TwInsertDiagramNative = Int32 Function(Int32);
typedef TwInsertDiagramDart = int Function(int);
typedef TwInsertChartNative = Int32 Function(Int32);
typedef TwInsertChartDart = int Function(int);
typedef TwGetChartDataJsonNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwGetChartDataJsonDart = int Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwLatestChartIdNative = Int32 Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwLatestChartIdDart = int Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwSetChartDataJsonNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Uint8>,
  IntPtr,
);
typedef TwSetChartDataJsonDart = int Function(
  Pointer<Utf8>,
  Pointer<Uint8>,
  int,
);
typedef TwInsertOfficeMathNative = Int32 Function(
  Pointer<Utf8>,
  Int32,
  Pointer<Uint8>,
  IntPtr,
);
typedef TwInsertOfficeMathDart = int Function(
  Pointer<Utf8>,
  int,
  Pointer<Uint8>,
  int,
);
typedef TwInsertOfficeMathDisplayNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Uint8>,
  IntPtr,
);
typedef TwInsertOfficeMathDisplayDart = int Function(
  Pointer<Utf8>,
  Pointer<Uint8>,
  int,
);
typedef TwSetOfficeMathXmlNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Uint8>,
  IntPtr,
);
typedef TwSetOfficeMathXmlDart = int Function(
  Pointer<Utf8>,
  Pointer<Uint8>,
  int,
);
typedef TwGetOfficeMathXmlNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwGetOfficeMathXmlDart = int Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwLatestOfficeMathRunIdNative = Int32 Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwLatestOfficeMathRunIdDart = int Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwDeleteBlockNative = Int32 Function(Pointer<Utf8>);
typedef TwDeleteBlockDart = int Function(Pointer<Utf8>);
typedef TwInsertImageBytesNative = Int32 Function(
    Pointer<Uint8>, IntPtr, Pointer<Utf8>);
typedef TwInsertImageBytesDart = int Function(
    Pointer<Uint8>, int, Pointer<Utf8>);
typedef TwReplaceImageBytesNative = Int32 Function(
    Pointer<Utf8>, Pointer<Uint8>, IntPtr, Pointer<Utf8>);
typedef TwReplaceImageBytesDart = int Function(
    Pointer<Utf8>, Pointer<Uint8>, int, Pointer<Utf8>);
typedef TwSetImageSizeNative = Int32 Function(Pointer<Utf8>, Float, Float);
typedef TwSetImageSizeDart = int Function(Pointer<Utf8>, double, double);
typedef TwSetImageWrapNative = Int32 Function(Pointer<Utf8>, Uint8);
typedef TwSetImageWrapDart = int Function(Pointer<Utf8>, int);
typedef TwSetImageAnchorNative = Int32 Function(
    Pointer<Utf8>, Float, Float, Uint8, Uint8);
typedef TwSetImageAnchorDart = int Function(
    Pointer<Utf8>, double, double, int, int);
typedef TwSetImageTransformNative = Int32 Function(
    Pointer<Utf8>, Float, Float, Float, Float, Float, Float);
typedef TwSetImageTransformDart = int Function(
    Pointer<Utf8>, double, double, double, double, double, double);
typedef TwInsertImageCaptionNative = Int32 Function(Pointer<Utf8>);
typedef TwInsertImageCaptionDart = int Function(Pointer<Utf8>);
typedef TwCompressImageNative = Int32 Function(Pointer<Utf8>, Uint8);
typedef TwCompressImageDart = int Function(Pointer<Utf8>, int);

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
typedef TwGrammarCheckDocumentNative = Int32 Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwGrammarCheckDocumentDart = int Function(
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwCompareDocumentTextNative = Int32 Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwCompareDocumentTextDart = int Function(
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwFindMatchesNative = Int32 Function(
  Pointer<Utf8>,
  Int32,
  Int32,
  Int32,
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwFindMatchesDart = int Function(
  Pointer<Utf8>,
  int,
  int,
  int,
  Pointer<Utf8>,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);

typedef TwSetTrackChangesNative = Int32 Function(Int32);
typedef TwSetTrackChangesDart = int Function(int);
typedef TwSetReadOnlyNative = Int32 Function(Int32);
typedef TwSetReadOnlyDart = int Function(int);
typedef TwAcceptAllRevisionsNative = Int32 Function();
typedef TwAcceptAllRevisionsDart = int Function();
typedef TwRejectAllRevisionsNative = Int32 Function();
typedef TwRejectAllRevisionsDart = int Function();
typedef TwAcceptRevisionAtNative = Int32 Function(Pointer<Utf8>);
typedef TwAcceptRevisionAtDart = int Function(Pointer<Utf8>);
typedef TwRejectRevisionAtNative = Int32 Function(Pointer<Utf8>);
typedef TwRejectRevisionAtDart = int Function(Pointer<Utf8>);
typedef TwAdjacentRevisionRunNative = Int32 Function(
  Pointer<Utf8>,
  Int32,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);
typedef TwAdjacentRevisionRunDart = int Function(
  Pointer<Utf8>,
  int,
  Pointer<Pointer<Uint8>>,
  Pointer<IntPtr>,
);

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

/// Typing never blocks on this — the caret advances optimistically and the
/// caret advances optimistically and the repaint arrives with the event — so
/// this only bounds how long a dropped or coalesced event can stall an
/// operation that genuinely needs settled layout before the sync fallback.
class NativeEngine {
  NativeEngine._(this._lib);

  static NativeEngine? _cached;
  static NativeCallable<TwEventCallbackNative>? _eventCallable;
  static Future<void>? _shutdownInFlight;
  static bool _startupReady = false;

  /// Worker startup publishes `DocumentOpened` under this id.
  static const int startupRequestId = 0;

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
  TwRegisterFontDart? registerFontNative;
  late final TwGetTextRangeDart getTextRange;
  late final TwGetCaretFormatDart getCaretFormat;
  late final TwGetDocumentOutlineDart getDocumentOutline;
  late final TwClearFormatDart clearFormatNative;
  late final TwInsertPageBreakDart insertPageBreak;
  late final TwOpenDocumentWithPathDart openDocumentWithPath;
  late final TwNewDocumentDart newDocumentNative;
  late final TwSaveDocumentDart saveDocument;
  late final TwSetCurrentPageDart setCurrentPage;
  late final TwApplyHeading1Dart applyHeading1;
  late final TwApplyNormalStyleDart applyNormalStyle;
  late final TwApplyParagraphStyleDart applyParagraphStyle;
  late final TwApplyDocumentThemeDart applyDocumentTheme;
  late final TwApplySectionFormatJsonDart applySectionFormatJson;
  late final TwGetSectionFormatJsonDart getSectionFormatJson;
  late final TwInsertSectionBreakDart insertSectionBreak;
  late final TwEnsureHeaderFooterDart ensureHeaderFooter;
  late final TwSetEvenAndOddHeadersDart setEvenAndOddHeaders;
  late final TwEvenAndOddHeadersEnabledDart evenAndOddHeadersEnabled;
  late final TwHeaderFooterLinkedDart headerFooterLinked;
  late final TwSetHeaderFooterLinkDart setHeaderFooterLink;
  late final TwHeaderFooterSeedRunDart headerFooterSeedRun;
  late final TwInsertFieldDart insertField;
  late final TwInsertFootnoteDart insertFootnote;
  late final TwInsertCommentDart insertComment;
  late final TwInsertTableOfContentsDart insertTableOfContents;
  late final TwAddBibliographySourceDart addBibliographySource;
  late final TwInsertCitationDart insertCitation;
  late final TwInsertBibliographyDart insertBibliography;
  late final TwInsertBookmarkDart insertBookmark;
  late final TwInsertCrossReferenceDart insertCrossReference;
  late final TwInsertIndexDart insertIndex;
  late final TwApplyBulletListDart applyBulletList;
  late final TwApplyNumberedListDart applyNumberedList;
  late final TwAdjustListLevelDart adjustListLevel;
  late final TwRestartNumberingDart restartNumbering;
  late final TwContinueNumberingDart continueNumbering;
  late final TwInsertTableDart insertTable;
  late final TwDeleteTableRowDart deleteTableRow;
  late final TwDeleteTableColumnDart deleteTableColumn;
  late final TwMergeTableCellsDart mergeTableCells;
  late final TwSplitTableCellDart splitTableCell;
  late final TwSetTableBorderDart setTableBorder;
  late final TwSetTableCellShadingDart setTableCellShading;
  late final TwResizeTableColumnDart resizeTableColumn;
  late final TwAutofitTableDart autofitTable;
  late final TwSortTableRowsDart sortTableRows;
  late final TwInsertNestedTableDart insertNestedTable;
  late final TwInsertTableSumFieldDart insertTableSumField;
  late final TwInsertImageDart insertImage;
  late final TwInsertShapeDart insertShape;
  late final TwInsertTextBoxDart insertTextBox;
  late final TwInsertWordArtDart insertWordArt;
  late final TwInsertDiagramDart insertDiagram;
  late final TwInsertChartDart insertChart;
  late final TwGetChartDataJsonDart getChartDataJson;
  late final TwLatestChartIdDart latestChartIdNative;
  late final TwSetChartDataJsonDart setChartDataJson;
  late final TwInsertOfficeMathDart insertOfficeMath;
  late final TwInsertOfficeMathDisplayDart insertOfficeMathDisplay;
  late final TwSetOfficeMathXmlDart setOfficeMathXml;
  late final TwGetOfficeMathXmlDart getOfficeMathXml;
  late final TwLatestOfficeMathRunIdDart latestOfficeMathRunIdNative;
  late final TwDeleteBlockDart deleteBlock;
  late final TwInsertImageBytesDart insertImageBytes;
  late final TwSetImageSizeDart setImageSize;
  late final TwSetImageWrapDart setImageWrap;
  late final TwSetImageAnchorDart setImageAnchor;
  late final TwSetImageTransformDart setImageTransform;
  late final TwInsertImageCaptionDart insertImageCaption;
  late final TwCompressImageDart compressImage;
  late final TwReplaceImageBytesDart replaceImageBytes;
  late final TwExportPdfDart exportPdf;
  late final TwUndoDart undo;
  late final TwRedoDart redo;
  late final TwSaveDocumentAsDart saveDocumentAs;
  late final TwSpellCheckDocumentDart spellCheckDocument;
  late final TwGrammarCheckDocumentDart grammarCheckDocument;
  late final TwCompareDocumentTextDart compareDocumentTextNative;
  late final TwFindMatchesDart findMatchesNative;
  late final TwSetReadOnlyDart setReadOnlyNative;
  late final TwSetTrackChangesDart setTrackChanges;
  late final TwAcceptAllRevisionsDart acceptAllRevisionsNative;
  late final TwRejectAllRevisionsDart rejectAllRevisionsNative;
  late final TwAcceptRevisionAtDart acceptRevisionAtNative;
  late final TwRejectRevisionAtDart rejectRevisionAtNative;
  late final TwAdjacentRevisionRunDart adjacentRevisionRunNative;
  late final TwHitTestDart hitTest;
  late final TwDocumentTailHitDart documentTailHit;
  late final TwLastRequestIdDart lastRequestIdNative;
  late final TwCaretGeometryDart caretGeometry;
  late final TwCaretAtPositionDart caretAtPositionNative;
  late final TwSelectionRectsDart selectionRects;
  late final TwFreeBufferDart freeBuffer;
  TwAwaitStartupDart? awaitStartupNative;

  static NativeEngine? load() {
    if (_cached != null) return _cached;
    try {
      final lib = _openLibrary();
      final engine = NativeEngine._(lib);
      _eventCallable ??= NativeCallable<TwEventCallbackNative>.listener(_eventCallback);
      final initCode = lib.lookupFunction<TwInitNative, TwInitDart>('tw_init')(
        _eventCallable!.nativeFunction,
      );
      if (initCode != 0) {
        debugPrint('NativeEngine: tw_init returned $initCode');
      }
      try {
        engine.registerFontNative =
            lib.lookupFunction<TwRegisterFontNative, TwRegisterFontDart>('tw_register_font');
      } on ArgumentError {
        engine.registerFontNative = null;
      }
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
      engine.getDocumentOutline =
          lib.lookupFunction<TwGetDocumentOutlineNative, TwGetDocumentOutlineDart>(
              'tw_get_document_outline');
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
      engine.applyParagraphStyle = lib.lookupFunction<TwApplyParagraphStyleNative, TwApplyParagraphStyleDart>(
          'tw_apply_paragraph_style');
      engine.applyDocumentTheme = lib.lookupFunction<TwApplyDocumentThemeNative, TwApplyDocumentThemeDart>(
          'tw_apply_document_theme');
      engine.applySectionFormatJson = lib.lookupFunction<TwApplySectionFormatJsonNative,
          TwApplySectionFormatJsonDart>('tw_apply_section_format_json');
      engine.getSectionFormatJson = lib.lookupFunction<TwGetSectionFormatJsonNative,
          TwGetSectionFormatJsonDart>('tw_get_section_format_json');
      engine.insertSectionBreak = lib.lookupFunction<TwInsertSectionBreakNative,
          TwInsertSectionBreakDart>('tw_insert_section_break');
      engine.ensureHeaderFooter = lib.lookupFunction<TwEnsureHeaderFooterNative,
          TwEnsureHeaderFooterDart>('tw_ensure_header_footer');
      engine.setEvenAndOddHeaders = lib.lookupFunction<TwSetEvenAndOddHeadersNative,
          TwSetEvenAndOddHeadersDart>('tw_set_even_and_odd_headers');
      engine.evenAndOddHeadersEnabled = lib.lookupFunction<TwEvenAndOddHeadersEnabledNative,
          TwEvenAndOddHeadersEnabledDart>('tw_even_and_odd_headers_enabled');
      engine.headerFooterLinked = lib.lookupFunction<TwHeaderFooterLinkedNative,
          TwHeaderFooterLinkedDart>('tw_header_footer_linked');
      engine.setHeaderFooterLink = lib.lookupFunction<TwSetHeaderFooterLinkNative,
          TwSetHeaderFooterLinkDart>('tw_set_header_footer_link');
      engine.headerFooterSeedRun = lib.lookupFunction<TwHeaderFooterSeedRunNative,
          TwHeaderFooterSeedRunDart>('tw_header_footer_seed_run');
      engine.insertField =
          lib.lookupFunction<TwInsertFieldNative, TwInsertFieldDart>('tw_insert_field');
      engine.insertFootnote = lib.lookupFunction<TwInsertFootnoteNative, TwInsertFootnoteDart>(
          'tw_insert_footnote');
      engine.insertComment = lib.lookupFunction<TwInsertCommentNative, TwInsertCommentDart>(
          'tw_insert_comment');
      engine.insertTableOfContents = lib
          .lookupFunction<TwInsertTableOfContentsNative, TwInsertTableOfContentsDart>(
              'tw_insert_table_of_contents');
      engine.addBibliographySource = lib.lookupFunction<TwAddBibliographySourceNative,
          TwAddBibliographySourceDart>('tw_add_bibliography_source');
      engine.insertCitation = lib.lookupFunction<TwInsertCitationNative, TwInsertCitationDart>(
          'tw_insert_citation');
      engine.insertBibliography = lib
          .lookupFunction<TwInsertBibliographyNative, TwInsertBibliographyDart>(
              'tw_insert_bibliography');
      engine.insertBookmark = lib.lookupFunction<TwInsertBookmarkNative, TwInsertBookmarkDart>(
          'tw_insert_bookmark');
      engine.insertCrossReference = lib
          .lookupFunction<TwInsertCrossReferenceNative, TwInsertCrossReferenceDart>(
              'tw_insert_cross_reference');
      engine.insertIndex =
          lib.lookupFunction<TwInsertIndexNative, TwInsertIndexDart>('tw_insert_index');
      engine.applyBulletList = lib.lookupFunction<TwApplyBulletListNative, TwApplyBulletListDart>(
          'tw_apply_bullet_list');
      engine.applyNumberedList = lib.lookupFunction<TwApplyNumberedListNative, TwApplyNumberedListDart>(
          'tw_apply_numbered_list');
      engine.adjustListLevel = lib.lookupFunction<TwAdjustListLevelNative, TwAdjustListLevelDart>(
          'tw_adjust_list_level');
      engine.restartNumbering = lib.lookupFunction<TwRestartNumberingNative, TwRestartNumberingDart>(
          'tw_restart_numbering');
      engine.continueNumbering = lib.lookupFunction<TwContinueNumberingNative, TwContinueNumberingDart>(
          'tw_continue_numbering');
      engine.insertTable =
          lib.lookupFunction<TwInsertTableNative, TwInsertTableDart>('tw_insert_table');
      engine.deleteTableRow = lib.lookupFunction<TwDeleteTableRowNative, TwDeleteTableRowDart>(
          'tw_delete_table_row');
      engine.deleteTableColumn = lib.lookupFunction<TwDeleteTableColumnNative,
          TwDeleteTableColumnDart>('tw_delete_table_column');
      engine.mergeTableCells = lib.lookupFunction<TwMergeTableCellsNative, TwMergeTableCellsDart>(
          'tw_merge_table_cells');
      engine.splitTableCell = lib.lookupFunction<TwSplitTableCellNative, TwSplitTableCellDart>(
          'tw_split_table_cell');
      engine.setTableBorder = lib.lookupFunction<TwSetTableBorderNative, TwSetTableBorderDart>(
          'tw_set_table_border');
      engine.setTableCellShading =
          lib.lookupFunction<TwSetTableCellShadingNative, TwSetTableCellShadingDart>(
              'tw_set_table_cell_shading');
      engine.resizeTableColumn =
          lib.lookupFunction<TwResizeTableColumnNative, TwResizeTableColumnDart>(
              'tw_resize_table_column');
      engine.autofitTable = lib.lookupFunction<TwAutofitTableNative, TwAutofitTableDart>(
          'tw_autofit_table');
      engine.sortTableRows = lib.lookupFunction<TwSortTableRowsNative, TwSortTableRowsDart>(
          'tw_sort_table_rows');
      engine.insertNestedTable =
          lib.lookupFunction<TwInsertNestedTableNative, TwInsertNestedTableDart>(
              'tw_insert_nested_table');
      engine.insertTableSumField =
          lib.lookupFunction<TwInsertTableSumFieldNative, TwInsertTableSumFieldDart>(
              'tw_insert_table_sum_field');
      engine.insertImage =
          lib.lookupFunction<TwInsertImageNative, TwInsertImageDart>('tw_insert_image');
      engine.insertShape =
          lib.lookupFunction<TwInsertShapeNative, TwInsertShapeDart>('tw_insert_shape');
      engine.insertTextBox =
          lib.lookupFunction<TwInsertTextBoxNative, TwInsertTextBoxDart>('tw_insert_text_box');
      engine.insertWordArt =
          lib.lookupFunction<TwInsertWordArtNative, TwInsertWordArtDart>('tw_insert_word_art');
      engine.insertDiagram =
          lib.lookupFunction<TwInsertDiagramNative, TwInsertDiagramDart>('tw_insert_diagram');
      engine.insertChart =
          lib.lookupFunction<TwInsertChartNative, TwInsertChartDart>('tw_insert_chart');
      engine.getChartDataJson = lib.lookupFunction<TwGetChartDataJsonNative,
          TwGetChartDataJsonDart>('tw_get_chart_data_json');
      engine.latestChartIdNative = lib.lookupFunction<TwLatestChartIdNative,
          TwLatestChartIdDart>('tw_latest_chart_id');
      engine.setChartDataJson = lib.lookupFunction<TwSetChartDataJsonNative,
          TwSetChartDataJsonDart>('tw_set_chart_data_json');
      engine.insertOfficeMath = lib.lookupFunction<TwInsertOfficeMathNative,
          TwInsertOfficeMathDart>('tw_insert_office_math');
      engine.insertOfficeMathDisplay = lib.lookupFunction<
          TwInsertOfficeMathDisplayNative,
          TwInsertOfficeMathDisplayDart>('tw_insert_office_math_display');
      engine.setOfficeMathXml = lib.lookupFunction<TwSetOfficeMathXmlNative,
          TwSetOfficeMathXmlDart>('tw_set_office_math_xml');
      engine.getOfficeMathXml = lib.lookupFunction<TwGetOfficeMathXmlNative,
          TwGetOfficeMathXmlDart>('tw_get_office_math_xml');
      engine.latestOfficeMathRunIdNative = lib.lookupFunction<
          TwLatestOfficeMathRunIdNative,
          TwLatestOfficeMathRunIdDart>('tw_latest_office_math_run_id');
      engine.deleteBlock =
          lib.lookupFunction<TwDeleteBlockNative, TwDeleteBlockDart>('tw_delete_block');
      engine.insertImageBytes = lib.lookupFunction<TwInsertImageBytesNative,
          TwInsertImageBytesDart>('tw_insert_image_bytes');
      engine.setImageSize = lib.lookupFunction<TwSetImageSizeNative, TwSetImageSizeDart>(
          'tw_set_image_size');
      engine.setImageWrap = lib.lookupFunction<TwSetImageWrapNative, TwSetImageWrapDart>(
          'tw_set_image_wrap');
      engine.setImageAnchor = lib.lookupFunction<TwSetImageAnchorNative, TwSetImageAnchorDart>(
          'tw_set_image_anchor');
      engine.setImageTransform = lib.lookupFunction<TwSetImageTransformNative, TwSetImageTransformDart>(
          'tw_set_image_transform');
      engine.insertImageCaption = lib.lookupFunction<TwInsertImageCaptionNative, TwInsertImageCaptionDart>(
          'tw_insert_image_caption');
      engine.compressImage = lib.lookupFunction<TwCompressImageNative, TwCompressImageDart>(
          'tw_compress_image');
      engine.replaceImageBytes = lib.lookupFunction<TwReplaceImageBytesNative,
          TwReplaceImageBytesDart>('tw_replace_image_bytes');
      engine.exportPdf =
          lib.lookupFunction<TwExportPdfNative, TwExportPdfDart>('tw_export_pdf');
      engine.undo = lib.lookupFunction<TwUndoNative, TwUndoDart>('tw_undo');
      engine.redo = lib.lookupFunction<TwRedoNative, TwRedoDart>('tw_redo');
      engine.saveDocumentAs = lib.lookupFunction<TwSaveDocumentAsNative, TwSaveDocumentAsDart>(
          'tw_save_document_as');
      engine.spellCheckDocument = lib.lookupFunction<TwSpellCheckDocumentNative,
          TwSpellCheckDocumentDart>('tw_spell_check_document');
      engine.grammarCheckDocument = lib.lookupFunction<TwGrammarCheckDocumentNative,
          TwGrammarCheckDocumentDart>('tw_grammar_check_document');
      engine.compareDocumentTextNative = lib.lookupFunction<TwCompareDocumentTextNative,
          TwCompareDocumentTextDart>('tw_compare_document_text');
      engine.findMatchesNative = lib.lookupFunction<TwFindMatchesNative, TwFindMatchesDart>(
          'tw_find_matches');
      engine.setTrackChanges =
          lib.lookupFunction<TwSetTrackChangesNative, TwSetTrackChangesDart>('tw_set_track_changes');
      engine.setReadOnlyNative =
          lib.lookupFunction<TwSetReadOnlyNative, TwSetReadOnlyDart>('tw_set_read_only');
      engine.acceptAllRevisionsNative = lib.lookupFunction<TwAcceptAllRevisionsNative,
          TwAcceptAllRevisionsDart>('tw_accept_all_revisions');
      engine.rejectAllRevisionsNative = lib.lookupFunction<TwRejectAllRevisionsNative,
          TwRejectAllRevisionsDart>('tw_reject_all_revisions');
      engine.acceptRevisionAtNative = lib.lookupFunction<TwAcceptRevisionAtNative,
          TwAcceptRevisionAtDart>('tw_accept_revision_at');
      engine.rejectRevisionAtNative = lib.lookupFunction<TwRejectRevisionAtNative,
          TwRejectRevisionAtDart>('tw_reject_revision_at');
      engine.adjacentRevisionRunNative = lib.lookupFunction<TwAdjacentRevisionRunNative,
          TwAdjacentRevisionRunDart>('tw_adjacent_revision_run');
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
      if (pump != null) {
        NativeEventRouter.instance.attachPump(pump);
        // Drain startup events so the first frame does not block on stale work.
        for (var i = 0; i < 16; i++) {
          pump();
        }
      }
      try {
        engine.awaitStartupNative =
            lib.lookupFunction<TwAwaitStartupNative, TwAwaitStartupDart>('tw_await_startup');
      } on ArgumentError {
        engine.awaitStartupNative = null;
      }
      _cached = engine;
      return engine;
    } catch (_) {
      return null;
    }
  }

  /// Wait until the worker's startup document is laid out (after non-blocking `tw_init`).
  ///
  /// Never calls blocking `tw_await_startup` on the UI isolate — that freezes painting.
  static Future<bool> ensureStartupReady({
    Duration timeout = const Duration(seconds: 30),
  }) async {
    if (_cached == null) return false;
    // Startup publishes one event for one request id, so a second wait would
    // find nothing and burn the whole timeout. Latch the first answer instead.
    if (_startupReady) return true;
    final deadline = DateTime.now().add(timeout);
    while (DateTime.now().isBefore(deadline)) {
      _cached!.pumpEventsNative?.call();
      try {
        final eventType = await NativeEventRouter.instance.waitFor(
          startupRequestId,
          timeout: const Duration(milliseconds: 50),
        );
        if (eventType == NativeEventTypes.documentOpened ||
            eventType == NativeEventTypes.displayListReady) {
          debugPrint('NativeEngine: startup ready');
          _startupReady = true;
          return true;
        }
        if (eventType == NativeEventTypes.error) {
          debugPrint('NativeEngine: startup error event');
          return false;
        }
      } on TimeoutException {
        await Future<void>.delayed(const Duration(milliseconds: 16));
      }
    }
    debugPrint('NativeEngine: startup wait timed out; continuing');
    _startupReady = true;
    return true;
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
    _startupReady = false;
    NativeEventRouter.instance.reset();
    NativeEventRouter.instance.detachPump();
    // Keep _eventCallable alive until process exit — closing it races worker callbacks.
  }

  /// Register a font face from raw bytes (mobile / injected-font hosts).
  bool registerFont(
    String family,
    Uint8List data, {
    bool bold = false,
    bool italic = false,
  }) {
    final register = registerFontNative;
    if (register == null || data.isEmpty) {
      return false;
    }
    final familyPtr = family.toNativeUtf8();
    final dataPtr = calloc<Uint8>(data.length);
    try {
      dataPtr.asTypedList(data.length).setAll(0, data);
      return register(familyPtr, bold, italic, dataPtr, data.length) == 0;
    } finally {
      calloc.free(familyPtr);
      calloc.free(dataPtr);
    }
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
    if (Platform.isAndroid) {
      return DynamicLibrary.open('libtw_ffi.so');
    }
    if (Platform.isIOS) {
      // Static libtw_ffi.a is linked into the Runner binary at build time.
      return DynamicLibrary.process();
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

  String? fetchDocumentOutline() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = getDocumentOutline(outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return '[]';
      final json = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return json;
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  String? fetchSectionFormat({String? caretRunId}) {
    final caretPtr = caretRunId?.toNativeUtf8() ?? nullptr;
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = getSectionFormatJson(caretPtr, outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final json = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return json;
    } finally {
      if (caretRunId != null) calloc.free(caretPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  String? fetchChartDataJson(String shapeId) {
    final shapePtr = shapeId.toNativeUtf8();
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = getChartDataJson(shapePtr, outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final json = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return json;
    } finally {
      calloc.free(shapePtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  String? latestChartId() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = latestChartIdNative(outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final id = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return id;
    } finally {
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  @override
  String? fetchOfficeMathXml(String runId) {
    final runPtr = runId.toNativeUtf8();
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = getOfficeMathXml(runPtr, outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final xml = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return xml;
    } finally {
      calloc.free(runPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  @override
  String? latestOfficeMathRunId() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = latestOfficeMathRunIdNative(outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final id = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return id;
    } finally {
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

  bool insertSectionBreakAt({String? caretRunId}) {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return insertSectionBreak(ptr) == 0;
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

  Future<bool> applyParagraphStyleAsync({
    String? caretRunId,
    required String styleName,
  }) async {
    final caretPtr = caretRunId?.toNativeUtf8() ?? nullptr;
    final stylePtr = styleName.toNativeUtf8();
    try {
      return enqueueEdit(() => applyParagraphStyle(caretPtr, stylePtr));
    } finally {
      if (caretRunId != null) calloc.free(caretPtr);
      calloc.free(stylePtr);
    }
  }

  Future<bool> applyDocumentThemeAsync({required String themeName}) async {
    final ptr = themeName.toNativeUtf8();
    try {
      return enqueueEdit(() => applyDocumentTheme(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  Future<bool> insertSectionBreakAtAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => insertSectionBreak(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> ensureHeaderFooterAsync({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(
        () => ensureHeaderFooter(ptr, isHeader ? 1 : 0, pageIndex),
      );
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  String? fetchHeaderFooterSeedRun({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) {
    final caretPtr = caretRunId?.toNativeUtf8() ?? nullptr;
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = headerFooterSeedRun(
        caretPtr,
        isHeader ? 1 : 0,
        pageIndex,
        outPtr,
        outLen,
      );
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final runId = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return runId;
    } finally {
      if (caretRunId != null) calloc.free(caretPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  bool fetchEvenAndOddHeadersEnabled() => evenAndOddHeadersEnabled() == 1;

  Future<bool> setEvenAndOddHeadersAsync({required bool enabled}) =>
      enqueueEdit(() => setEvenAndOddHeaders(enabled ? 1 : 0));

  bool fetchHeaderFooterLinked({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) {
    final caretPtr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      final result = headerFooterLinked(caretPtr, isHeader ? 1 : 0, pageIndex);
      return result == 1;
    } finally {
      if (caretRunId != null) calloc.free(caretPtr);
    }
  }

  Future<bool> setHeaderFooterLinkAsync({
    String? caretRunId,
    required bool isHeader,
    required bool linked,
    int pageIndex = 0,
  }) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(
        () => setHeaderFooterLink(ptr, isHeader ? 1 : 0, pageIndex, linked ? 1 : 0),
      );
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> insertFieldAsync({
    required String runId,
    required int offset,
    required String fieldType,
  }) async {
    final runPtr = runId.toNativeUtf8();
    final typePtr = fieldType.toNativeUtf8();
    try {
      return enqueueEdit(() => insertField(runPtr, offset, typePtr));
    } finally {
      calloc.free(runPtr);
      calloc.free(typePtr);
    }
  }

  Future<bool> insertFootnoteAsync({
    required String runId,
    required int offset,
  }) async {
    final runPtr = runId.toNativeUtf8();
    try {
      return enqueueEdit(() => insertFootnote(runPtr, offset));
    } finally {
      calloc.free(runPtr);
    }
  }

  Future<bool> insertCommentAsync({
    required String runId,
    required int offset,
    String bodyText = '',
  }) async {
    final runPtr = runId.toNativeUtf8();
    final bodyPtr = bodyText.toNativeUtf8();
    try {
      return enqueueEdit(() => insertComment(runPtr, offset, bodyPtr));
    } finally {
      calloc.free(runPtr);
      calloc.free(bodyPtr);
    }
  }

  Future<bool> insertTableOfContentsAsync({String? caretRunId}) async {
    final caretPtr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => insertTableOfContents(caretPtr));
    } finally {
      if (caretRunId != null) calloc.free(caretPtr);
    }
  }

  Future<bool> addBibliographySourceAsync({
    required String key,
    required String author,
    required String title,
    required String year,
  }) async {
    final keyPtr = key.toNativeUtf8();
    final authorPtr = author.toNativeUtf8();
    final titlePtr = title.toNativeUtf8();
    final yearPtr = year.toNativeUtf8();
    try {
      return enqueueEdit(
        () => addBibliographySource(keyPtr, authorPtr, titlePtr, yearPtr),
      );
    } finally {
      calloc.free(keyPtr);
      calloc.free(authorPtr);
      calloc.free(titlePtr);
      calloc.free(yearPtr);
    }
  }

  Future<bool> insertCitationAsync({
    required String runId,
    required int offset,
    required String sourceKey,
  }) async {
    final runPtr = runId.toNativeUtf8();
    final keyPtr = sourceKey.toNativeUtf8();
    try {
      return enqueueEdit(() => insertCitation(runPtr, offset, keyPtr));
    } finally {
      calloc.free(runPtr);
      calloc.free(keyPtr);
    }
  }

  Future<bool> insertBibliographyAsync({String? caretRunId}) async {
    final caretPtr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => insertBibliography(caretPtr));
    } finally {
      if (caretRunId != null) calloc.free(caretPtr);
    }
  }

  Future<bool> insertBookmarkAsync({
    required String runId,
    required int offset,
    required String name,
  }) async {
    final runPtr = runId.toNativeUtf8();
    final namePtr = name.toNativeUtf8();
    try {
      return enqueueEdit(() => insertBookmark(runPtr, offset, namePtr));
    } finally {
      calloc.free(runPtr);
      calloc.free(namePtr);
    }
  }

  Future<bool> insertCrossReferenceAsync({
    required String runId,
    required int offset,
    required String bookmarkName,
  }) async {
    final runPtr = runId.toNativeUtf8();
    final namePtr = bookmarkName.toNativeUtf8();
    try {
      return enqueueEdit(() => insertCrossReference(runPtr, offset, namePtr));
    } finally {
      calloc.free(runPtr);
      calloc.free(namePtr);
    }
  }

  Future<bool> insertIndexAsync({String? caretRunId}) async {
    final caretPtr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => insertIndex(caretPtr));
    } finally {
      if (caretRunId != null) calloc.free(caretPtr);
    }
  }

  Future<bool> applySectionFormatJsonAsync({
    required String formatJson,
    String? caretRunId,
  }) async {
    final jsonPtr = formatJson.toNativeUtf8();
    final caretPtr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => applySectionFormatJson(jsonPtr, caretPtr));
    } finally {
      calloc.free(jsonPtr);
      if (caretRunId != null) calloc.free(caretPtr);
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

  /// Promote (+1) or demote (−1) list level at the caret (F05.S2).
  Future<bool> adjustListLevelAsync({String? caretRunId, required int delta}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      final code = adjustListLevel(ptr, delta);
      if (code == 1) return true;
      if (code != 0) return false;
      return awaitEditCompletion();
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> restartNumberingAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => restartNumbering(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> continueNumberingAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => continueNumbering(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> insertTableBlockAsync(int rows, int cols, {String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => insertTable(rows, cols, ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> deleteTableRowAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => deleteTableRow(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> deleteTableColumnAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => deleteTableColumn(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> mergeTableCellsAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => mergeTableCells(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> splitTableCellAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => splitTableCell(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> setTableBorderAsync({
    String? caretRunId,
    required double width,
    required Color color,
  }) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(
        () => setTableBorder(
          ptr,
          width,
          color.red,
          color.green,
          color.blue,
          (color.a * 255).round(),
        ),
      );
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> setTableCellShadingAsync({
    String? caretRunId,
    Color? shading,
  }) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(
        () => setTableCellShading(
          ptr,
          shading == null ? -1 : shading.red,
          shading?.green ?? 0,
          shading?.blue ?? 0,
          shading == null ? 0 : (shading.a * 255).round(),
        ),
      );
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> resizeTableColumnAsync({
    String? caretRunId,
    required double width,
  }) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => resizeTableColumn(ptr, width));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> autofitTableAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => autofitTable(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> sortTableRowsAsync({
    String? caretRunId,
    required bool ascending,
  }) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => sortTableRows(ptr, ascending));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> insertNestedTableAsync({
    String? caretRunId,
    required int rows,
    required int cols,
  }) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => insertNestedTable(ptr, rows, cols));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> insertTableSumFieldAsync({String? caretRunId}) async {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return enqueueEdit(() => insertTableSumField(ptr));
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

  Future<bool> insertImageBlockAsync(double width, double height) =>
      enqueueEdit(() => insertImage(width, height));

  Future<bool> insertShapeBlockAsync(int shapeType) =>
      enqueueEdit(() => insertShape(shapeType));

  Future<bool> insertTextBoxAsync() => enqueueEdit(() => insertTextBox());

  Future<bool> insertWordArtAsync(String text) async {
    final ptr = text.toNativeUtf8();
    try {
      return enqueueEdit(() => insertWordArt(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  Future<bool> insertDiagramAsync({int diagramType = 0}) =>
      enqueueEdit(() => insertDiagram(diagramType));

  Future<bool> insertChartAsync({int chartType = 0}) =>
      enqueueEdit(() => insertChart(chartType));

  Future<bool> setChartDataAsync(String shapeId, Map<String, dynamic> chartData) async {
    final shapePtr = shapeId.toNativeUtf8();
    final jsonBytes = Uint8List.fromList(utf8.encode(jsonEncode(chartData)));
    final jsonPtr = calloc<Uint8>(jsonBytes.length);
    try {
      jsonPtr.asTypedList(jsonBytes.length).setAll(0, jsonBytes);
      return enqueueEdit(
        () => setChartDataJson(shapePtr, jsonPtr, jsonBytes.length),
      );
    } finally {
      calloc.free(shapePtr);
      calloc.free(jsonPtr);
    }
  }

  @override
  Future<bool> insertOfficeMathAsync({
    required String runId,
    required int offset,
    required String xml,
  }) async {
    final runPtr = runId.toNativeUtf8();
    final xmlBytes = Uint8List.fromList(utf8.encode(xml));
    final xmlPtr = calloc<Uint8>(xmlBytes.length);
    try {
      xmlPtr.asTypedList(xmlBytes.length).setAll(0, xmlBytes);
      return enqueueEdit(
        () => insertOfficeMath(runPtr, offset, xmlPtr, xmlBytes.length),
      );
    } finally {
      calloc.free(runPtr);
      calloc.free(xmlPtr);
    }
  }

  @override
  Future<bool> insertOfficeMathDisplayAsync({
    String? caretRunId,
    required String xml,
  }) async {
    final caretPtr = (caretRunId ?? '').toNativeUtf8();
    final xmlBytes = Uint8List.fromList(utf8.encode(xml));
    final xmlPtr = calloc<Uint8>(xmlBytes.length);
    try {
      xmlPtr.asTypedList(xmlBytes.length).setAll(0, xmlBytes);
      return enqueueEdit(
        () => insertOfficeMathDisplay(caretPtr, xmlPtr, xmlBytes.length),
      );
    } finally {
      calloc.free(caretPtr);
      calloc.free(xmlPtr);
    }
  }

  @override
  Future<bool> setOfficeMathAsync(String runId, String xml) async {
    final runPtr = runId.toNativeUtf8();
    final xmlBytes = Uint8List.fromList(utf8.encode(xml));
    final xmlPtr = calloc<Uint8>(xmlBytes.length);
    try {
      xmlPtr.asTypedList(xmlBytes.length).setAll(0, xmlBytes);
      return enqueueEdit(
        () => setOfficeMathXml(runPtr, xmlPtr, xmlBytes.length),
      );
    } finally {
      calloc.free(runPtr);
      calloc.free(xmlPtr);
    }
  }

  Future<bool> deleteBlockAsync(String blockId) async {
    final ptr = blockId.toNativeUtf8();
    try {
      return enqueueEdit(() => deleteBlock(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  Future<bool> insertImageBytesAsync(Uint8List bytes, String mimeType) async {
    final dataPtr = calloc<Uint8>(bytes.length);
    final mimePtr = mimeType.toNativeUtf8();
    try {
      dataPtr.asTypedList(bytes.length).setAll(0, bytes);
      return enqueueEdit(
        () => insertImageBytes(dataPtr, bytes.length, mimePtr),
      );
    } finally {
      calloc.free(dataPtr);
      calloc.free(mimePtr);
    }
  }

  Future<bool> setImageSizeAsync(String imageId, double width, double height) async {
    final ptr = imageId.toNativeUtf8();
    try {
      return enqueueEdit(() => setImageSize(ptr, width, height));
    } finally {
      calloc.free(ptr);
    }
  }

  Future<bool> replaceImageBytesAsync(
    String imageId,
    Uint8List bytes,
    String mimeType,
  ) async {
    final idPtr = imageId.toNativeUtf8();
    final dataPtr = calloc<Uint8>(bytes.length);
    final mimePtr = mimeType.toNativeUtf8();
    try {
      dataPtr.asTypedList(bytes.length).setAll(0, bytes);
      return enqueueEdit(
        () => replaceImageBytes(idPtr, dataPtr, bytes.length, mimePtr),
      );
    } finally {
      calloc.free(idPtr);
      calloc.free(dataPtr);
      calloc.free(mimePtr);
    }
  }

  Future<bool> setImageWrapAsync(String imageId, int wrap) async {
    final ptr = imageId.toNativeUtf8();
    try {
      return enqueueEdit(() => setImageWrap(ptr, wrap));
    } finally {
      calloc.free(ptr);
    }
  }

  Future<bool> setImageAnchorAsync(
    String imageId,
    double x,
    double y, {
    int originX = 0,
    int originY = 0,
  }) async {
    final ptr = imageId.toNativeUtf8();
    try {
      return enqueueEdit(() => setImageAnchor(ptr, x, y, originX, originY));
    } finally {
      calloc.free(ptr);
    }
  }

  Future<bool> setImageTransformAsync(
    String imageId, {
    double rotationDeg = 0,
    double cropLeft = 0,
    double cropTop = 0,
    double cropRight = 0,
    double cropBottom = 0,
    double opacity = 1,
  }) async {
    final ptr = imageId.toNativeUtf8();
    try {
      return enqueueEdit(
        () => setImageTransform(
          ptr,
          rotationDeg,
          cropLeft,
          cropTop,
          cropRight,
          cropBottom,
          opacity,
        ),
      );
    } finally {
      calloc.free(ptr);
    }
  }

  Future<bool> insertImageCaptionAsync(String imageId) async {
    final ptr = imageId.toNativeUtf8();
    try {
      return enqueueEdit(() => insertImageCaption(ptr));
    } finally {
      calloc.free(ptr);
    }
  }

  Future<bool> compressImageAsync(String imageId, int quality) async {
    final ptr = imageId.toNativeUtf8();
    try {
      return enqueueEdit(() => compressImage(ptr, quality));
    } finally {
      calloc.free(ptr);
    }
  }

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

  bool insertTableBlock(int rows, int cols, {String? caretRunId}) {
    final ptr = caretRunId?.toNativeUtf8() ?? nullptr;
    try {
      return insertTable(rows, cols, ptr) == 0;
    } finally {
      if (caretRunId != null) calloc.free(ptr);
    }
  }

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

  List<String>? grammarCheckIssues() {
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = grammarCheckDocument(outPtr, outLen);
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

  List<FindMatch>? findMatches(
    String query,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
    FindFormatFilter formatFilter = FindFormatFilter.none,
  }) {
    final queryPtr = query.toNativeUtf8();
    final formatPtr = formatFilter.isActive ? formatFilter.encode().toNativeUtf8() : nullptr;
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = findMatchesNative(
        queryPtr,
        matchCase ? 1 : 0,
        useRegex ? 1 : 0,
        useWildcards ? 1 : 0,
        formatPtr,
        outPtr,
        outLen,
      );
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return [];
      final text = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      final decoded = jsonDecode(text);
      if (decoded is! List) return [];
      return decoded
          .whereType<Map>()
          .map((entry) => FindMatch.fromJson(Map<String, dynamic>.from(entry)))
          .toList();
    } finally {
      calloc.free(queryPtr);
      if (formatPtr != nullptr) calloc.free(formatPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
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
    const marginLeft = 72.0;
    const marginTop = 72.0;
    final start = hitTestPage(0, marginLeft, marginTop + 11);
    final tail = fetchDocumentTailHit(0);
    if (start == null || tail == null) return null;
    final ok = await enqueueEdit(() => dispatchCommand(CommandCodec.findReplace(
          startRunId: start.runId,
          startOffset: start.charOffset,
          endRunId: tail.runId,
          endOffset: tail.charOffset,
          find: find,
          replace: replace,
          matchCase: matchCase,
          useRegex: useRegex,
          useWildcards: useWildcards,
        )));
    return ok ? count : null;
  }

  String? compareDocumentText(String otherText) {
    final otherPtr = otherText.toNativeUtf8();
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = compareDocumentTextNative(otherPtr, outPtr, outLen);
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return '';
      final text = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return text;
    } finally {
      calloc.free(otherPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

  bool setTrackChangesEnabled(bool enabled) => setTrackChanges(enabled ? 1 : 0) == 0;

  bool setReadOnlyEnabled(bool enabled) => setReadOnlyNative(enabled ? 1 : 0) == 0;

  bool acceptAllRevisions() => acceptAllRevisionsNative() == 0;

  bool rejectAllRevisions() => rejectAllRevisionsNative() == 0;

  bool acceptRevisionAtCaret({String? caretRunId}) {
    if (caretRunId == null) return false;
    final ptr = caretRunId.toNativeUtf8();
    try {
      return acceptRevisionAtNative(ptr) == 0;
    } finally {
      calloc.free(ptr);
    }
  }

  bool rejectRevisionAtCaret({String? caretRunId}) {
    if (caretRunId == null) return false;
    final ptr = caretRunId.toNativeUtf8();
    try {
      return rejectRevisionAtNative(ptr) == 0;
    } finally {
      calloc.free(ptr);
    }
  }

  String? adjacentRevisionRunId(String? caretRunId, {required bool forward}) {
    if (caretRunId == null) return null;
    final caretPtr = caretRunId.toNativeUtf8();
    final outPtr = calloc<Pointer<Uint8>>();
    final outLen = calloc<IntPtr>();
    try {
      final result = adjacentRevisionRunNative(
        caretPtr,
        forward ? 1 : 0,
        outPtr,
        outLen,
      );
      if (result != 0) return null;
      final len = outLen.value;
      final ptr = outPtr.value;
      if (ptr == nullptr || len == 0) return null;
      final id = ptr.cast<Utf8>().toDartString(length: len);
      freeBuffer(ptr, len);
      return id;
    } finally {
      calloc.free(caretPtr);
      calloc.free(outPtr);
      calloc.free(outLen);
    }
  }

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
