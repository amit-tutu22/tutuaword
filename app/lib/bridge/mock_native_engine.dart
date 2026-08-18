import 'dart:convert';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/spell_issue.dart';
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/find_format_filter.dart';
import 'package:tutuaword/bridge/find_match.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/editor/doc_range.dart';

/// In-memory engine for widget/unit tests (R2.4 — no TextField fallback).
class MockDocumentEngine implements DocumentEngine {
  static const _spellWords = {
    'the', 'a', 'an', 'has', 'document', 'editor', 'works', 'well', 'received', 'invitation',
    'party', 'misspelling', 'errors', 'quick', 'brown', 'fox', 'with', 'and', 'to', 'in',
    'is', 'are', 'was', 'were', 'be', 'been', 'being', 'have', 'had', 'do', 'does', 'did',
    'will', 'would', 'could', 'should', 'may', 'might', 'must', 'shall', 'can', 'need',
    'this', 'that', 'these', 'those', 'it', 'its', 'they', 'them', 'their', 'we', 'our',
    'you', 'your', 'he', 'she', 'his', 'her', 'who', 'what', 'when', 'where', 'why', 'how',
    'all', 'each', 'every', 'both', 'few', 'more', 'most', 'other', 'some', 'such', 'no',
    'not', 'only', 'same', 'so', 'than', 'too', 'very', 'just', 'also', 'now', 'then',
    'from', 'for', 'of', 'on', 'at', 'by', 'about', 'into', 'through', 'during', 'before',
    'after', 'above', 'below', 'between', 'under', 'again', 'further', 'once', 'here',
    'there', 'any', 'own', 'or', 'if', 'because', 'until', 'while',
    'word', 'text', 'spell', 'check', 'grammar', 'review', 'track', 'changes',
  };

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
  int _sectionCount = 1;
  int _headerEditSection = 0;
  String _sectionOneHeader = '';
  bool _sectionOneHeaderLinked = true;
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
  String? _lastError;
  String? _encryptionPassword;
  int setCurrentPageIndexCount = 0;
  int _imageAnchorOriginX = 0;
  int _imageAnchorOriginY = 0;
  double _imageRotationDeg = 0;
  double _imageOpacity = 1;
  bool _imageCaptionInserted = false;
  String _imageAltText = '';
  Uint8List? _replacedImageBytes;
  String? _mockChartId;
  Map<String, dynamic>? _chartData;
  String? _mockOfficeMathRunId;
  final Map<String, String> _officeMathXml = {};
  bool _evenAndOddHeaders = false;
  final Map<String, String> _fieldDisplay = {};
  /// Form field run id → kind (`text` / `checkbox`) and current value (F26.S1).
  final Map<String, String> _formFieldKind = {};
  final Map<String, String> _formFieldValue = {};
  String? lastFormFieldRunId;
  bool _headerReady = false;
  bool _footerReady = false;
  int _version = 1;
  bool _readOnly = false;
  final Set<int> _stalePages = {};
  bool _trackChanges = false;
  final List<_TrackedRevision> _trackedRevisions = [];
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
  String get headerText => resolvedHeaderText(0);
  String get footerText => _footerText;
  int get sectionCount => _sectionCount;

  String resolvedHeaderText(int sectionIndex) {
    if (sectionIndex <= 0) return _headerText;
    if (_sectionOneHeaderLinked) return _headerText;
    return _sectionOneHeader;
  }

  int _sectionIndexForPage(int pageIndex) =>
      pageIndex >= 1 && _sectionCount > 1 ? 1 : 0;
  bool get hasTable => _tableRows != null && _tableCols != null;
  int? get tableRows => _tableRows;
  int? get tableCols => _tableCols;

  int? _lastDiagramType;
  int? get lastDiagramType => _lastDiagramType;
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
  String? get lastImageAltText => _mockImageId == null ? null : _imageAltText;
  Uint8List? get lastReplacedImageBytes => _replacedImageBytes;
  int get tableLeadColspan => _tableLeadColspan;

  String _bufferForRun(String runId) {
    if (_fieldDisplay.containsKey(runId)) return _fieldDisplay[runId]!;
    if (runId == headerRunId) {
      if (_headerEditSection <= 0) return _headerText;
      if (_sectionOneHeaderLinked) return _headerText;
      return _sectionOneHeader;
    }
    if (runId == footerRunId) return _footerText;
    return _text;
  }

  void _setBufferForRun(String runId, String value) {
    if (runId == headerRunId) {
      if (_headerEditSection <= 0) {
        _headerText = value;
        return;
      }
      if (_sectionOneHeaderLinked) {
        _sectionOneHeaderLinked = false;
        _sectionOneHeader = _headerText;
      }
      _sectionOneHeader = value;
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
  /// Injected multi-heading outline for F19.S2 widget tests.
  List<Map<String, dynamic>>? _outlineOverride;

  /// Override [fetchDocumentOutline] with an explicit headings tree fixture.
  void setOutlineEntriesForTest(List<Map<String, dynamic>> entries) {
    _outlineOverride = entries;
  }

  String? fetchDocumentOutline() {
    if (_outlineOverride != null) {
      return jsonEncode(_outlineOverride);
    }
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

  List<Map<String, dynamic>>? _semanticTreeOverride;
  List<Map<String, dynamic>>? _accessibilityIssuesOverride;
  List<Map<String, dynamic>>? _documentInspectOverride;
  int _inspectCommentCount = 0;
  int _inspectMetadataCount = 0;
  int _inspectHiddenCount = 0;
  String? _mockTitle;
  String? _mockAuthor;

  /// Override [fetchSemanticTree] with an explicit fixture (F21.S1).
  void setSemanticTreeForTest(List<Map<String, dynamic>> roots) {
    _semanticTreeOverride = roots;
  }

  /// Override [fetchAccessibilityIssues] with an explicit fixture (F21.S4).
  void setAccessibilityIssuesForTest(List<Map<String, dynamic>> issues) {
    _accessibilityIssuesOverride = issues;
  }

  /// Seed Document Inspector counts for tests (F22.S3).
  void setDocumentInspectCountsForTest({
    int comments = 0,
    int metadata = 0,
    int hiddenText = 0,
    String? title,
    String? author,
  }) {
    _documentInspectOverride = null;
    _inspectCommentCount = comments;
    _inspectMetadataCount = metadata;
    _inspectHiddenCount = hiddenText;
    _mockTitle = title;
    _mockAuthor = author;
    if (metadata == 0 && (title != null || author != null)) {
      _inspectMetadataCount =
          (title != null && title.trim().isNotEmpty ? 1 : 0) +
              (author != null && author.trim().isNotEmpty ? 1 : 0);
    }
  }

  @override
  String? fetchSemanticTree() {
    if (_semanticTreeOverride != null) {
      return jsonEncode(_semanticTreeOverride);
    }
    final level = _mockOutlineLevel();
    final text = _text.split('\n').first.trim();
    if (text.isEmpty) return '[]';
    if (level != null) {
      return jsonEncode([
        {
          'id': defaultRunId,
          'role': 'heading',
          'level': level,
          'text': text,
          'children': <Map<String, dynamic>>[],
        },
      ]);
    }
    return jsonEncode([
      {
        'id': defaultRunId,
        'role': 'paragraph',
        'text': text,
        'children': <Map<String, dynamic>>[],
      },
    ]);
  }

  @override
  String? fetchAccessibilityIssues() {
    if (_accessibilityIssuesOverride != null) {
      return jsonEncode(_accessibilityIssuesOverride);
    }
    final issues = <Map<String, dynamic>>[];
    final level = _mockOutlineLevel();
    final text = _text.split('\n').first.trim();
    if (level != null && text.isEmpty) {
      issues.add({
        'rule': 'empty_heading',
        'severity': 'error',
        'message': 'Heading is empty',
        'node_id': defaultRunId,
        'run_id': defaultRunId,
      });
    }
    if (_mockImageId != null && _imageAltText.trim().isEmpty) {
      issues.add({
        'rule': 'missing_alt',
        'severity': 'error',
        'message': 'Picture is missing alternative text',
        'node_id': _mockImageId,
      });
    }
    final color = _charFormat['color'];
    if (color is Map &&
        color['r'] == 200 &&
        color['g'] == 200 &&
        color['b'] == 200) {
      issues.add({
        'rule': 'low_contrast',
        'severity': 'warning',
        'message': 'Text contrast ratio 1.6:1 is below 4.5:1 (WCAG AA)',
        'node_id': defaultRunId,
        'run_id': defaultRunId,
      });
    }
    return jsonEncode(issues);
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
  String? fetchDocumentInspect() {
    if (_documentInspectOverride != null) {
      return jsonEncode(_documentInspectOverride);
    }
    final findings = <Map<String, dynamic>>[];
    if (_inspectCommentCount > 0) {
      findings.add({
        'category': 'comments',
        'count': _inspectCommentCount,
        'message':
            '$_inspectCommentCount comment${_inspectCommentCount == 1 ? '' : 's'}',
      });
    }
    if (_inspectMetadataCount > 0) {
      findings.add({
        'category': 'metadata',
        'count': _inspectMetadataCount,
        'message':
            '$_inspectMetadataCount document propert${_inspectMetadataCount == 1 ? 'y' : 'ies'}',
      });
    }
    if (_inspectHiddenCount > 0) {
      findings.add({
        'category': 'hidden_text',
        'count': _inspectHiddenCount,
        'message':
            '$_inspectHiddenCount hidden text run${_inspectHiddenCount == 1 ? '' : 's'}',
      });
    }
    return jsonEncode(findings);
  }

  @override
  bool removeInspectFindings({
    bool comments = false,
    bool metadata = false,
    bool hiddenText = false,
  }) {
    if (comments) _inspectCommentCount = 0;
    if (metadata) {
      _inspectMetadataCount = 0;
      _mockTitle = null;
      _mockAuthor = null;
    }
    if (hiddenText) _inspectHiddenCount = 0;
    return true;
  }

  final List<Map<String, dynamic>> _digitalSignatures = [];
  bool _signaturesTampered = false;

  /// Mark mock signatures as tampered for verification tests (F22.S4).
  void setSignaturesTamperedForTest(bool value) => _signaturesTampered = value;

  @override
  String? fetchDigitalSignatures() => jsonEncode(_digitalSignatures);

  @override
  String? verifyDigitalSignatures() {
    return jsonEncode(_digitalSignatures.map((sig) {
      final status = _signaturesTampered ? 'tampered' : 'valid';
      return {
        'signature_id': sig['id'],
        'status': status,
        'signer_name': (sig['signer'] as Map?)?['name'] ?? '',
        'message': _signaturesTampered
            ? 'Document has changed since it was signed'
            : 'Self-attested signature intact (not certificate-backed)',
      };
    }).toList());
  }

  @override
  bool signDocument({
    required String name,
    String email = '',
    String? organization,
  }) {
    if (name.trim().isEmpty) return false;
    _digitalSignatures.add({
      'id': 'mock-sig-${_digitalSignatures.length + 1}',
      'signer': {
        'name': name.trim(),
        'email': email,
        if (organization != null && organization.isNotEmpty)
          'organization': organization,
      },
      'timestamp': DateTime.now().toUtc().toIso8601String(),
      'public_key': 'mock',
      'signature_value': 'mock',
      'signed_content_hash': 'mockhash',
    });
    _signaturesTampered = false;
    return true;
  }

  @override
  bool clearDigitalSignatures() {
    _digitalSignatures.clear();
    _signaturesTampered = false;
    return true;
  }

  @override
  DocumentProperties fetchDocumentProperties() => DocumentProperties(
        title: _mockTitle,
        author: _mockAuthor,
      );

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
  String? getLastError() => _lastError;

  bool _looksPasswordProtected(Uint8List bytes, {String? path}) =>
      DocumentReader.isPasswordProtectedDocx(bytes, path: path);

  @override
  bool newDocument() {
    _text = '';
    _headerText = '';
    _footerText = '';
    _sectionCount = 1;
    _headerEditSection = 0;
    _sectionOneHeader = '';
    _sectionOneHeaderLinked = true;
    _formatSpans.clear();
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
  int openDocumentBytes(Uint8List bytes, {String? path, String? password}) {
    if (_looksPasswordProtected(bytes, path: path)) {
      if (password == null || password.isEmpty) {
        _lastError = 'document is password-protected';
        return -2;
      }
      if (password != 'secret') {
        _lastError = 'incorrect password';
        return -2;
      }
      // Mock decrypt: treat as empty body after a correct password.
      _text = '';
      _encryptionPassword = password;
      _version++;
      _lastError = null;
      return 0;
    }
    _encryptionPassword = null;
    // Real OOXML / ZIP templates (F24.S1): extract body text for widget tests.
    if (bytes.length >= 2 && bytes[0] == 0x50 && bytes[1] == 0x4B) {
      try {
        _text = DocumentReader.extractText(bytes, path: path ?? 'template.docx');
        _headerText = '';
        _footerText = '';
        _headerReady = false;
        _footerReady = false;
        _tableRows = null;
        _tableCols = null;
        _version++;
        _lastError = null;
        return 0;
      } catch (_) {
        // Fall through to JSON / printable-byte parsers.
      }
    }
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
    _sectionCount = 1;
    _headerEditSection = 0;
    _sectionOneHeader = '';
    _sectionOneHeaderLinked = true;
    _formatSpans.clear();
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
  Uint8List? saveDocumentBytes() {
    final plain = Uint8List.fromList(jsonEncode({
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
    final password = _encryptionPassword;
    if (password == null || password.isEmpty) return plain;
    return _mockEncryptedPackage(plain);
  }

  @override
  Uint8List? saveDocumentAsBytes(String formatExtension) => saveDocumentBytes();

  /// Synthetic encrypted OOXML markers for tests (not real Agile crypto).
  Uint8List _mockEncryptedPackage(Uint8List plaintext) {
    final archive = Archive()
      ..addFile(ArchiveFile(
        '[Content_Types].xml',
        32,
        '<?xml version="1.0"?><Types/>'.codeUnits,
      ))
      ..addFile(ArchiveFile('EncryptionInfo', 9, 'encrypted'.codeUnits))
      ..addFile(ArchiveFile('EncryptedPackage', plaintext.length, plaintext));
    return ZipEncoder().encodeBytes(archive);
  }

  /// Minimal PDF used by File→Print tests (F25.S1/S2).
  static final Uint8List mockPrintPdf = Uint8List.fromList(
    '%PDF-1.4\n1 0 obj<< /Type /Catalog >>endobj\ntrailer<<>>\n%%EOF\n'
        .codeUnits,
  );

  /// Last layout passed to [exportPdfBytesForPrint] (F25.S2 tests).
  PrintLayoutSettings? lastPrintLayout;

  /// Last selection passed to [exportPdfBytesForPrint] (F25.S3 tests).
  DocRange? lastPrintSelection;

  @override
  Uint8List? exportPdfBytes() => Uint8List(0);

  @override
  Uint8List? exportPdfBytesForPrint([
    PrintLayoutSettings? layout,
    DocRange? selection,
  ]) {
    lastPrintLayout = layout ?? PrintLayoutSettings.defaults;
    lastPrintSelection = selection;
    return Uint8List.fromList(mockPrintPdf);
  }

  CaretGeometry _geomForOffset(String runId, int offset) {
    final buffer = _bufferForRun(runId);
    final (line, col) = _lineColumnForOffset(buffer, offset);
    final x = _marginLeft + col * _charWidth;
    final y = switch (runId) {
      _ when runId == headerRunId => _marginTop * 0.25 + _lineHeight,
      _ when runId == footerRunId => _pageHeight() - _marginBottom * 0.75,
      _ => _marginTop + (line + 1) * _lineHeight,
    };
    return CaretGeometry(x: x, y: y, height: _lineHeight);
  }

  int _lineCount(String text) {
    if (text.isEmpty) return 1;
    return '\n'.allMatches(text).length + 1;
  }

  (int line, int col) _lineColumnForOffset(String text, int offset) {
    final clamped = offset.clamp(0, text.length);
    final before = text.substring(0, clamped);
    final parts = before.split('\n');
    return (parts.length - 1, parts.last.length);
  }

  int _offsetForLineColumn(String text, int line, int col) {
    var currentLine = 0;
    var i = 0;
    while (i < text.length && currentLine < line) {
      if (text[i] == '\n') currentLine++;
      i++;
    }
    if (currentLine < line) return text.length;
    final lineStart = i;
    final nextNl = text.indexOf('\n', lineStart);
    final lineEnd = nextNl == -1 ? text.length : nextNl;
    final colClamped = col.clamp(0, lineEnd - lineStart);
    return lineStart + colClamped;
  }

  int _lineIndexFromY(double y) {
    final raw = ((y - _marginTop - _lineHeight) / _lineHeight).round();
    return raw.clamp(0, _lineCount(_text) - 1);
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
    final line = _lineIndexFromY(y);
    final col = ((x - _marginLeft) / _charWidth).round();
    final offset = _offsetForLineColumn(_text, line, col).clamp(0, _text.length);
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
    final line = _lineIndexFromY(y);
    final col = ((x - _marginLeft) / _charWidth).round();
    final offset = _offsetForLineColumn(_text, line, col).clamp(0, _text.length);
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
    if (text.isEmpty) return;
    _pushUndo();
    // Mirror engine paste normalize: CRLF → LF, strip junk controls / map PUA.
    final normalized = text
        .replaceAll('\r\n', '\n')
        .replaceAll('\r', '\n')
        .replaceAllMapped(RegExp(r'[\uF0E0-\uF0EF]'), (_) => '→')
        .replaceAllMapped(RegExp(r'[\uF035\uF0B6\uF0B7\uF0A7\uF0A8]'), (_) => '•')
        .replaceAllMapped(RegExp(r'[\uF000-\uF8FF]'), (_) => '·');
    final buffer = _bufferForRun(runId);
    final off = offset.clamp(0, buffer.length);
    _setBufferForRun(runId, buffer.substring(0, off) + normalized + buffer.substring(off));
    if (_trackChanges && normalized.isNotEmpty) {
      _trackedRevisions.add(_TrackedRevision(runId: runId, offset: off, text: normalized));
    }
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
  Future<bool> tryPasteDocxAsync(String runId, int offset, Uint8List bytes) async {
    final text = _plainTextFromDocx(bytes);
    if (text == null || text.isEmpty) return false;
    _pushUndo();
    _insert(runId, offset, text);
    _version++;
    return true;
  }

  @override
  Future<bool> deleteRangeAsync(String runId, int start, int end) async {
    _deleteRange(runId, start, end);
    return true;
  }

  @override
  Future<bool> replaceRangeAsync(
    String runId,
    int start,
    int end,
    String text,
  ) async {
    final buffer = _bufferForRun(runId);
    final lo = start.clamp(0, buffer.length);
    final hi = end.clamp(0, buffer.length);
    if (lo == hi && text.isEmpty) return true;
    _pushUndo();
    final next = buffer.substring(0, lo) + text + buffer.substring(hi);
    _setBufferForRun(runId, next);
    if (_trackChanges && text.isNotEmpty) {
      _trackedRevisions.add(_TrackedRevision(runId: runId, offset: lo, text: text));
    }
    _version++;
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
  Future<HitTestResult?> splitParagraphAsync(String runId, int offset) async {
    final buffer = _bufferForRun(runId);
    final off = offset.clamp(0, buffer.length);
    _insert(runId, off, '\n');
    final caretOffset = off == 0 ? 0 : off + 1;
    return HitTestResult(
      runId: runId,
      charOffset: caretOffset.clamp(0, _bufferForRun(runId).length),
    );
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
    if (_sectionCount < 2) {
      _sectionCount = 2;
      _sectionOneHeader = '';
      _sectionOneHeaderLinked = true;
    _formatSpans.clear();
    }
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
      _headerEditSection = _sectionIndexForPage(pageIndex);
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

  bool _linkedForSection(int sectionIndex) =>
      sectionIndex <= 0 ? false : _sectionOneHeaderLinked;

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
  }) {
    if (!isHeader) return _headerFooterLinked;
    final sectionIndex = _sectionIndexForPage(pageIndex);
    return _linkedForSection(sectionIndex);
  }

  @override
  Future<bool> setHeaderFooterLinkAsync({
    String? caretRunId,
    required bool isHeader,
    required bool linked,
    int pageIndex = 0,
  }) async {
    if (isHeader) {
      final sectionIndex = _sectionIndexForPage(pageIndex);
      if (sectionIndex <= 0) return false;
      if (linked) {
        _sectionOneHeaderLinked = true;
    _formatSpans.clear();
        _sectionOneHeader = '';
      } else {
        _sectionOneHeaderLinked = false;
        _sectionOneHeader = resolvedHeaderText(sectionIndex);
      }
    } else {
      _headerFooterLinked = linked;
    }
    _version++;
    return true;
  }

  int _footnoteCount = 0;
  int _commentCount = 0;
  final List<Map<String, dynamic>> _commentThreads = [];
  Uint8List? _lastExportedSelectionDocx;

  static String _superscriptNumber(int n) {
    const supers = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    return n.toString().split('').map((d) => supers[int.parse(d)]).join();
  }

  @override
  Future<bool> insertFootnoteAsync({
    required String runId,
    required int offset,
  }) async {
    _pushUndo();
    _footnoteCount++;
    _insert(runId, offset, _superscriptNumber(_footnoteCount));
    _version++;
    return true;
  }

  int _endnoteCount = 0;

  @override
  Future<bool> insertEndnoteAsync({
    required String runId,
    required int offset,
  }) async {
    _pushUndo();
    _endnoteCount++;
    _insert(runId, offset, _romanNumeral(_endnoteCount));
    _version++;
    return true;
  }

  String _romanNumeral(int n) {
    const values = [
      (1000, 'm'),
      (900, 'cm'),
      (500, 'd'),
      (400, 'cd'),
      (100, 'c'),
      (90, 'xc'),
      (50, 'l'),
      (40, 'xl'),
      (10, 'x'),
      (9, 'ix'),
      (5, 'v'),
      (4, 'iv'),
      (1, 'i'),
    ];
    var remaining = n;
    final buf = StringBuffer();
    for (final (value, symbol) in values) {
      while (remaining >= value) {
        buf.write(symbol);
        remaining -= value;
      }
    }
    return buf.isEmpty ? 'i' : buf.toString();
  }

  @override
  Future<bool> insertTableOfFiguresAsync({String? caretRunId}) async {
    _pushUndo();
    _text = '$_text\nTable of Figures\nFigure 1 Example\t1';
    _version++;
    return true;
  }

  @override
  Future<bool> insertCommentAsync({
    required String runId,
    required int offset,
    String bodyText = '',
  }) async {
    _pushUndo();
    _commentCount++;
    _commentThreads.add({
      'comment_id': _commentCount - 1,
      'resolved': false,
      'messages': [
        {
          'author': 'Author',
          'body': [
            {
              'runs': [
                {'text': bodyText.isEmpty ? 'Comment' : bodyText},
              ],
            },
          ],
        },
      ],
    });
    _insert(runId, offset, '[C$_commentCount]');
    _version++;
    return true;
  }

  @override
  Future<bool> applySpellReplacementAsync({
    required int plainStart,
    required int plainEnd,
    required String replacement,
  }) async {
    final text = _fullDocumentText();
    if (plainStart < 0 || plainEnd > text.length || plainStart >= plainEnd) {
      return false;
    }
    _pushUndo();
    final updated = text.replaceRange(plainStart, plainEnd, replacement);
    _setFullDocumentText(updated);
    _version++;
    return true;
  }

  @override
  Future<Uint8List?> exportSelectionDocxAsync({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
  }) async {
    final selected = _selectedText(startRunId, startOffset, endRunId, endOffset);
    if (selected.isEmpty) return null;
    _lastExportedSelectionDocx = _minimalDocxBytes(selected);
    return _lastExportedSelectionDocx;
  }

  @override
  String? getCommentsJson() {
    if (_commentThreads.isEmpty) return '[]';
    return jsonEncode(_commentThreads);
  }

  @override
  Future<bool> replyToCommentAsync({
    required int commentId,
    required String bodyText,
  }) async {
    final thread = _commentThreads.cast<Map<String, dynamic>?>().firstWhere(
          (t) => t?['comment_id'] == commentId,
          orElse: () => null,
        );
    if (thread == null) return false;
    (thread['messages'] as List).add({
      'author': 'Author',
      'body': [
        {
          'runs': [
            {'text': bodyText},
          ],
        },
      ],
    });
    _version++;
    return true;
  }

  @override
  Future<bool> resolveCommentAsync({
    required int commentId,
    required bool resolved,
  }) async {
    for (final thread in _commentThreads) {
      if (thread['comment_id'] == commentId) {
        thread['resolved'] = resolved;
        _version++;
        return true;
      }
    }
    return false;
  }

  String _fullDocumentText() => _text;

  void _setFullDocumentText(String text) {
    _text = text;
  }

  String _selectedText(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) {
    if (startRunId == endRunId) {
      final buf = _bufferForRun(startRunId);
      final start = startOffset.clamp(0, buf.length);
      final end = endOffset.clamp(start, buf.length);
      return buf.substring(start, end);
    }
    return selectedTextFromRuns(startRunId, startOffset, endRunId, endOffset);
  }

  String selectedTextFromRuns(
    String startRunId,
    int startOffset,
    String endRunId,
    int endOffset,
  ) {
    final start = _bufferForRun(startRunId);
    final end = _bufferForRun(endRunId);
    return '${start.substring(startOffset.clamp(0, start.length))}${end.substring(0, endOffset.clamp(0, end.length))}';
  }

  Uint8List _minimalDocxBytes(String text) {
    final escaped = text
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;');
    final documentXml =
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
        '<w:body><w:p><w:r><w:t>$escaped</w:t></w:r></w:p></w:body></w:document>';
    final contentTypes =
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
        '<Override PartName="/word/document.xml" '
        'ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
        '</Types>';
    final archive = Archive()
      ..addFile(ArchiveFile('[Content_Types].xml', contentTypes.length, contentTypes.codeUnits))
      ..addFile(ArchiveFile('word/document.xml', documentXml.length, documentXml.codeUnits));
    return ZipEncoder().encodeBytes(archive);
  }

  String? _plainTextFromDocx(Uint8List bytes) {
    try {
      final archive = ZipDecoder().decodeBytes(bytes);
      final doc = archive.findFile('word/document.xml');
      if (doc == null) return null;
      final xml = utf8.decode(doc.readBytes() ?? const <int>[]);
      final matches = RegExp(r'<w:t[^>]*>([^<]*)</w:t>').allMatches(xml);
      return matches.map((m) => m.group(1) ?? '').join();
    } catch (_) {
      return null;
    }
  }

  @override
  Future<bool> insertTableOfContentsAsync({String? caretRunId}) async {
    _pushUndo();
    final toc = StringBuffer('\nTable of Contents');
    for (final heading in _mockOutlineHeadings) {
      toc.write('\n${heading.text}\t${heading.page}');
    }
    if (_mockOutlineHeadings.isEmpty) {
      toc.write('\nIntroduction\t1');
      toc.write('\nBackground\t1');
    }
    _text = '$_text$toc';
    _version++;
    return true;
  }

  final List<({String text, int page})> _mockOutlineHeadings = [];

  final Map<String, ({String author, String title, String year})> _bibliographySources = {
    'Smith2020': (author: 'Smith, John', title: 'Example Research', year: '2020'),
  };
  final List<String> _citedKeys = [];

  /// Test hook: headings that appear in the next TOC insert.
  @visibleForTesting
  void setMockOutlineHeadingsForTest(List<({String text, int page})> headings) {
    _mockOutlineHeadings
      ..clear()
      ..addAll(headings);
  }

  @override
  Future<bool> addBibliographySourceAsync({
    required String key,
    required String author,
    required String title,
    required String year,
  }) async {
    _bibliographySources[key] = (author: author, title: title, year: year);
    _version++;
    return true;
  }

  @override
  Future<bool> insertCitationAsync({
    required String runId,
    required int offset,
    required String sourceKey,
  }) async {
    _pushUndo();
    final source = _bibliographySources[sourceKey];
    final display = source == null
        ? '[$sourceKey]'
        : '(${source.author.split(',').first.trim()}, ${source.year})';
    _insert(runId, offset, display);
    if (!_citedKeys.contains(sourceKey)) {
      _citedKeys.add(sourceKey);
    }
    _version++;
    return true;
  }

  @override
  Future<bool> insertBibliographyAsync({String? caretRunId}) async {
    _pushUndo();
    final buffer = StringBuffer('\nBibliography');
    for (final key in _citedKeys) {
      final source = _bibliographySources[key];
      if (source != null) {
        buffer.write('\n${source.author}. ${source.title}. ${source.year}.');
      }
    }
    if (_citedKeys.isEmpty && _bibliographySources.containsKey('Smith2020')) {
      final source = _bibliographySources['Smith2020']!;
      buffer.write('\n${source.author}. ${source.title}. ${source.year}.');
    }
    _text = '$_text$buffer';
    _version++;
    return true;
  }

  final Map<String, String> _bookmarks = {};
  final List<Map<String, dynamic>> _bookmarkEntries = [];
  List<Map<String, dynamic>>? _bookmarksOverride;
  final Map<String, Map<String, dynamic>> _hyperlinks = {};

  /// Override [fetchBookmarks] with an explicit fixture (F19.S4).
  void setBookmarksForTest(List<Map<String, dynamic>> entries) {
    _bookmarksOverride = entries;
  }

  @override
  String? fetchBookmarks() {
    if (_bookmarksOverride != null) {
      return jsonEncode(_bookmarksOverride);
    }
    return jsonEncode(_bookmarkEntries);
  }

  @override
  String? fetchHyperlinkAt(String runId) {
    final stored = _hyperlinks[runId];
    if (stored == null) return null;
    return jsonEncode(stored);
  }

  @override
  Future<bool> insertBookmarkAsync({
    required String runId,
    required int offset,
    required String name,
  }) async {
    _pushUndo();
    if (_text.isEmpty) {
      _text = 'Introduction';
    }
    final anchor = _text.substring(offset.clamp(0, _text.length)).trim();
    _bookmarks[name] = anchor.isEmpty ? name : anchor;
    _bookmarkEntries.removeWhere(
      (e) => (e['name'] as String?)?.toLowerCase() == name.toLowerCase(),
    );
    _bookmarkEntries.add({
      'name': name,
      'run_id': runId,
      'paragraph_id': runId,
      'page': 0,
    });
    _version++;
    return true;
  }

  @override
  Future<bool> insertHyperlinkAsync({
    required String runId,
    required int offset,
    required String url,
    required String text,
    String? tooltip,
  }) async {
    _pushUndo();
    final display = text.isEmpty ? url : text;
    _insert(runId, offset, display);
    final anchor = url.startsWith('#') ? url.substring(1) : null;
    _hyperlinks[runId] = {
      'url': url,
      'anchor': anchor,
      'text': display,
      'tooltip': tooltip,
    };
    _version++;
    return true;
  }

  @override
  Future<bool> insertFormFieldAsync({
    required String runId,
    required int offset,
    required String kind,
    String? name,
    String? initialValue,
  }) async {
    _pushUndo();
    final normalized = kind.toLowerCase();
    final isCheckbox =
        normalized == 'checkbox' || normalized == 'formcheckbox' || normalized == 'check';
    final value = initialValue ?? '';
    final display = isCheckbox
        ? (const {'1', 'true', 'yes', 'checked', 'x'}.contains(value.trim().toLowerCase())
            ? '☑'
            : '☐')
        : (value.isEmpty ? '____' : value);
    final fieldId =
        '00000000-0000-0000-0000-${(_formFieldKind.length + 40).toString().padLeft(12, '0')}';
    _formFieldKind[fieldId] = isCheckbox ? 'checkbox' : 'text';
    _formFieldValue[fieldId] = isCheckbox
        ? (display == '☑' ? 'true' : 'false')
        : display;
    lastFormFieldRunId = fieldId;
    _fieldDisplay[fieldId] = display;
    _insert(runId, offset, display);
    if (name != null && name.isNotEmpty) {
      // Name is metadata-only in the mock; keep it out of plain text.
    }
    _version++;
    return true;
  }

  @override
  Future<bool> setFormFieldValueAsync({
    required String runId,
    required String value,
  }) async {
    final kind = _formFieldKind[runId];
    if (kind == null) return false;
    _pushUndo();
    if (kind == 'checkbox') {
      final lower = value.trim().toLowerCase();
      final current = _formFieldValue[runId] == 'true';
      final checked = lower == 'toggle'
          ? !current
          : const {'1', 'true', 'yes', 'checked', 'x'}.contains(lower);
      _formFieldValue[runId] = checked ? 'true' : 'false';
      _fieldDisplay[runId] = checked ? '☑' : '☐';
      final old = current ? '☑' : '☐';
      final next = checked ? '☑' : '☐';
      _text = _text.replaceFirst(old, next);
    } else {
      final old = _formFieldValue[runId] ?? '';
      _formFieldValue[runId] = value;
      _fieldDisplay[runId] = value;
      if (old.isNotEmpty && _text.contains(old)) {
        _text = _text.replaceFirst(old, value);
      } else {
        _text = '$_text$value';
      }
    }
    _version++;
    return true;
  }

  @override
  Future<bool> insertMergeFieldAsync({
    required String runId,
    required int offset,
    required String name,
  }) async {
    if (name.trim().isEmpty) return false;
    _pushUndo();
    final display = '«${name.trim()}»';
    _insert(runId, offset, display);
    _version++;
    return true;
  }

  @override
  Future<bool> applyMailMergeRowAsync({
    required Map<String, String> values,
  }) async {
    _pushUndo();
    var next = _text;
    for (final entry in values.entries) {
      next = next.replaceAll('«${entry.key}»', entry.value);
    }
    _text = next;
    _version++;
    return true;
  }

  @override
  Future<bool> insertCrossReferenceAsync({
    required String runId,
    required int offset,
    required String bookmarkName,
  }) async {
    _pushUndo();
    final display = _bookmarks[bookmarkName];
    if (display == null) {
      return false;
    }
    _insert(runId, offset, display);
    _version++;
    return true;
  }

  @override
  Future<bool> insertIndexAsync({String? caretRunId}) async {
    _pushUndo();
    if (_bookmarks.isEmpty) {
      return false;
    }
    final buffer = StringBuffer('\nIndex');
    final entries = _bookmarks.values.toList()
      ..sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
    for (final entry in entries) {
      buffer.write('\n$entry');
    }
    _text = '$_text$buffer';
    _version++;
    return true;
  }

  @override
  Future<bool> insertFieldAsync({
    required String runId,
    required int offset,
    required String fieldType,
    String? mergeName,
  }) async {
    _pushUndo();
    final display = switch (fieldType.toLowerCase()) {
      'page' => '1',
      'date' => 'January 1, 2026',
      'next' => '<<Next Record>>',
      'if' => mergeName == null ? '<<IF>>' : '<<IF $mergeName>>',
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
  bool setCurrentPageIndex(int page) {
    setCurrentPageIndexCount++;
    return true;
  }

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
  String? fetchChartDataJson(String shapeId) {
    if (_mockChartId == null || shapeId != _mockChartId || _chartData == null) {
      return null;
    }
    return jsonEncode(_chartData);
  }

  @override
  String? latestChartId() => _mockChartId;

  @override
  String? fetchOfficeMathXml(String runId) => _officeMathXml[runId];

  @override
  String? fetchImageAltText(String imageId) {
    if (_mockImageId == null || imageId != _mockImageId) return null;
    return _imageAltText;
  }

  @override
  String? latestOfficeMathRunId() => _mockOfficeMathRunId;

  /// Last OMML applied via equation insert/edit (tests).
  String? get lastOfficeMathXml =>
      _mockOfficeMathRunId == null ? null : _officeMathXml[_mockOfficeMathRunId];

  /// Last chart dataset applied via [setChartDataAsync] (tests).
  Map<String, dynamic>? get lastChartData =>
      _chartData == null ? null : Map<String, dynamic>.from(_chartData!);

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
  Future<bool> insertTableBlockAsync(int rows, int cols, {String? caretRunId}) async {
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
  Future<bool> insertDiagramAsync({int diagramType = 0}) async {
    _pushUndo();
    _lastDiagramType = diagramType;
    _version++;
    return true;
  }

  @override
  Future<bool> insertChartAsync({int chartType = 0}) async {
    _pushUndo();
    _mockChartId = '00000000-0000-0000-0000-00000000c001';
    final kind = switch (chartType) {
      1 => 'bar',
      2 => 'line',
      3 => 'pie',
      _ => 'column',
    };
    _chartData = {
      'kind': kind,
      'categories': ['Category 1', 'Category 2', 'Category 3', 'Category 4'],
      'series': [
        {
          'name': 'Series 1',
          'values': [4.3, 2.5, 3.5, 4.5],
        },
        {
          'name': 'Series 2',
          'values': [2.4, 4.4, 1.8, 2.8],
        },
      ],
    };
    _version++;
    return true;
  }

  @override
  Future<bool> setChartDataAsync(String shapeId, Map<String, dynamic> chartData) async {
    if (_mockChartId == null || shapeId != _mockChartId) return false;
    _pushUndo();
    _chartData = Map<String, dynamic>.from(chartData);
    _version++;
    return true;
  }

  @override
  Future<bool> insertOfficeMathAsync({
    required String runId,
    required int offset,
    required String xml,
  }) async {
    _pushUndo();
    final id = offset == 0 && _bufferForRun(runId).isEmpty
        ? runId
        : '00000000-0000-0000-0000-${(_officeMathXml.length + 30).toString().padLeft(12, '0')}';
    _mockOfficeMathRunId = id;
    _officeMathXml[id] = xml;
    _version++;
    return true;
  }

  @override
  Future<bool> insertOfficeMathDisplayAsync({
    String? caretRunId,
    required String xml,
  }) async {
    _pushUndo();
    final id = '00000000-0000-0000-0000-${(_officeMathXml.length + 40).toString().padLeft(12, '0')}';
    _mockOfficeMathRunId = id;
    _officeMathXml[id] = xml;
    _version++;
    return true;
  }

  @override
  Future<bool> setOfficeMathAsync(String runId, String xml) async {
    if (!_officeMathXml.containsKey(runId)) return false;
    _pushUndo();
    _officeMathXml[runId] = xml;
    _mockOfficeMathRunId = runId;
    _version++;
    return true;
  }

  @override
  Future<bool> deleteBlockAsync(String blockId) async {
    _pushUndo();
    _version++;
    return true;
  }

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
  Future<bool> setImageAltTextAsync(String imageId, String? altText) async {
    if (_mockImageId == null) return false;
    _pushUndo();
    _imageAltText = (altText ?? '').trim();
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
  List<String>? spellCheckMisspellings() {
    final issues = spellCheckIssues();
    return issues?.map((issue) => issue.word).toList();
  }

  @override
  List<SpellIssue>? spellCheckIssues() {
    final found = <SpellIssue>[];
    final seen = <String>{};
    for (final match in RegExp(r"[A-Za-z']+").allMatches(_text)) {
      final word = match.group(0)!;
      final lower = word.toLowerCase();
      if (!_spellWords.contains(lower) && seen.add(word)) {
        final suggestions = switch (lower) {
          'teh' => const ['the'],
          'recieved' => const ['received'],
          _ => const <String>[],
        };
        found.add(SpellIssue(word: word, suggestions: suggestions));
      }
    }
    return found;
  }

  @override
  List<String>? grammarCheckIssues() {
    final issues = <String>[];
    final lower = ' $_text '.toLowerCase();
    if (lower.contains(' could of ')) {
      issues.add('Use "could have" instead of "could of"');
    }
    if (lower.contains('  ')) {
      issues.add('Remove extra space');
    }
    if (lower.contains(' alot ')) {
      issues.add('Use "a lot" instead of "alot"');
    }
    return issues;
  }

  final List<_MockFormatSpan> _formatSpans = [];

  void setFormatSpanForTest({
    required int start,
    required int end,
    bool? bold,
    String? styleName,
  }) {
    _formatSpans.add(_MockFormatSpan(start: start, end: end, bold: bold, styleName: styleName));
  }

  List<FindMatch> _formatOnlyMatches(FindFormatFilter filter) {
    final matches = <FindMatch>[];
    for (final span in _formatSpans) {
      if (_spanMatchesFilter(span, filter)) {
        matches.add(FindMatch(runId: defaultRunId, start: span.start, end: span.end));
      }
    }
    return matches;
  }

  bool _spanMatchesFilter(_MockFormatSpan span, FindFormatFilter filter) {
    if (filter.bold != null && span.bold != filter.bold) return false;
    if (filter.styleName != null &&
        filter.styleName!.isNotEmpty &&
        span.styleName?.toLowerCase() != filter.styleName!.toLowerCase()) {
      return false;
    }
    return true;
  }

  bool _rangeMatchesFormat(int start, int end, FindFormatFilter filter) {
    if (!filter.isActive) return true;
    for (final span in _formatSpans) {
      if (start >= span.start && end <= span.end && _spanMatchesFilter(span, filter)) {
        return true;
      }
    }
    return false;
  }

  @override
  List<FindMatch>? findMatches(
    String query,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
    FindFormatFilter formatFilter = FindFormatFilter.none,
  }) {
    if (query.isEmpty && !formatFilter.isActive) return [];
    if (query.isEmpty && formatFilter.isActive) {
      return _formatOnlyMatches(formatFilter);
    }
    if (useRegex || useWildcards) {
      final pattern = useWildcards ? _wildcardToRegex(query) : query;
      RegExp re;
      try {
        re = RegExp(pattern, caseSensitive: matchCase);
      } catch (_) {
        return null;
      }
      final matches = <FindMatch>[];
      for (final m in re.allMatches(_text)) {
        if (_rangeMatchesFormat(m.start, m.end, formatFilter)) {
          matches.add(FindMatch(runId: defaultRunId, start: m.start, end: m.end));
        }
      }
      return matches;
    }
    final haystack = matchCase ? _text : _text.toLowerCase();
    final needle = matchCase ? query : query.toLowerCase();
    final matches = <FindMatch>[];
    var from = 0;
    while (from <= haystack.length - needle.length) {
      final idx = haystack.indexOf(needle, from);
      if (idx < 0) break;
      final end = idx + query.length;
      if (_rangeMatchesFormat(idx, end, formatFilter)) {
        matches.add(FindMatch(runId: defaultRunId, start: idx, end: end));
      }
      from = idx + 1;
    }
    return matches;
  }

  String _wildcardToRegex(String pattern) {
    final buffer = StringBuffer('(?:');
    for (final ch in pattern.split('')) {
      switch (ch) {
        case '?':
          buffer.write('.');
        case '*':
          buffer.write('.*?');
        case r'\':
        case '.':
        case '+':
        case '^':
        case r'$':
        case '|':
        case '(':
        case ')':
        case '[':
        case ']':
        case '{':
        case '}':
          buffer
            ..write(r'\')
            ..write(ch);
        default:
          buffer.write(ch);
      }
    }
    buffer.write(')');
    return buffer.toString();
  }

  @override
  Future<int?> replaceAll(
    String find,
    String replace,
    bool matchCase, {
    bool useRegex = false,
    bool useWildcards = false,
  }) async {
    if (find.isEmpty) return null;
    final matches = findMatches(
          find,
          matchCase,
          useRegex: useRegex,
          useWildcards: useWildcards,
        ) ??
        const [];
    if (matches.isEmpty) return 0;
    if (useRegex || useWildcards) {
      final pattern = useWildcards ? _wildcardToRegex(find) : find;
      final re = RegExp(pattern, caseSensitive: matchCase);
      _text = _text.replaceAll(re, replace);
      return matches.length;
    }
    final haystack = matchCase ? _text : _text.toLowerCase();
    final needle = matchCase ? find : find.toLowerCase();
    final out = StringBuffer();
    var from = 0;
    var replacements = 0;
    while (from < _text.length) {
      final idx = haystack.indexOf(needle, from);
      if (idx < 0) {
        out.write(_text.substring(from));
        break;
      }
      out.write(_text.substring(from, idx));
      out.write(replace);
      replacements++;
      from = idx + find.length;
    }
    _text = out.toString();
    return replacements;
  }

  @override
  String? compareDocumentText(String otherText) {
    final left = _text.split('\n');
    final right = otherText.split('\n');
    var insertions = 0;
    var deletions = 0;
    final max = left.length > right.length ? left.length : right.length;
    for (var i = 0; i < max; i++) {
      final l = i < left.length ? left[i] : null;
      final r = i < right.length ? right[i] : null;
      if (l == r) continue;
      if (l != null) deletions++;
      if (r != null) insertions++;
    }
    return 'insertions:$insertions deletions:$deletions';
  }

  @override
  bool setReadOnlyEnabled(bool enabled) {
    _readOnly = enabled;
    return true;
  }

  @override
  bool setEncryptionPassword(String? password) {
    _encryptionPassword =
        (password == null || password.isEmpty) ? null : password;
    return true;
  }

  @override
  bool setTrackChangesEnabled(bool enabled) {
    _trackChanges = enabled;
    return true;
  }

  @override
  bool acceptAllRevisions() {
    _trackedRevisions.clear();
    return true;
  }

  @override
  bool rejectAllRevisions() {
    for (final rev in _trackedRevisions.reversed) {
      _deleteRange(rev.runId, rev.offset, rev.offset + rev.text.length);
    }
    _trackedRevisions.clear();
    return true;
  }

  int? _revisionIndexAt(String? caretRunId, [int? caretOffset]) {
    if (caretRunId == null) return null;
    for (var i = 0; i < _trackedRevisions.length; i++) {
      final rev = _trackedRevisions[i];
      if (rev.runId != caretRunId) continue;
      if (caretOffset == null) return i;
      final end = rev.offset + rev.text.length;
      if (caretOffset >= rev.offset && caretOffset <= end) return i;
    }
    return null;
  }

  @override
  bool acceptRevisionAtCaret({String? caretRunId}) {
    final idx = _revisionIndexAt(caretRunId);
    if (idx == null) return false;
    _trackedRevisions.removeAt(idx);
    return true;
  }

  @override
  bool rejectRevisionAtCaret({String? caretRunId}) {
    final idx = _revisionIndexAt(caretRunId);
    if (idx == null) return false;
    final rev = _trackedRevisions.removeAt(idx);
    final buffer = _bufferForRun(rev.runId);
    final lo = rev.offset.clamp(0, buffer.length);
    final hi = (rev.offset + rev.text.length).clamp(0, buffer.length);
    if (lo < hi) {
      _setBufferForRun(
        rev.runId,
        buffer.substring(0, lo) + buffer.substring(hi),
      );
      _version++;
    }
    return true;
  }

  @override
  String? adjacentRevisionRunId(String? caretRunId, {required bool forward}) {
    if (caretRunId == null || _trackedRevisions.isEmpty) return null;
    final ids = _trackedRevisions.map((r) => r.runId).toList();
    final idx = ids.indexOf(caretRunId);
    if (idx >= 0) {
      final next = forward
          ? (idx + 1) % ids.length
          : (idx == 0 ? ids.length - 1 : idx - 1);
      return ids[next];
    }
    return forward ? ids.first : ids.last;
  }
}

class _TrackedRevision {
  const _TrackedRevision({
    required this.runId,
    required this.offset,
    required this.text,
  });

  final String runId;
  final int offset;
  final String text;
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
    'No Spacing': _ResolvedStyle(
      charFormat: {..._baseChar, 'font_size': 12.0, 'bold': false, 'italic': false},
      paraFormat: {
        ..._basePara,
        'space_before': 0.0,
        'space_after': 0.0,
        'line_spacing': 1.0,
      },
    ),
  };
}

class _ResolvedStyle {
  const _ResolvedStyle({required this.charFormat, required this.paraFormat});

  final Map<String, dynamic> charFormat;
  final Map<String, dynamic> paraFormat;
}

class _MockFormatSpan {
  const _MockFormatSpan({
    required this.start,
    required this.end,
    this.bold,
    this.styleName,
  });

  final int start;
  final int end;
  final bool? bold;
  final String? styleName;
}
