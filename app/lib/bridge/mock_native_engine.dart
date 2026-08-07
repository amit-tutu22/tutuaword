import 'dart:convert';
import 'dart:typed_data';

import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';

/// In-memory engine for widget/unit tests (R2.4 — no TextField fallback).
class MockDocumentEngine implements DocumentEngine {
  MockDocumentEngine({
    this.defaultRunId = '00000000-0000-0000-0000-000000000004',
    String initialText = '',
    this.pageWidth = 612,
    this.pageHeight = 792,
  }) : _text = initialText;

  final String defaultRunId;
  final double pageWidth;
  final double pageHeight;
  String _text;
  int _version = 1;
  bool _readOnly = false;
  final Set<int> _stalePages = {};
  bool _trackChanges = false;
  final List<_MockEditSnapshot> _undoStack = [];
  final List<_MockEditSnapshot> _redoStack = [];
  final Map<String, dynamic> _charFormat = {
    'font_family': 'Calibri',
    'font_size': 11,
    'bold': false,
    'italic': false,
    'underline': 'None',
    'strikethrough': false,
    'subscript': false,
    'superscript': false,
    'all_caps': false,
    'small_caps': false,
    'hidden': false,
    'ligatures': true,
  };
  final Map<String, dynamic> _paraFormat = {
    'alignment': 'Left',
    'indent_left': 0,
  };
  String _styleName = 'Normal';

  static const _pageMargin = 72.0;
  static const _lineHeight = 15.4;
  static const _charWidth = 5.72;

  String get text => _text;

  @override
  DisplayListData? fetchDisplayList() => DisplayListData(
        bytes: Uint8List(0),
        version: _version,
        pageWidth: pageWidth,
        pageHeight: pageHeight,
        pageCount: 1,
      );

  @override
  PageDisplayListData? fetchPageDisplayList(int page) => PageDisplayListData(
        bytes: Uint8List(0),
        version: _version,
        pageWidth: pageWidth,
        pageHeight: pageHeight,
      );

  @override
  AtlasData? fetchAtlas() => null;

  @override
  int? fetchAtlasGeneration() => null;

  @override
  String? fetchDocumentText() => _text;

  @override
  String? fetchTextRange(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) {
    if (startRunId != endRunId) return _text.substring(startOffset, endOffset.clamp(0, _text.length));
    final lo = startOffset < endOffset ? startOffset : endOffset;
    final hi = startOffset < endOffset ? endOffset : startOffset;
    if (lo >= _text.length) return '';
    return _text.substring(lo, hi.clamp(0, _text.length));
  }

  @override
  String? fetchCaretFormat(String runId) => jsonEncode({
        'char_format': Map<String, dynamic>.from(_charFormat),
        'para_format': Map<String, dynamic>.from(_paraFormat),
        'style_name': _styleName,
      });

  @override
  DocumentProperties fetchDocumentProperties() => DocumentProperties.empty;

  @override
  bool isDocumentReadOnly() => _readOnly;

  void setReadOnlyForTest(bool value) => _readOnly = value;

  @override
  bool isPageStale(int page) => _stalePages.contains(page);

  void setStalePagesForTest(Set<int> pages) {
    _stalePages
      ..clear()
      ..addAll(pages);
  }

  @override
  String? getLastError() => null;

  @override
  bool newDocument() {
    _text = '';
    _version++;
    return true;
  }

  @override
  int openDocumentBytes(Uint8List bytes, {String? path}) {
    _text = String.fromCharCodes(bytes.where((b) => b >= 32 || b == 10));
    _version++;
    return 0;
  }

  @override
  Uint8List? saveDocumentBytes() => Uint8List.fromList(_text.codeUnits);

  @override
  Uint8List? saveDocumentAsBytes(String formatExtension) => saveDocumentBytes();

  @override
  Uint8List? exportPdfBytes() => Uint8List(0);

  CaretGeometry _geomForOffset(int offset) {
    final x = _pageMargin + (offset * _charWidth);
    return CaretGeometry(x: x, y: _pageMargin + _lineHeight, height: _lineHeight);
  }

  @override
  HitTestResult? hitTestPage(int page, double x, double y) {
    if (_stalePages.contains(page)) return null;
    final offset = ((x - _pageMargin) / _charWidth).round().clamp(0, _text.length);
    return HitTestResult(runId: defaultRunId, charOffset: offset);
  }

  @override
  HitTestResult? fetchDocumentTailHit(int page) =>
      HitTestResult(runId: defaultRunId, charOffset: _text.length);

  @override
  CaretGeometry? caretGeometryAt(int page, double x, double y) {
    final offset = ((x - _pageMargin) / _charWidth).round().clamp(0, _text.length);
    return _geomForOffset(offset);
  }

  @override
  CaretGeometry? caretAtPosition(int page, String runId, int charOffset) =>
      _geomForOffset(charOffset.clamp(0, _text.length));

  @override
  List<GlyphSelectionRect> selectionRectsOnPage(
    int page,
    double startX,
    double startY,
    double endX,
    double endY,
  ) {
    final loX = startX < endX ? startX : endX;
    final hiX = startX < endX ? endX : startX;
    return [
      GlyphSelectionRect(
        x: loX,
        y: startY - _lineHeight,
        width: (hiX - loX).clamp(1, pageWidth),
        height: _lineHeight,
      ),
    ];
  }

  void _pushUndo() {
    _undoStack.add(_MockEditSnapshot.capture(
      text: _text,
      charFormat: _charFormat,
      paraFormat: _paraFormat,
      styleName: _styleName,
    ));
    if (_undoStack.length > 256) {
      _undoStack.removeAt(0);
    }
    _redoStack.clear();
  }

  void _restoreSnapshot(_MockEditSnapshot snap) {
    _text = snap.text;
    _charFormat
      ..clear()
      ..addAll(snap.charFormat);
    _paraFormat
      ..clear()
      ..addAll(snap.paraFormat);
    _styleName = snap.styleName;
    _version++;
  }

  void _insert(String runId, int offset, String text) {
    if (text.isNotEmpty) _pushUndo();
    final off = offset.clamp(0, _text.length);
    _text = _text.substring(0, off) + text + _text.substring(off);
    _version++;
  }

  void _deleteRange(int start, int end) {
    final lo = start.clamp(0, _text.length);
    final hi = end.clamp(0, _text.length);
    if (lo >= hi) return;
    _pushUndo();
    _text = _text.substring(0, lo) + _text.substring(hi);
    _version++;
  }

  @override
  void insertText(String runId, int offset, String text) => _insert(runId, offset, text);

  @override
  bool tryInsertText(String runId, int offset, String text) {
    _insert(runId, offset, text);
    return true;
  }

  @override
  Future<bool> tryInsertTextAsync(String runId, int offset, String text) async {
    _insert(runId, offset, text);
    return true;
  }

  @override
  Future<bool> tryPasteHtmlAsync(String runId, int offset, String html) async {
    _insert(runId, offset, html.replaceAll(RegExp(r'<[^>]+>'), ''));
    return true;
  }

  @override
  Future<bool> tryPasteDocxAsync(String runId, int offset, Uint8List bytes) async => false;

  @override
  Future<bool> deleteRangeAsync(String runId, int start, int end) async {
    _deleteRange(start, end);
    return true;
  }

  @override
  Future<bool> deleteDocRangeAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) async {
    final lo = startOffset < endOffset ? startOffset : endOffset;
    final hi = startOffset < endOffset ? endOffset : startOffset;
    _deleteRange(lo, hi);
    return true;
  }

  @override
  Future<bool> splitParagraphAsync(String runId, int offset) async {
    _insert(runId, offset, '\n');
    return true;
  }

  void _mergeCharFormatPatch(Map<String, dynamic> patch) {
    if (patch.containsKey('clear_highlight') && patch['clear_highlight'] == true) {
      _charFormat.remove('highlight');
    }
    for (final entry in patch.entries) {
      if (entry.key == 'clear_highlight') continue;
      _charFormat[entry.key] = entry.value;
    }
  }

  @override
  Future<bool> applyCharFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) async {
    _pushUndo();
    _mergeCharFormatPatch(jsonDecode(formatJson) as Map<String, dynamic>);
    return true;
  }

  @override
  Future<bool> applyParaFormatJsonAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String formatJson,
  }) async {
    _pushUndo();
    _paraFormat.addAll(jsonDecode(formatJson) as Map<String, dynamic>);
    return true;
  }

  @override
  Future<bool> clearFormatAsync(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) async {
    _charFormat
      ..clear()
      ..addAll({
        'font_family': 'Calibri',
        'font_size': 11,
        'bold': false,
        'italic': false,
        'underline': 'None',
        'strikethrough': false,
        'subscript': false,
        'superscript': false,
        'all_caps': false,
        'small_caps': false,
        'hidden': false,
        'ligatures': true,
      });
    _paraFormat['indent_left'] = 0;
    return true;
  }

  @override
  Future<bool> insertPageBreakAtAsync({String? caretRunId}) async => true;

  @override
  bool setCurrentPageIndex(int page) => true;

  @override
  Future<bool> applyHeading1StyleAsync({String? caretRunId}) async {
    _styleName = 'Heading 1';
    return true;
  }

  @override
  Future<bool> applyNormalStyleAtAsync({String? caretRunId}) async {
    _styleName = 'Normal';
    return true;
  }

  @override
  Future<bool> applyBulletListStyleAsync({String? caretRunId}) async => true;

  @override
  Future<bool> applyNumberedListStyleAsync({String? caretRunId}) async => true;

  @override
  Future<bool> insertTableBlockAsync(int rows, int cols) async => true;

  @override
  Future<bool> insertImageBlockAsync(double width, double height) async => true;

  @override
  Future<bool> undoEditAsync() async {
    if (_undoStack.isEmpty) return false;
    _redoStack.add(_MockEditSnapshot.capture(
      text: _text,
      charFormat: _charFormat,
      paraFormat: _paraFormat,
      styleName: _styleName,
    ));
    _restoreSnapshot(_undoStack.removeLast());
    return true;
  }

  @override
  Future<bool> redoEditAsync() async {
    if (_redoStack.isEmpty) return false;
    _undoStack.add(_MockEditSnapshot.capture(
      text: _text,
      charFormat: _charFormat,
      paraFormat: _paraFormat,
      styleName: _styleName,
    ));
    _restoreSnapshot(_redoStack.removeLast());
    return true;
  }

  @override
  List<String>? spellCheckMisspellings() => const [];

  @override
  bool setTrackChangesEnabled(bool enabled) {
    _trackChanges = enabled;
    return true;
  }

  @override
  bool acceptAllRevisions() => true;

  @override
  bool rejectAllRevisions() => true;
}

class _MockEditSnapshot {
  const _MockEditSnapshot({
    required this.text,
    required this.charFormat,
    required this.paraFormat,
    required this.styleName,
  });

  final String text;
  final Map<String, dynamic> charFormat;
  final Map<String, dynamic> paraFormat;
  final String styleName;

  static _MockEditSnapshot capture({
    required String text,
    required Map<String, dynamic> charFormat,
    required Map<String, dynamic> paraFormat,
    required String styleName,
  }) =>
      _MockEditSnapshot(
        text: text,
        charFormat: Map<String, dynamic>.from(charFormat),
        paraFormat: Map<String, dynamic>.from(paraFormat),
        styleName: styleName,
      );
}
