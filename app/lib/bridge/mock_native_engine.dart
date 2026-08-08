import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';

/// In-memory engine for widget/unit tests (R2.4 — no TextField fallback).
class MockDocumentEngine implements DocumentEngine {
  MockDocumentEngine({
    this.defaultRunId = '00000000-0000-0000-0000-000000000004',
    this.headerRunId = '00000000-0000-0000-0000-000000000010',
    this.footerRunId = '00000000-0000-0000-0000-000000000011',
    String initialText = '',
    double pageWidth = 612,
    double pageHeight = 792,
  })  : _text = initialText,
        _sectionFormat = {
          'page_width': pageWidth,
          'page_height': pageHeight,
          'margin_top': 72.0,
          'margin_bottom': 72.0,
          'margin_left': 72.0,
          'margin_right': 72.0,
          'columns': {'count': 1, 'gap': 12.0},
        };

  final String defaultRunId;
  final String headerRunId;
  final String footerRunId;
  final Map<String, dynamic> _sectionFormat;

  double get pageWidth => (_sectionFormat['page_width'] as num).toDouble();
  double get pageHeight => (_sectionFormat['page_height'] as num).toDouble();
  double get _marginLeft => (_sectionFormat['margin_left'] as num).toDouble();
  double get _marginTop => (_sectionFormat['margin_top'] as num).toDouble();
  double get _marginBottom => (_sectionFormat['margin_bottom'] as num).toDouble();
  String _text;
  String _headerText = '';
  String _footerText = '';
  int? _tableRows;
  int? _tableCols;
  int _tableLeadColspan = 1;
  double _tableBorderWidth = 0;
  int? _tableCellShadingArgb;
  List<double>? _tableColumnWidths;
  bool _tableAutofitApplied = false;
  bool _tableSortedAscending = false;
  int _nestedTableCount = 0;
  bool _tableSumFieldInserted = false;
  Uint8List? _insertedImageBytes;
  String? _insertedImageMime;
  String? _mockImageId;
  double _imageDisplayWidth = 72;
  double _imageDisplayHeight = 72;
  String _imageWrap = 'inline';
  double _imageAnchorX = 0;
  double _imageAnchorY = 0;
  int _imageAnchorOriginX = 0;
  int _imageAnchorOriginY = 0;
  double _imageRotationDeg = 0;
  double _imageOpacity = 1;
  bool _imageCaptionInserted = false;
  Uint8List? _replacedImageBytes;
  bool _evenAndOddHeaders = false;
  final Map<String, String> _fieldDisplay = {};
  bool _headerReady = false;
  bool _footerReady = false;
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
  String _themeName = 'Office';
  final List<String> _directInspectorLabels = [];

  static const _themeColors = {
    'Office': {
      'Background1': (255, 255, 255),
      'Text1': (0, 0, 0),
      'Accent1': (68, 114, 196),
      'Accent2': (237, 125, 49),
    },
    'Facet': {
      'Background1': (255, 255, 255),
      'Text1': (0, 0, 0),
      'Accent1': (75, 172, 198),
      'Accent2': (247, 150, 70),
    },
    'Ion': {
      'Background1': (255, 255, 255),
      'Text1': (0, 0, 0),
      'Accent1': (255, 114, 0),
      'Accent2': (91, 155, 213),
    },
  };

  static const _themeFonts = {
    'Office': (major: 'Calibri Light', minor: 'Calibri'),
    'Facet': (major: 'Century Gothic', minor: 'Calibri'),
    'Ion': (major: 'Arial', minor: 'Arial'),
  };

  static const _lineHeight = 15.4;
  static const _charWidth = 5.72;

  String get text => _text;
  String get headerText => _headerText;
  String get footerText => _footerText;
  bool get hasTable => _tableRows != null && _tableCols != null;
  int? get tableRows => _tableRows;
  int? get tableCols => _tableCols;
  double get tableBorderWidth => _tableBorderWidth;
  int? get tableCellShadingArgb => _tableCellShadingArgb;
  List<double>? get tableColumnWidths => _tableColumnWidths;
  bool get tableAutofitApplied => _tableAutofitApplied;
  bool get tableSortedAscending => _tableSortedAscending;
  int get nestedTableCount => _nestedTableCount;
  bool get tableSumFieldInserted => _tableSumFieldInserted;
  Uint8List? get lastInsertedImageBytes => _insertedImageBytes;
  String? get lastInsertedImageMime => _insertedImageMime;
  String? get mockImageId => _mockImageId;
  double? get lastImageDisplayWidth => _mockImageId == null ? null : _imageDisplayWidth;
  double? get lastImageDisplayHeight => _mockImageId == null ? null : _imageDisplayHeight;
  String? get lastImageWrap => _mockImageId == null ? null : _imageWrap;
  double? get lastImageAnchorX => _mockImageId == null ? null : _imageAnchorX;
  double? get lastImageAnchorY => _mockImageId == null ? null : _imageAnchorY;
  double? get lastImageRotationDeg => _mockImageId == null ? null : _imageRotationDeg;
  double? get lastImageOpacity => _mockImageId == null ? null : _imageOpacity;
  bool? get lastImageCaptionInserted =>
      _mockImageId == null ? null : _imageCaptionInserted;
  Uint8List? get lastReplacedImageBytes => _replacedImageBytes;
  int get tableLeadColspan => _tableLeadColspan;

  String _bufferForRun(String runId) {
    if (_fieldDisplay.containsKey(runId)) return _fieldDisplay[runId]!;
    if (runId == headerRunId) return _headerText;
    if (runId == footerRunId) return _footerText;
    return _text;
  }

  void _setBufferForRun(String runId, String value) {
    if (runId == headerRunId) {
      _headerText = value;
    } else if (runId == footerRunId) {
      _footerText = value;
    } else {
      _text = value;
    }
  }

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
    if (startRunId != endRunId) {
      final buffer = _bufferForRun(startRunId);
      return buffer.substring(startOffset, endOffset.clamp(0, buffer.length));
    }
    final buffer = _bufferForRun(startRunId);
    final lo = startOffset < endOffset ? startOffset : endOffset;
    final hi = startOffset < endOffset ? endOffset : startOffset;
    if (lo >= buffer.length) return '';
    return buffer.substring(lo, hi.clamp(0, buffer.length));
  }

  @override
  String? fetchCaretFormat(String runId) => jsonEncode({
        'char_format': Map<String, dynamic>.from(_charFormat),
        'para_format': Map<String, dynamic>.from(_paraFormat),
        'style_name': _styleName,
        'inspector_summary': _inspectorSummary(),
      });

  String _inspectorSummary() {
    final parts = <String>[_styleName];
    for (final label in _directInspectorLabels) {
      parts.add('$label direct');
    }
    return parts.join(' + ');
  }

  @override
  String? fetchDocumentOutline() {
    final level = _mockOutlineLevel();
    if (level == null) return '[]';
    final text = _text.split('\n').first.trim();
    if (text.isEmpty) return '[]';
    return jsonEncode([
      {
        'paragraph_id': defaultRunId,
        'level': level,
        'text': text,
        'run_id': defaultRunId,
        'page': 0,
      },
    ]);
  }

  int? _mockOutlineLevel() {
    if (_styleName.startsWith('Heading ')) {
      final level = int.tryParse(_styleName.substring('Heading '.length));
      if (level != null && level >= 1 && level <= 9) {
        return level - 1;
      }
    }
    final numbering = _paraFormat['numbering'];
    if (numbering is! Map) return null;
    final numberingId = (numbering['numbering_id'] as num?)?.toInt() ?? 0;
    if (numberingId == 1) return null;
    return (numbering['level'] as num?)?.toInt() ?? 0;
  }

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
    _headerText = '';
    _footerText = '';
    _tableRows = null;
    _tableCols = null;
    _tableLeadColspan = 1;
    _tableBorderWidth = 0;
    _tableCellShadingArgb = null;
    _tableColumnWidths = null;
    _tableAutofitApplied = false;
    _tableSortedAscending = false;
    _nestedTableCount = 0;
    _tableSumFieldInserted = false;
    _insertedImageBytes = null;
    _insertedImageMime = null;
    _mockImageId = null;
    _replacedImageBytes = null;
    _imageDisplayWidth = 72;
    _imageDisplayHeight = 72;
    _imageWrap = 'inline';
    _headerReady = false;
    _footerReady = false;
    _version++;
    return true;
  }

  @override
  int openDocumentBytes(Uint8List bytes, {String? path}) {
    try {
      final decoded = jsonDecode(String.fromCharCodes(bytes));
      if (decoded is Map<String, dynamic>) {
        _text = decoded['body'] as String? ?? '';
        _headerText = decoded['header'] as String? ?? '';
        _footerText = decoded['footer'] as String? ?? '';
        _headerReady = decoded.containsKey('header');
        _footerReady = decoded.containsKey('footer');
        final table = decoded['table'];
        if (table is Map<String, dynamic>) {
          _tableRows = (table['rows'] as num?)?.toInt();
          _tableCols = (table['cols'] as num?)?.toInt();
          _tableLeadColspan = (table['lead_colspan'] as num?)?.toInt() ?? 1;
          _tableBorderWidth = (table['border_width'] as num?)?.toDouble() ?? 0;
          _tableCellShadingArgb = (table['cell_shading'] as num?)?.toInt();
          final widths = table['column_widths'];
          _tableColumnWidths = widths is List
              ? widths.map((w) => (w as num).toDouble()).toList()
              : null;
          _tableAutofitApplied = table['autofit'] as bool? ?? false;
          _tableSortedAscending = table['sorted_asc'] as bool? ?? false;
          _nestedTableCount = (table['nested_count'] as num?)?.toInt() ?? 0;
          _tableSumFieldInserted = table['sum_field'] as bool? ?? false;
        } else {
          _tableRows = null;
          _tableCols = null;
          _tableLeadColspan = 1;
          _tableBorderWidth = 0;
          _tableCellShadingArgb = null;
          _tableColumnWidths = null;
          _tableAutofitApplied = false;
          _tableSortedAscending = false;
          _nestedTableCount = 0;
          _tableSumFieldInserted = false;
        }
        final image = decoded['image'];
        if (image is Map<String, dynamic>) {
          final bytes = image['bytes'];
          _insertedImageBytes =
              bytes is List ? Uint8List.fromList(bytes.cast<int>()) : null;
          _insertedImageMime = image['mime'] as String?;
          _mockImageId = image['id'] as String?;
          _imageDisplayWidth = (image['width'] as num?)?.toDouble() ?? 72;
          _imageDisplayHeight = (image['height'] as num?)?.toDouble() ?? 72;
          _imageWrap = image['wrap'] as String? ?? 'inline';
        } else {
          _insertedImageBytes = null;
          _insertedImageMime = null;
          _mockImageId = null;
          _replacedImageBytes = null;
        }
        _version++;
        return 0;
      }
    } catch (_) {}
    _text = String.fromCharCodes(bytes.where((b) => b >= 32 || b == 10));
    _headerText = '';
    _footerText = '';
    _tableRows = null;
    _tableCols = null;
    _tableLeadColspan = 1;
    _tableBorderWidth = 0;
    _tableCellShadingArgb = null;
    _tableColumnWidths = null;
    _tableAutofitApplied = false;
    _tableSortedAscending = false;
    _nestedTableCount = 0;
    _tableSumFieldInserted = false;
    _insertedImageBytes = null;
    _insertedImageMime = null;
    _mockImageId = null;
    _replacedImageBytes = null;
    _imageDisplayWidth = 72;
    _imageDisplayHeight = 72;
    _imageWrap = 'inline';
    _headerReady = false;
    _footerReady = false;
    _version++;
    return 0;
  }

  @override
  Uint8List? saveDocumentBytes() => Uint8List.fromList(jsonEncode({
        'body': _text,
        if (_headerReady) 'header': _headerText,
        if (_footerReady) 'footer': _footerText,
        if (_tableRows != null && _tableCols != null)
          'table': {
            'rows': _tableRows,
            'cols': _tableCols,
            'lead_colspan': _tableLeadColspan,
            if (_tableBorderWidth > 0) 'border_width': _tableBorderWidth,
            if (_tableCellShadingArgb != null) 'cell_shading': _tableCellShadingArgb,
            if (_tableColumnWidths != null) 'column_widths': _tableColumnWidths,
            if (_tableAutofitApplied) 'autofit': true,
            if (_tableSortedAscending) 'sorted_asc': true,
            if (_nestedTableCount > 0) 'nested_count': _nestedTableCount,
            if (_tableSumFieldInserted) 'sum_field': true,
          },
        if (_insertedImageBytes != null)
          'image': {
            'id': _mockImageId ?? 'mock-image-1',
            'bytes': _insertedImageBytes!.toList(),
            if (_insertedImageMime != null) 'mime': _insertedImageMime,
            'width': _imageDisplayWidth,
            'height': _imageDisplayHeight,
            'wrap': _imageWrap,
          },
      }).codeUnits);

  @override
  Uint8List? saveDocumentAsBytes(String formatExtension) => saveDocumentBytes();

  @override
  Uint8List? exportPdfBytes() => Uint8List(0);

  CaretGeometry _geomForOffset(String runId, int offset) {
    final buffer = _bufferForRun(runId);
    final x = _marginLeft + (offset * _charWidth);
    final y = switch (runId) {
      _ when runId == headerRunId => _marginTop * 0.25 + _lineHeight,
      _ when runId == footerRunId => _pageHeight() - _marginBottom * 0.75,
      _ => _marginTop + _lineHeight,
    };
    return CaretGeometry(x: x, y: y, height: _lineHeight);
  }

  double _pageHeight() => (_sectionFormat['page_height'] as num).toDouble();

  @override
  HitTestResult? hitTestPage(int page, double x, double y) {
    if (_stalePages.contains(page)) return null;
    if (_headerReady && y <= _marginTop) {
      final offset = ((x - _marginLeft) / _charWidth).round().clamp(0, _headerText.length);
      return HitTestResult(runId: headerRunId, charOffset: offset);
    }
    if (_footerReady && y >= _pageHeight() - _marginBottom) {
      final offset = ((x - _marginLeft) / _charWidth).round().clamp(0, _footerText.length);
      return HitTestResult(runId: footerRunId, charOffset: offset);
    }
    final offset = ((x - _marginLeft) / _charWidth).round().clamp(0, _text.length);
    return HitTestResult(runId: defaultRunId, charOffset: offset);
  }

  @override
  HitTestResult? fetchDocumentTailHit(int page) =>
      HitTestResult(runId: defaultRunId, charOffset: _text.length);

  @override
  CaretGeometry? caretGeometryAt(int page, double x, double y) {
    if (_headerReady && y <= _marginTop) {
      final offset = ((x - _marginLeft) / _charWidth).round().clamp(0, _headerText.length);
      return _geomForOffset(headerRunId, offset);
    }
    if (_footerReady && y >= _pageHeight() - _marginBottom) {
      final offset = ((x - _marginLeft) / _charWidth).round().clamp(0, _footerText.length);
      return _geomForOffset(footerRunId, offset);
    }
    final offset = ((x - _marginLeft) / _charWidth).round().clamp(0, _text.length);
    return _geomForOffset(defaultRunId, offset);
  }

  @override
  CaretGeometry? caretAtPosition(int page, String runId, int charOffset) =>
      _geomForOffset(runId, charOffset.clamp(0, _bufferForRun(runId).length));

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
      directInspectorLabels: _directInspectorLabels,
      sectionFormat: _sectionFormat,
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
    _sectionFormat
      ..clear()
      ..addAll(snap.sectionFormat);
    _styleName = snap.styleName;
    _directInspectorLabels
      ..clear()
      ..addAll(snap.directInspectorLabels);
    _version++;
  }

  void _insert(String runId, int offset, String text) {
    if (text.isNotEmpty) _pushUndo();
    final buffer = _bufferForRun(runId);
    final off = offset.clamp(0, buffer.length);
    _setBufferForRun(runId, buffer.substring(0, off) + text + buffer.substring(off));
    _version++;
  }

  void _deleteRange(String runId, int start, int end) {
    final buffer = _bufferForRun(runId);
    final lo = start.clamp(0, buffer.length);
    final hi = end.clamp(0, buffer.length);
    if (lo >= hi) return;
    _pushUndo();
    _setBufferForRun(runId, buffer.substring(0, lo) + buffer.substring(hi));
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
    _deleteRange(runId, start, end);
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
    _deleteRange(startRunId, lo, hi);
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
    if (patch.containsKey('clear_color') && patch['clear_color'] == true) {
      _charFormat.remove('color');
      _charFormat.remove('theme_color');
    }
    for (final entry in patch.entries) {
      if (entry.key == 'clear_highlight' || entry.key == 'clear_color') continue;
      if (entry.key == 'theme_color' && entry.value == null) {
        _charFormat.remove('theme_color');
        continue;
      }
      _charFormat[entry.key] = entry.value;
      _trackDirectCharLabel(entry.key, entry.value);
    }
    _resolveThemeColorRef();
  }

  void _trackDirectCharLabel(String key, dynamic value) {
    final label = _directCharLabelFor(key, value);
    if (label == null) return;
    final base = _directLabelBase(key);
    _directInspectorLabels.removeWhere((existing) => _directLabelBaseForExisting(existing) == base);
    _directInspectorLabels.add(label);
  }

  String? _directCharLabelFor(String key, dynamic value) {
    switch (key) {
      case 'bold':
        if (value == true) return 'Bold';
        if (value == false) return 'Not bold';
      case 'italic':
        if (value == true) return 'Italic';
        if (value == false) return 'Not italic';
      case 'underline':
        if (value == 'Single' || value == 'Double') return 'Underline';
        if (value == 'None') return 'No underline';
      case 'strikethrough':
        if (value == true) return 'Strikethrough';
      case 'subscript':
        if (value == true) return 'Subscript';
      case 'superscript':
        if (value == true) return 'Superscript';
      case 'all_caps':
        if (value == true) return 'All caps';
      case 'small_caps':
        if (value == true) return 'Small caps';
      case 'hidden':
        if (value == true) return 'Hidden';
      case 'font_size':
        if (value is num) return '${value.toDouble()} pt';
      case 'font_family':
        if (value is String && value.isNotEmpty) return value;
    }
    return null;
  }

  String _directLabelBase(String key) => switch (key) {
        'bold' => 'Bold',
        'italic' => 'Italic',
        'underline' => 'Underline',
        'strikethrough' => 'Strikethrough',
        'subscript' => 'Subscript',
        'superscript' => 'Superscript',
        'all_caps' => 'All caps',
        'small_caps' => 'Small caps',
        'hidden' => 'Hidden',
        'font_size' => 'Font size',
        'font_family' => 'Font',
        _ => key,
      };

  String _directLabelBaseForExisting(String label) {
    if (label.startsWith('Not ')) {
      return label.substring(4);
    }
    if (label.endsWith(' pt')) return 'Font size';
    const known = {
      'Bold',
      'Italic',
      'Underline',
      'Strikethrough',
      'Subscript',
      'Superscript',
      'All caps',
      'Small caps',
      'Hidden',
    };
    if (known.contains(label)) return label;
    return 'Font';
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
    final patch = jsonDecode(formatJson) as Map<String, dynamic>;
    for (final entry in patch.entries) {
      // `tab_stops: []` must replace (clear); addAll alone is fine for maps,
      // but keep an explicit assign so list identity stays predictable in tests.
      _paraFormat[entry.key] = entry.value;
    }
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
    _directInspectorLabels.clear();
    _charFormat.remove('theme_color');
    return true;
  }

  @override
  Future<bool> insertPageBreakAtAsync({String? caretRunId}) async => true;

  @override
  Future<bool> insertSectionBreakAtAsync({String? caretRunId}) async {
    _version++;
    return true;
  }

  @override
  Future<bool> ensureHeaderFooterAsync({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) async {
    if (isHeader) {
      _headerReady = true;
    } else {
      _footerReady = true;
    }
    _version++;
    return true;
  }

  @override
  String? fetchHeaderFooterSeedRun({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) {
    if (isHeader) {
      return _headerReady ? headerRunId : null;
    }
    return _footerReady ? footerRunId : null;
  }

  @override
  bool fetchEvenAndOddHeadersEnabled() => _evenAndOddHeaders;

  bool _headerFooterLinked = true;

  @override
  Future<bool> setEvenAndOddHeadersAsync({required bool enabled}) async {
    _evenAndOddHeaders = enabled;
    _version++;
    return true;
  }

  @override
  bool fetchHeaderFooterLinked({
    String? caretRunId,
    required bool isHeader,
    int pageIndex = 0,
  }) =>
      _headerFooterLinked;

  @override
  Future<bool> setHeaderFooterLinkAsync({
    String? caretRunId,
    required bool isHeader,
    required bool linked,
    int pageIndex = 0,
  }) async {
    _headerFooterLinked = linked;
    _version++;
    return true;
  }

  @override
  Future<bool> insertFieldAsync({
    required String runId,
    required int offset,
    required String fieldType,
  }) async {
    _pushUndo();
    final display = switch (fieldType.toLowerCase()) {
      'page' => '1',
      'date' => 'January 1, 2026',
      _ => '[field]',
    };
    if (offset == 0 && _bufferForRun(runId).isEmpty) {
      _fieldDisplay[runId] = display;
    } else {
      final newId = '00000000-0000-0000-0000-${(_fieldDisplay.length + 20).toString().padLeft(12, '0')}';
      _fieldDisplay[newId] = display;
    }
    _version++;
    return true;
  }

  @override
  bool setCurrentPageIndex(int page) => true;

  @override
  Future<bool> applyHeading1StyleAsync({String? caretRunId}) async =>
      applyParagraphStyleAsync(caretRunId: caretRunId, styleName: 'Heading 1');

  @override
  Future<bool> applyNormalStyleAtAsync({String? caretRunId}) async =>
      applyParagraphStyleAsync(caretRunId: caretRunId, styleName: 'Normal');

  @override
  Future<bool> applyParagraphStyleAsync({
    String? caretRunId,
    required String styleName,
  }) async {
    final resolved = _MockBuiltinStyles.resolved(styleName);
    if (resolved == null) return false;
    _pushUndo();
    _styleName = styleName;
    _charFormat
      ..clear()
      ..addAll(resolved.charFormat);
    _paraFormat
      ..clear()
      ..addAll(resolved.paraFormat);
    return true;
  }

  @override
  Future<bool> applyDocumentThemeAsync({required String themeName}) async {
    if (!_themeFonts.containsKey(themeName)) return false;
    _pushUndo();
    _themeName = themeName;
    _resolveThemeFontRefs();
    _resolveThemeColorRef();
    return true;
  }

  @override
  String? fetchSectionFormat({String? caretRunId}) => jsonEncode(_sectionFormat);

  @override
  Future<bool> applySectionFormatJsonAsync({
    required String formatJson,
    String? caretRunId,
  }) async {
    final decoded = jsonDecode(formatJson);
    if (decoded is! Map<String, dynamic>) return false;
    _pushUndo();
    _sectionFormat
      ..clear()
      ..addAll(decoded);
    _version++;
    return true;
  }

  void _resolveThemeColorRef() {
    final ref = _charFormat['theme_color'];
    if (ref is! Map) return;
    final slot = ref['slot'];
    final variant = (ref['variant'] as num?)?.toInt() ?? 3;
    if (slot is! String) return;
    final base = _themeColorForSlot(slot);
    if (base == null) return;
    _charFormat['color'] = _colorJson(_applyThemeVariant(base, variant));
  }

  (int, int, int)? _themeColorForSlot(String slot) {
    final theme = _themeColors[_themeName] ?? _themeColors['Office']!;
    return theme[slot];
  }

  (int, int, int) _applyThemeVariant((int, int, int) base, int variant) {
    int blend(int channel, int target, double amount) =>
        (channel + ((target - channel) * amount).round()).clamp(0, 255);
    return switch (variant) {
      0 => (blend(base.$1, 255, 0.80), blend(base.$2, 255, 0.80), blend(base.$3, 255, 0.80)),
      1 => (blend(base.$1, 255, 0.55), blend(base.$2, 255, 0.55), blend(base.$3, 255, 0.55)),
      2 => (blend(base.$1, 255, 0.30), blend(base.$2, 255, 0.30), blend(base.$3, 255, 0.30)),
      4 => (blend(base.$1, 0, 0.25), blend(base.$2, 0, 0.25), blend(base.$3, 0, 0.25)),
      5 => (blend(base.$1, 0, 0.50), blend(base.$2, 0, 0.50), blend(base.$3, 0, 0.50)),
      _ => base,
    };
  }

  Map<String, dynamic> _colorJson((int, int, int) rgb) => {
        'r': rgb.$1,
        'g': rgb.$2,
        'b': rgb.$3,
        'a': 255,
      };

  void _resolveThemeFontRefs() {
    final family = _charFormat['font_family'];
    if (family is String && family.startsWith('+')) {
      _charFormat['font_family'] = _themeFontForReference(family);
    }
  }

  String _themeFontForReference(String reference) {
    final fonts = _themeFonts[_themeName] ?? _themeFonts['Office']!;
    return reference.contains('major') ? fonts.major : fonts.minor;
  }

  @override
  Future<bool> applyBulletListStyleAsync({String? caretRunId}) async {
    _pushUndo();
    _paraFormat['numbering'] = {'numbering_id': 1, 'level': 0};
    return true;
  }

  @override
  Future<bool> applyNumberedListStyleAsync({String? caretRunId}) async {
    _pushUndo();
    _paraFormat['numbering'] = {'numbering_id': 2, 'level': 0};
    _paraFormat['outline_level'] = 0;
    return true;
  }

  @override
  Future<bool> adjustListLevelAsync({String? caretRunId, required int delta}) async {
    final numbering = _paraFormat['numbering'];
    if (numbering is! Map) return false;
    final current = (numbering['level'] as num?)?.toInt() ?? 0;
    final numberingId = (numbering['numbering_id'] as num?)?.toInt() ?? 0;
    // Default bullet/numbered defs expose levels 0 and 1 only.
    final maxLevel = numberingId == 1 || numberingId == 2 ? 1 : 8;
    final next = (current + delta).clamp(0, maxLevel);
    if (next == current) return true;
    _pushUndo();
    _paraFormat['numbering'] = {
      'numbering_id': numberingId,
      'level': next,
    };
    if (numberingId == 2) {
      _paraFormat['outline_level'] = next;
    }
    return true;
  }

  @override
  Future<bool> restartNumberingAsync({String? caretRunId}) async {
    if (_paraFormat['numbering'] == null) return false;
    _pushUndo();
    _paraFormat['num_restart'] = true;
    return true;
  }

  @override
  Future<bool> continueNumberingAsync({String? caretRunId}) async {
    if (_paraFormat['numbering'] == null) return false;
    _pushUndo();
    _paraFormat.remove('num_restart');
    return true;
  }

  @override
  Future<bool> insertTableBlockAsync(int rows, int cols) async {
    _pushUndo();
    _tableRows = rows;
    _tableCols = cols;
    _tableLeadColspan = 1;
    _version++;
    return true;
  }

  @override
  Future<bool> deleteTableRowAsync({String? caretRunId}) async {
    if (_tableRows == null || _tableRows! <= 1) return false;
    _pushUndo();
    _tableRows = _tableRows! - 1;
    _version++;
    return true;
  }

  @override
  Future<bool> deleteTableColumnAsync({String? caretRunId}) async {
    if (_tableCols == null || _tableCols! <= 1) return false;
    _pushUndo();
    _tableCols = _tableCols! - 1;
    _version++;
    return true;
  }

  @override
  Future<bool> mergeTableCellsAsync({String? caretRunId}) async {
    if (_tableCols == null || _tableCols! <= 1 || _tableLeadColspan > 1) return false;
    _pushUndo();
    _tableLeadColspan = 2;
    _version++;
    return true;
  }

  @override
  Future<bool> splitTableCellAsync({String? caretRunId}) async {
    if (_tableLeadColspan <= 1) return false;
    _pushUndo();
    _tableLeadColspan = 1;
    _version++;
    return true;
  }

  @override
  Future<bool> setTableBorderAsync({
    String? caretRunId,
    required double width,
    required Color color,
  }) async {
    if (_tableRows == null) return false;
    _pushUndo();
    _tableBorderWidth = width;
    _version++;
    return true;
  }

  @override
  Future<bool> setTableCellShadingAsync({
    String? caretRunId,
    Color? shading,
  }) async {
    if (_tableRows == null) return false;
    _pushUndo();
    _tableCellShadingArgb = shading == null
        ? null
        : ((shading.alpha * 255).round() << 24) |
            (shading.red << 16) |
            (shading.green << 8) |
            shading.blue;
    _version++;
    return true;
  }

  @override
  Future<bool> resizeTableColumnAsync({
    String? caretRunId,
    required double width,
  }) async {
    if (_tableCols == null) return false;
    _pushUndo();
    _tableColumnWidths = List<double>.filled(_tableCols!, width);
    _version++;
    return true;
  }

  @override
  Future<bool> autofitTableAsync({String? caretRunId}) async {
    if (_tableCols == null) return false;
    _pushUndo();
    final contentWidth = pageWidth - _marginLeft - (_sectionFormat['margin_right'] as num).toDouble();
    final colWidth = contentWidth / _tableCols!;
    _tableColumnWidths = List<double>.filled(_tableCols!, colWidth);
    _tableAutofitApplied = true;
    _version++;
    return true;
  }

  @override
  Future<bool> sortTableRowsAsync({String? caretRunId, required bool ascending}) async {
    if (_tableRows == null) return false;
    _pushUndo();
    _tableSortedAscending = ascending;
    _version++;
    return true;
  }

  @override
  Future<bool> insertNestedTableAsync({
    String? caretRunId,
    required int rows,
    required int cols,
  }) async {
    if (_tableRows == null) return false;
    _pushUndo();
    _nestedTableCount += 1;
    _version++;
    return true;
  }

  @override
  Future<bool> insertTableSumFieldAsync({String? caretRunId}) async {
    if (_tableRows == null) return false;
    _pushUndo();
    _tableSumFieldInserted = true;
    _version++;
    return true;
  }

  @override
  Future<bool> insertImageBlockAsync(double width, double height) async => true;

  @override
  Future<bool> insertShapeBlockAsync(int shapeType) async => true;

  @override
  Future<bool> insertTextBoxAsync() async => true;

  @override
  Future<bool> insertWordArtAsync(String text) async => true;

  @override
  Future<bool> insertDiagramAsync() async => true;

  @override
  Future<bool> insertChartAsync() async => true;

  @override
  Future<bool> insertImageBytesAsync(Uint8List bytes, String mimeType) async {
    _pushUndo();
    _insertedImageBytes = Uint8List.fromList(bytes);
    _insertedImageMime = mimeType;
    _mockImageId ??= 'mock-image-1';
    _replacedImageBytes = null;
    _version++;
    return true;
  }

  @override
  Future<bool> setImageSizeAsync(String imageId, double width, double height) async {
    if (_mockImageId == null) return false;
    _pushUndo();
    _imageDisplayWidth = width;
    _imageDisplayHeight = height;
    _version++;
    return true;
  }

  @override
  Future<bool> replaceImageBytesAsync(
    String imageId,
    Uint8List bytes,
    String mimeType,
  ) async {
    if (_mockImageId == null) return false;
    _pushUndo();
    _replacedImageBytes = Uint8List.fromList(bytes);
    _insertedImageMime = mimeType;
    _version++;
    return true;
  }

  @override
  Future<bool> setImageWrapAsync(String imageId, int wrap) async {
    if (_mockImageId == null) return false;
    _pushUndo();
    _imageWrap = switch (wrap) {
      1 => 'square',
      3 => 'behind',
      _ => 'inline',
    };
    _version++;
    return true;
  }

  @override
  Future<bool> setImageAnchorAsync(
    String imageId,
    double x,
    double y, {
    int originX = 0,
    int originY = 0,
  }) async {
    if (_mockImageId == null) return false;
    _pushUndo();
    _imageAnchorX = x;
    _imageAnchorY = y;
    _imageAnchorOriginX = originX;
    _imageAnchorOriginY = originY;
    if (_imageWrap == 'inline') {
      _imageWrap = 'square';
    }
    _version++;
    return true;
  }

  @override
  Future<bool> setImageTransformAsync(
    String imageId, {
    double rotationDeg = 0,
    double cropLeft = 0,
    double cropTop = 0,
    double cropRight = 0,
    double cropBottom = 0,
    double opacity = 1,
  }) async {
    if (_mockImageId == null) return false;
    _pushUndo();
    _imageRotationDeg = rotationDeg;
    _imageOpacity = opacity;
    _version++;
    return true;
  }

  @override
  Future<bool> insertImageCaptionAsync(String imageId) async {
    if (_mockImageId == null) return false;
    _pushUndo();
    _imageCaptionInserted = true;
    _version++;
    return true;
  }

  @override
  Future<bool> compressImageAsync(String imageId, int quality) async {
    if (_mockImageId == null) return false;
    _pushUndo();
    _insertedImageMime = 'image/jpeg';
    _version++;
    return true;
  }

  @override
  Future<bool> undoEditAsync() async {
    if (_undoStack.isEmpty) return false;
    _redoStack.add(_MockEditSnapshot.capture(
      text: _text,
      charFormat: _charFormat,
      paraFormat: _paraFormat,
      styleName: _styleName,
      directInspectorLabels: _directInspectorLabels,
      sectionFormat: _sectionFormat,
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
      directInspectorLabels: _directInspectorLabels,
      sectionFormat: _sectionFormat,
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
    required this.directInspectorLabels,
    required this.sectionFormat,
  });

  final String text;
  final Map<String, dynamic> charFormat;
  final Map<String, dynamic> paraFormat;
  final String styleName;
  final List<String> directInspectorLabels;
  final Map<String, dynamic> sectionFormat;

  static _MockEditSnapshot capture({
    required String text,
    required Map<String, dynamic> charFormat,
    required Map<String, dynamic> paraFormat,
    required String styleName,
    required List<String> directInspectorLabels,
    required Map<String, dynamic> sectionFormat,
  }) =>
      _MockEditSnapshot(
        text: text,
        charFormat: Map<String, dynamic>.from(charFormat),
        paraFormat: Map<String, dynamic>.from(paraFormat),
        styleName: styleName,
        directInspectorLabels: List<String>.from(directInspectorLabels),
        sectionFormat: Map<String, dynamic>.from(sectionFormat),
      );
}

/// Resolved built-in paragraph styles mirroring [`StyleSheet::with_defaults`].
class _MockBuiltinStyles {
  static _ResolvedStyle? resolved(String styleName) => _styles[styleName];

  static const _baseChar = {
    'font_family': 'Calibri',
    'underline': 'None',
    'strikethrough': false,
    'subscript': false,
    'superscript': false,
    'all_caps': false,
    'small_caps': false,
    'hidden': false,
    'ligatures': true,
  };

  static const _basePara = {
    'alignment': 'Left',
    'indent_left': 0.0,
  };

  static final Map<String, _ResolvedStyle> _styles = {
    'Normal': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 12.0, 'bold': false, 'italic': false},
      paraFormat: Map<String, dynamic>.from(_basePara),
    ),
    'Heading 1': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 16.0, 'bold': true, 'italic': false},
      paraFormat: {
        ..._basePara,
        'space_before': 12.0,
        'space_after': 6.0,
        'outline_level': 0,
      },
    ),
    'Heading 2': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 14.0, 'bold': true, 'italic': false},
      paraFormat: {
        ..._basePara,
        'space_before': 12.0,
        'space_after': 6.0,
        'outline_level': 1,
      },
    ),
    'Heading 3': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 13.0, 'bold': true, 'italic': false},
      paraFormat: {
        ..._basePara,
        'space_before': 6.0,
        'space_after': 3.0,
        'outline_level': 2,
      },
    ),
    'Heading 4': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 12.0, 'bold': false, 'italic': true},
      paraFormat: {
        ..._basePara,
        'space_before': 6.0,
        'space_after': 3.0,
        'outline_level': 3,
      },
    ),
    'Heading 5': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 11.0, 'bold': false, 'italic': false},
      paraFormat: {
        ..._basePara,
        'space_before': 6.0,
        'space_after': 3.0,
        'outline_level': 4,
      },
    ),
    'Heading 6': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 10.0, 'bold': false, 'italic': true},
      paraFormat: {
        ..._basePara,
        'space_before': 6.0,
        'space_after': 3.0,
        'outline_level': 5,
      },
    ),
    'Heading 7': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 10.0, 'bold': false, 'italic': false},
      paraFormat: {
        ..._basePara,
        'space_before': 6.0,
        'space_after': 3.0,
        'outline_level': 6,
      },
    ),
    'Heading 8': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 9.0, 'bold': false, 'italic': true},
      paraFormat: {
        ..._basePara,
        'space_before': 6.0,
        'space_after': 3.0,
        'outline_level': 7,
      },
    ),
    'Heading 9': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 9.0, 'bold': false, 'italic': false},
      paraFormat: {
        ..._basePara,
        'space_before': 6.0,
        'space_after': 3.0,
        'outline_level': 8,
      },
    ),
    'Quote': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 12.0, 'bold': false, 'italic': true},
      paraFormat: {
        ..._basePara,
        'indent_left': 36.0,
        'indent_right': 36.0,
        'space_before': 6.0,
        'space_after': 6.0,
      },
    ),
    'Caption': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 9.0, 'bold': false, 'italic': true},
      paraFormat: {
        ..._basePara,
        'alignment': 'Center',
        'space_before': 3.0,
        'space_after': 3.0,
      },
    ),
  };
}

class _ResolvedStyle {
  const _ResolvedStyle({required this.charFormat, required this.paraFormat});

  final Map<String, dynamic> charFormat;
  final Map<String, dynamic> paraFormat;
}
