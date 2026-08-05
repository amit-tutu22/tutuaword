import 'dart:io';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/native_engine.dart';
import 'package:tutuaword/bridge/twdoc_io.dart';
import 'package:tutuaword/editor/display_list.dart';

/// Editor controller — bridges Flutter UI to Rust engine (or mock for dev).
class EditorController extends ChangeNotifier {
  EditorController() {
    _engine = NativeEngine.load();
    _statusText = _engine == null
        ? 'Mock mode (build libtw_ffi to enable Rust engine)'
        : 'Rust engine connected';
    // Glyph-first when the engine is present; TextField fallback otherwise.
    _preferTextRendering = _engine == null;
    _recomputePageCount();
  }

  static const _pageMargin = 72.0;
  static const _lineHeightFactor = 1.4;
  static const _avgCharWidthFactor = 0.52;

  NativeEngine? _engine;
  String _statusText = '';
  String _documentText = '';
  String? _currentPath;
  int _displayVersion = 0;
  double _pageWidth = 612;
  double _pageHeight = 792;
  Uint8List _displayListBytes = Uint8List(0);
  bool _bold = false;
  bool _italic = false;
  bool _underline = false;
  int _pageCount = 1;
  int _currentPage = 0;
  bool _printPreview = false;
  bool _preferTextRendering = false;
  bool _trackChanges = false;
  List<String> _spellMisspellings = const [];
  String _fontFamily = 'Calibri';
  double _fontSize = 11;
  TextAlign _alignment = TextAlign.left;
  bool _strikethrough = false;
  bool _subscript = false;
  bool _superscript = false;
  double _zoom = 1.0;
  bool _showRuler = false;
  bool _showNavigationPane = false;
  String? _infoMessage;
  String? _caretRunId;
  int _caretOffset = 0;
  CaretGeometry? _caretGeometry;
  String? _selAnchorRunId;
  int _selAnchorOffset = 0;
  double _selAnchorX = 0;
  double _selAnchorY = 0;
  String? _selFocusRunId;
  int _selFocusOffset = 0;
  double _selFocusX = 0;
  double _selFocusY = 0;
  int _selPage = 0;
  List<GlyphSelectionRect> _selectionRects = const [];
  TextEditingController? _textController;
  FocusNode? _textFocusNode;

  /// Active page text field — used for cut/copy/paste and Select All.
  void attachTextEditor(TextEditingController controller, FocusNode focusNode) {
    _textController = controller;
    _textFocusNode = focusNode;
  }

  void detachTextEditor(TextEditingController controller) {
    if (_textController == controller) {
      _textController = null;
      _textFocusNode = null;
    }
  }

  TextEditingController? get textController => _textController;

  String get selectedText {
    final editor = _textController;
    if (editor == null || !editor.selection.isValid || editor.selection.isCollapsed) {
      return '';
    }
    return editor.text.substring(editor.selection.start, editor.selection.end);
  }

  bool get canCutOrCopy => selectedText.isNotEmpty;

  Future<void> copySelection() async {
    final text = selectedText;
    if (text.isEmpty) return;
    await Clipboard.setData(ClipboardData(text: text));
  }

  Future<void> cutSelection() async {
    if (!canCutOrCopy) return;
    final text = selectedText;
    await Clipboard.setData(ClipboardData(text: text));
    _replaceSelection('');
  }

  Future<void> paste({bool plainText = false}) async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    final text = data?.text;
    if (text == null || text.isEmpty) return;
    _replaceSelection(plainText ? text : text);
    _textFocusNode?.requestFocus();
  }

  void selectAll() {
    final editor = _textController;
    if (editor == null) return;
    editor.selection = TextSelection(baseOffset: 0, extentOffset: editor.text.length);
    _textFocusNode?.requestFocus();
  }

  void deleteSelection() {
    if (!canCutOrCopy) return;
    _replaceSelection('');
  }

  void _replaceSelection(String replacement) {
    final editor = _textController;
    if (editor == null || !editor.selection.isValid) return;
    final selection = editor.selection;
    final updated = editor.text.replaceRange(selection.start, selection.end, replacement);
    final cursor = selection.start + replacement.length;
    editor.value = editor.value.copyWith(
      text: updated,
      selection: TextSelection.collapsed(offset: cursor),
      composing: TextRange.empty,
    );
    replacePageText(_currentPage, updated);
  }

  /// Merge edited page text back into the full document.
  void replacePageText(int pageIndex, String newPageText) {
    if (_pageCount <= 1) {
      _documentText = newPageText;
      _recomputePageCount();
      notifyListeners();
      return;
    }

    final allLines = _wrapDocumentLines(_documentText);
    final linesPerPage = _linesPerPage();
    final start = pageIndex * linesPerPage;
    final end = (start + linesPerPage).clamp(0, allLines.length);
    final newLines = newPageText.split('\n');
    final merged = [
      ...allLines.sublist(0, start),
      ...newLines,
      if (end < allLines.length) ...allLines.sublist(end),
    ];
    _documentText = merged.join('\n');
    _recomputePageCount();
    notifyListeners();
  }

  String get statusText {
    final path = _currentPath == null ? '' : ' · ${_currentPath!.split(Platform.pathSeparator).last}';
    return '$_statusText$path · ${_documentText.length} chars · page ${_currentPage + 1}/$_pageCount · v$_displayVersion';
  }

  String get documentText => _documentText;
  Uint8List get displayListBytes => _displayListBytes;
  double get pageWidth => _pageWidth;
  double get pageHeight => _pageHeight;
  int get displayVersion => _displayVersion;
  int get pageCount => _pageCount;
  int get currentPage => _currentPage;
  bool get printPreview => _printPreview;
  bool get preferTextRendering => _preferTextRendering;
  bool get trackChanges => _trackChanges;
  List<String> get spellMisspellings => _spellMisspellings;
  bool get bold => _bold;
  bool get italic => _italic;
  bool get underline => _underline;
  bool get isEngineConnected => _engine != null;
  String get fontFamily => _fontFamily;
  double get fontSize => _fontSize;
  TextAlign get alignment => _alignment;
  bool get strikethrough => _strikethrough;
  bool get subscript => _subscript;
  bool get superscript => _superscript;
  double get zoom => _zoom;
  bool get showRuler => _showRuler;
  bool get showNavigationPane => _showNavigationPane;
  String? get infoMessage => _infoMessage;
  CaretGeometry? get caretGeometry => _caretGeometry;
  List<GlyphSelectionRect> get selectionRects => _selectionRects;
  bool get hasGlyphSelection {
    if (_selAnchorRunId == null || _selFocusRunId == null) return false;
    return _selAnchorRunId != _selFocusRunId || _selAnchorOffset != _selFocusOffset;
  }
  String? get caretRunId => _caretRunId;
  int get caretOffset => _caretOffset;

  String get documentTitle {
    if (_currentPath == null) return 'Document1';
    final parts = _currentPath!.split(Platform.pathSeparator);
    final name = parts.last;
    final dot = name.lastIndexOf('.');
    return dot == -1 ? name : name.substring(0, dot);
  }

  int get wordCount {
    if (_documentText.trim().isEmpty) return 0;
    return _documentText.trim().split(RegExp(r'\s+')).length;
  }

  /// Text content for a specific page (paginated by printable area and font metrics).
  String textForPage(int pageIndex) {
    if (_documentText.isEmpty) return '';
    if (_pageCount <= 1) return _documentText;

    final lines = _wrapDocumentLines(_documentText);
    final linesPerPage = _linesPerPage();
    final start = pageIndex * linesPerPage;
    if (start >= lines.length) return '';
    final end = (start + linesPerPage).clamp(0, lines.length);
    return lines.sublist(start, end).join('\n');
  }

  int _linesPerPage() {
    final contentHeight = _pageHeight - (_pageMargin * 2);
    final lineHeight = _fontSize * _lineHeightFactor;
    return (contentHeight / lineHeight).floor().clamp(1, 999);
  }

  int _charsPerLine() {
    final contentWidth = _pageWidth - (_pageMargin * 2);
    final avgCharWidth = _fontSize * _avgCharWidthFactor;
    return (contentWidth / avgCharWidth).floor().clamp(20, 200);
  }

  List<String> _wrapDocumentLines(String text) {
    final charsPerLine = _charsPerLine();
    final result = <String>[];
    for (final paragraph in text.split('\n')) {
      if (paragraph.isEmpty) {
        result.add('');
        continue;
      }
      var remaining = paragraph;
      while (remaining.isNotEmpty) {
        if (remaining.length <= charsPerLine) {
          result.add(remaining);
          break;
        }
        var breakAt = charsPerLine;
        final space = remaining.lastIndexOf(' ', breakAt);
        if (space > charsPerLine ~/ 3) {
          breakAt = space;
        }
        result.add(remaining.substring(0, breakAt).trimRight());
        remaining = remaining.substring(breakAt).trimLeft();
      }
    }
    return result;
  }

  void setCurrentPage(int page) {
    _currentPage = page.clamp(0, _pageCount - 1);
    if (_engine != null) {
      _engine!.setCurrentPageIndex(_currentPage);
      _refreshFromEngine();
    }
    notifyListeners();
  }

  /// Records which page the reader has scrolled to.
  ///
  /// Continuous scrolling renders every page at once, so this only tracks the
  /// page for the status bar and navigator — no engine round-trip.
  void setVisiblePage(int page) {
    final clamped = page.clamp(0, _pageCount - 1);
    if (clamped == _currentPage) return;
    _currentPage = clamped;
    notifyListeners();
  }

  /// Display list bytes for [page], cached per layout version.
  Uint8List displayListForPage(int page) {
    if (page == _currentPage && _displayListBytes.isNotEmpty) {
      return _displayListBytes;
    }
    if (_engine == null) return Uint8List(0);

    final cached = _pageDisplayLists[page];
    if (cached != null) return cached;

    final bytes = _engine!.fetchPageDisplayList(page) ?? Uint8List(0);
    _pageDisplayLists[page] = bytes;
    return bytes;
  }

  final Map<int, Uint8List> _pageDisplayLists = {};

  void togglePrintPreview() {
    _printPreview = !_printPreview;
    notifyListeners();
  }

  void insertCharacter(String char) {
    if (usesGlyphRendering) {
      insertGlyphCharacter(char);
      return;
    }
    if (_engine != null) {
      _engine!.insertText('00000000-0000-0000-0000-000000000004', _documentText.length, char);
      _refreshFromEngine();
    } else {
      _documentText += char;
      _recomputePageCount();
    }
    notifyListeners();
  }

  void deleteBackward() {
    if (usesGlyphRendering) {
      deleteGlyphBackward();
      return;
    }
    if (_documentText.isEmpty) return;
    _documentText = _documentText.substring(0, _documentText.length - 1);
    _recomputePageCount();
    notifyListeners();
  }

  void hitTestAt(int pageIndex, double x, double y) {
    if (_engine == null) return;
    final result = _engine!.hitTestPage(pageIndex, x, y);
    if (result == null) return;
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _caretGeometry = _engine!.caretGeometryAt(pageIndex, x, y);
    // Collapse selection to the caret.
    _selPage = pageIndex;
    _selAnchorRunId = result.runId;
    _selAnchorOffset = result.charOffset;
    _selAnchorX = x;
    _selAnchorY = y;
    _selFocusRunId = result.runId;
    _selFocusOffset = result.charOffset;
    _selFocusX = x;
    _selFocusY = y;
    _selectionRects = const [];
    notifyListeners();
  }

  void beginGlyphSelection(int pageIndex, double x, double y) {
    hitTestAt(pageIndex, x, y);
  }

  void updateGlyphSelection(int pageIndex, double x, double y) {
    if (_engine == null || _selAnchorRunId == null) return;
    final result = _engine!.hitTestPage(pageIndex, x, y);
    if (result == null) return;
    _selPage = pageIndex;
    _selFocusRunId = result.runId;
    _selFocusOffset = result.charOffset;
    _selFocusX = x;
    _selFocusY = y;
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _caretGeometry = _engine!.caretGeometryAt(pageIndex, x, y);
    _selectionRects = _engine!.selectionRectsOnPage(
      pageIndex,
      _selAnchorX,
      _selAnchorY,
      _selFocusX,
      _selFocusY,
    );
    notifyListeners();
  }

  void endGlyphSelection(int pageIndex, double x, double y) {
    updateGlyphSelection(pageIndex, x, y);
  }

  void insertGlyphCharacter(String char) {
    if (_engine == null) return;
    final runId = _caretRunId ?? _defaultRunId();
    if (runId == null) return;
    _engine!.insertText(runId, _caretOffset, char);
    _caretOffset += char.length;
    _selAnchorRunId = runId;
    _selAnchorOffset = _caretOffset;
    _selFocusRunId = runId;
    _selFocusOffset = _caretOffset;
    _selectionRects = const [];
    _refreshFromEngine();
    _updateRenderModeAfterEngineOpen();
    notifyListeners();
  }

  void deleteGlyphBackward() {
    if (_engine == null) return;
    if (hasGlyphSelection) {
      _deleteGlyphSelection();
      return;
    }
    if (_caretOffset <= 0) return;
    final runId = _caretRunId ?? _defaultRunId();
    if (runId == null) return;
    _engine!.deleteRange(runId, _caretOffset - 1, _caretOffset);
    _caretOffset = (_caretOffset - 1).clamp(0, 1 << 30);
    _selAnchorRunId = runId;
    _selAnchorOffset = _caretOffset;
    _selFocusRunId = runId;
    _selFocusOffset = _caretOffset;
    _selectionRects = const [];
    _refreshFromEngine();
    notifyListeners();
  }

  void _deleteGlyphSelection() {
    // Single-run selection delete; multi-run delete is deferred.
    if (_engine == null || _selAnchorRunId == null || _selFocusRunId == null) return;
    if (_selAnchorRunId != _selFocusRunId) {
      // Collapse and delete one char at focus for now.
      final runId = _selFocusRunId!;
      final offset = _selFocusOffset;
      if (offset > 0) {
        _engine!.deleteRange(runId, offset - 1, offset);
        _caretOffset = offset - 1;
      }
    } else {
      final start = _selAnchorOffset < _selFocusOffset ? _selAnchorOffset : _selFocusOffset;
      final end = _selAnchorOffset < _selFocusOffset ? _selFocusOffset : _selAnchorOffset;
      if (start < end) {
        _engine!.deleteRange(_selAnchorRunId!, start, end);
        _caretOffset = start;
      }
    }
    _selAnchorOffset = _caretOffset;
    _selFocusOffset = _caretOffset;
    _selAnchorRunId = _caretRunId;
    _selFocusRunId = _caretRunId;
    _selectionRects = const [];
    _refreshFromEngine();
    notifyListeners();
  }

  String? _defaultRunId() {
    if (_documentText.isEmpty) return null;
    return '00000000-0000-0000-0000-000000000004';
  }

  (String, int, String, int)? _formatRange() {
    final runId = _caretRunId ?? _defaultRunId();
    if (runId == null) return null;
    if (hasGlyphSelection && _selAnchorRunId != null && _selFocusRunId != null) {
      return (
        _selAnchorRunId!,
        _selAnchorOffset,
        _selFocusRunId!,
        _selFocusOffset,
      );
    }
    return (runId, _caretOffset, runId, _caretOffset);
  }

  void _applyCharFormatJson(String json) {
    final range = _formatRange();
    if (range == null || _engine == null || !usesGlyphRendering) return;
    final (startRun, startOff, endRun, endOff) = range;
    _engine!.applyCharFormatJson(
      startRunId: startRun,
      startOffset: startOff,
      endRunId: endRun,
      endOffset: endOff,
      formatJson: json,
    );
    _refreshFromEngine();
  }

  void _applyParaFormatJson(String json) {
    final range = _formatRange();
    if (range == null || _engine == null || !usesGlyphRendering) return;
    final (startRun, startOff, endRun, endOff) = range;
    _engine!.applyParaFormatJson(
      startRunId: startRun,
      startOffset: startOff,
      endRunId: endRun,
      endOffset: endOff,
      formatJson: json,
    );
    _refreshFromEngine();
  }

  void toggleBold() {
    _bold = !_bold;
    if (usesGlyphRendering && _engine != null) {
      _applyCharFormatJson('{"bold":$_bold}');
    }
    notifyListeners();
  }

  void toggleItalic() {
    _italic = !_italic;
    if (usesGlyphRendering && _engine != null) {
      _applyCharFormatJson('{"italic":$_italic}');
    }
    notifyListeners();
  }

  void toggleUnderline() {
    _underline = !_underline;
    if (usesGlyphRendering && _engine != null) {
      _applyCharFormatJson(_underline ? '{"underline":"Single"}' : '{"underline":"None"}');
    }
    notifyListeners();
  }

  void setFontFamily(String family) {
    _fontFamily = family;
    if (usesGlyphRendering && _engine != null) {
      final escaped = family.replaceAll(r'\', r'\\').replaceAll('"', r'\"');
      _applyCharFormatJson('{"font_family":"$escaped"}');
    }
    notifyListeners();
  }

  void setFontSize(double size) {
    _fontSize = size.clamp(6, 96);
    if (usesGlyphRendering && _engine != null) {
      _applyCharFormatJson('{"font_size":$_fontSize}');
    } else if (_preferTextRendering) {
      _recomputePageCount();
    }
    notifyListeners();
  }

  void increaseFontSize() {
    setFontSize(_fontSize + 1);
  }

  void decreaseFontSize() {
    setFontSize(_fontSize - 1);
  }

  void setAlignment(TextAlign align) {
    _alignment = align;
    if (usesGlyphRendering && _engine != null) {
      final name = switch (align) {
        TextAlign.left || TextAlign.start => 'Left',
        TextAlign.center => 'Center',
        TextAlign.right || TextAlign.end => 'Right',
        TextAlign.justify => 'Justify',
      };
      _applyParaFormatJson('{"alignment":"$name"}');
    }
    notifyListeners();
  }

  void toggleStrikethrough() {
    _strikethrough = !_strikethrough;
    if (usesGlyphRendering && _engine != null) {
      _applyCharFormatJson('{"strikethrough":$_strikethrough}');
    }
    notifyListeners();
  }

  void toggleSubscript() {
    _subscript = !_subscript;
    if (_subscript) _superscript = false;
    if (usesGlyphRendering && _engine != null) {
      _applyCharFormatJson(
        '{"subscript":$_subscript,"superscript":false}',
      );
    }
    notifyListeners();
  }

  void toggleSuperscript() {
    _superscript = !_superscript;
    if (_superscript) _subscript = false;
    if (usesGlyphRendering && _engine != null) {
      _applyCharFormatJson(
        '{"superscript":$_superscript,"subscript":false}',
      );
    }
    notifyListeners();
  }

  void setZoom(double value) {
    _zoom = value.clamp(0.5, 3.0);
    notifyListeners();
  }

  void zoomIn() => setZoom(_zoom + 0.1);
  void zoomOut() => setZoom(_zoom - 0.1);

  void toggleRuler() {
    _showRuler = !_showRuler;
    notifyListeners();
  }

  void toggleNavigationPane() {
    _showNavigationPane = !_showNavigationPane;
    notifyListeners();
  }

  void clearInfoMessage() {
    _infoMessage = null;
    notifyListeners();
  }

  void insertTable() {
    if (_engine != null && _engine!.insertTableBlock(3, 3)) {
      _refreshFromEngine();
      _updateRenderModeAfterEngineOpen();
      _statusText = 'Table inserted (3×3)';
    } else {
      _statusText = 'Table inserted (3×3 — mock)';
    }
    notifyListeners();
  }

  void insertImage() {
    if (_engine != null && _engine!.insertImageBlock(200, 150)) {
      _refreshFromEngine();
      _updateRenderModeAfterEngineOpen();
      _statusText = 'Image placeholder inserted';
    } else {
      _statusText = 'Image placeholder inserted (mock)';
    }
    notifyListeners();
  }

  void applyHeading1() {
    if (_engine != null && _engine!.applyHeading1Style()) {
      _refreshFromEngine();
      _updateRenderModeAfterEngineOpen();
      _statusText = 'Heading 1 applied';
    } else {
      _statusText = 'Heading 1 applied (mock)';
    }
    notifyListeners();
  }

  void applyBulletList() {
    if (_engine != null && _engine!.applyBulletListStyle()) {
      _refreshFromEngine();
      _updateRenderModeAfterEngineOpen();
      _statusText = 'Bullet list applied';
    } else {
      _statusText = 'Bullet list applied (mock)';
    }
    notifyListeners();
  }

  Future<void> exportPdf() async {
    try {
      final path = await FilePicker.platform.saveFile(
        dialogTitle: 'Export PDF',
        fileName: 'document.pdf',
        type: FileType.custom,
        allowedExtensions: ['pdf'],
      );
      if (path == null) {
        _statusText = 'PDF export cancelled';
        notifyListeners();
        return;
      }

      final outPath = path.endsWith('.pdf') ? path : '$path.pdf';
      Uint8List? bytes;
      if (_engine != null) {
        bytes = _engine!.exportPdfBytes();
      }
      if (bytes == null || bytes.isEmpty) {
        _statusText = 'PDF export failed';
        notifyListeners();
        return;
      }
      await File(outPath).writeAsBytes(bytes);
      _statusText = 'PDF exported';
      notifyListeners();
    } catch (e) {
      _statusText = 'PDF export failed: $e';
      notifyListeners();
    }
  }

  void undo() {
    if (_engine != null && _engine!.undoEdit()) {
      _refreshFromEngine();
      _statusText = 'Undo';
    }
    notifyListeners();
  }

  void redo() {
    if (_engine != null && _engine!.redoEdit()) {
      _refreshFromEngine();
      _statusText = 'Redo';
    }
    notifyListeners();
  }

  Future<void> saveDocument() async {
    await _saveWithExtension(
      suggestedPath: _currentPath ?? 'document.twdoc',
      defaultExtension: _extensionFromPath(_currentPath) ?? 'twdoc',
      dialogTitle: 'Save document',
    );
  }

  Future<void> saveDocumentAs({required String extension}) async {
    final ext = extension.startsWith('.') ? extension.substring(1) : extension;
    await _saveWithExtension(
      suggestedPath: 'document.$ext',
      defaultExtension: ext,
      dialogTitle: 'Save document as',
      forceFormat: ext,
    );
  }

  Future<void> _saveWithExtension({
    required String suggestedPath,
    required String defaultExtension,
    required String dialogTitle,
    String? forceFormat,
  }) async {
    try {
      final path = await FilePicker.platform.saveFile(
        dialogTitle: dialogTitle,
        fileName: suggestedPath.split(Platform.pathSeparator).last,
        type: FileType.custom,
        allowedExtensions: [defaultExtension],
      );
      if (path == null) {
        _statusText = 'Save cancelled';
        notifyListeners();
        return;
      }

      final ext = forceFormat ?? defaultExtension;
      final outPath = path.endsWith('.$ext') ? path : '$path.$ext';
      Uint8List bytes;
      if (_engine != null) {
        if (forceFormat != null) {
          bytes = _engine!.saveDocumentAsBytes(ext) ?? Uint8List(0);
        } else {
          bytes = _engine!.saveDocumentBytes() ?? TwdocWriter.fromText(_documentText);
        }
        if (bytes.isEmpty) {
          bytes = TwdocWriter.fromText(_documentText);
        }
      } else {
        bytes = TwdocWriter.fromText(_documentText);
      }
      await File(outPath).writeAsBytes(bytes);
      _currentPath = outPath;
      _statusText = 'Saved';
      notifyListeners();
    } catch (e) {
      _statusText = 'Save failed: $e';
      notifyListeners();
    }
  }

  String? _extensionFromPath(String? path) {
    if (path == null) return null;
    final dot = path.lastIndexOf('.');
    if (dot == -1) return null;
    return path.substring(dot + 1).toLowerCase();
  }

  void toggleTrackChanges() {
    _trackChanges = !_trackChanges;
    if (_engine != null) {
      _engine!.setTrackChangesEnabled(_trackChanges);
    }
    _statusText = _trackChanges ? 'Track changes on' : 'Track changes off';
    notifyListeners();
  }

  Future<void> spellCheckDocument() async {
    if (_engine != null) {
      final words = _engine!.spellCheckMisspellings();
      if (words == null) {
        _statusText = 'Spell check failed';
        notifyListeners();
        return;
      }
      _spellMisspellings = words;
      _statusText = words.isEmpty
          ? 'No spelling issues found'
          : 'Spell check: ${words.length} issue(s)';
      if (words.isNotEmpty) {
        _infoMessage = 'Spell check found ${words.length} issue(s): ${words.take(5).join(", ")}';
      }
    } else {
      _spellMisspellings = const [];
      _statusText = 'Spell check (mock): no issues';
    }
    notifyListeners();
  }

  Future<void> openDocument() async {
    _statusText = 'Opening…';
    notifyListeners();
    try {
      final result = await FilePicker.platform.pickFiles(
        dialogTitle: 'Open document',
        type: FileType.custom,
        allowedExtensions: kSupportedOpenExtensions,
        withData: false,
      );
      if (result == null || result.files.isEmpty) {
        _statusText = 'Open cancelled';
        notifyListeners();
        return;
      }

      final file = result.files.single;
      final path = file.path;
      if (path == null) {
        _statusText = 'Open failed: no file path (try again)';
        notifyListeners();
        return;
      }

      final bytes = await File(path).readAsBytes();
      final textFromFile = DocumentReader.extractText(bytes, path: path);

      if (_engine != null && _engine!.openDocumentBytes(bytes, path: path)) {
        _currentPage = 0;
        _engine!.setCurrentPageIndex(0);
        _refreshFromEngine();
        _statusText = 'Opened';
      } else {
        _documentText = textFromFile;
        _displayListBytes = Uint8List(0);
        _displayVersion++;
        _recomputePageCount();
        _preferTextRendering = true;
        _statusText = _engine == null ? 'Opened (mock mode)' : 'Opened (mock fallback)';
      }

      _currentPath = path;
      _currentPage = 0;
      notifyListeners();
    } catch (e) {
      _statusText = 'Open failed: $e';
      notifyListeners();
    }
  }

  void _refreshFromEngine() {
    final data = _engine?.fetchDisplayList();
    if (data == null) return;
    if (data.version != _displayVersion) {
      _pageDisplayLists.clear();
    }
    _displayListBytes = data.bytes;
    _displayVersion = data.version;
    _pageWidth = data.pageWidth;
    _pageHeight = data.pageHeight;
    if (data.documentText.isNotEmpty || _documentText.isEmpty) {
      _documentText = data.documentText;
    }
    _updateRenderModeAfterEngineOpen();
    if (!_preferTextRendering) {
      _pageCount = data.pageCount.clamp(1, 9999);
    } else {
      _recomputePageCount();
    }
    if (_currentPage >= _pageCount) {
      _currentPage = _pageCount - 1;
    }
  }

  void _recomputePageCount() {
    if (_documentText.isEmpty) {
      _pageCount = 1;
      if (_currentPage >= _pageCount) {
        _currentPage = _pageCount - 1;
      }
      return;
    }
    final lines = _wrapDocumentLines(_documentText);
    final linesPerPage = _linesPerPage();
    _pageCount = (lines.length / linesPerPage).ceil().clamp(1, 9999);
    if (_currentPage >= _pageCount) {
      _currentPage = _pageCount - 1;
    }
  }

  void _updateRenderModeAfterEngineOpen() {
    if (_engineHasPaintableDisplayList()) {
      _preferTextRendering = false;
    } else {
      _preferTextRendering = true;
      _recomputePageCount();
    }
  }

  bool _engineHasPaintableDisplayList() {
    if (_displayListBytes.isEmpty) return false;
    final snapshot = DisplayListSnapshot.fromBytes(_displayListBytes);
    return snapshot.hasPaintableGlyphs || snapshot.hasPaintableContent;
  }

  /// Whether the active page should paint via Rust display list glyphs.
  bool get usesGlyphRendering => !_preferTextRendering && _engineHasPaintableDisplayList();

  /// Test hook: inject display list bytes and switch to glyph rendering mode.
  @visibleForTesting
  void setDisplayListForTest(
    Uint8List bytes, {
    bool preferTextRendering = false,
    int pageCount = 1,
  }) {
    _displayListBytes = bytes;
    _preferTextRendering = preferTextRendering;
    _displayVersion++;
    _pageDisplayLists.clear();
    if (!preferTextRendering) {
      _pageCount = pageCount;
      for (var page = 0; page < pageCount; page++) {
        _pageDisplayLists[page] = bytes;
      }
    }
    notifyListeners();
  }

  @override
  void dispose() {
    super.dispose();
  }
}
