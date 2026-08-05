import 'dart:convert';
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
    if (_engine != null) {
      _refreshFromEngine();
      _ensureGlyphCaret();
      _syncRibbonFromCaret();
    }
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
  String _activeParagraphStyle = 'Normal';
  double _indentLeft = 0;
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
    if (usesGlyphRendering && _engine != null && hasGlyphSelection) {
      return _glyphSelectedText();
    }
    final editor = _textController;
    if (editor == null || !editor.selection.isValid || editor.selection.isCollapsed) {
      return '';
    }
    return editor.text.substring(editor.selection.start, editor.selection.end);
  }

  String _glyphSelectedText() {
    if (_engine == null ||
        _selAnchorRunId == null ||
        _selFocusRunId == null ||
        !hasGlyphSelection) {
      return '';
    }
    return _engine!.fetchTextRange(
          _selAnchorRunId!,
          _selAnchorOffset,
          _selFocusRunId!,
          _selFocusOffset,
        ) ??
        '';
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
    if (usesGlyphRendering && _engine != null) {
      _deleteGlyphSelection();
      notifyListeners();
      return;
    }
    _replaceSelection('');
  }

  Future<void> paste({bool plainText = false}) async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    final text = data?.text;
    if (text == null || text.isEmpty) return;
    if (usesGlyphRendering && _engine != null) {
      if (hasGlyphSelection) {
        _deleteGlyphSelection();
      }
      final runId = _caretRunId ?? _defaultRunId();
      if (runId == null) return;
      if (!_engine!.tryInsertText(runId, _caretOffset, text)) return;
      _caretOffset += text.length;
      _selAnchorRunId = runId;
      _selAnchorOffset = _caretOffset;
      _selFocusRunId = runId;
      _selFocusOffset = _caretOffset;
      _selectionRects = const [];
      _refreshFromEngine();
      notifyListeners();
      return;
    }
    _replaceSelection(plainText ? text : text);
    _textFocusNode?.requestFocus();
  }

  void selectAll() {
    if (usesGlyphRendering && _engine != null) {
      final start = _engine!.hitTestPage(0, _pageMargin, _pageMargin + _fontSize);
      if (start == null) return;
      final lastPage = _pageCount - 1;
      final end = _engine!.hitTestPage(
        lastPage,
        _pageWidth - _pageMargin,
        _pageHeight - _pageMargin,
      );
      if (end == null) return;
      _selPage = lastPage;
      _selAnchorRunId = start.runId;
      _selAnchorOffset = start.charOffset;
      _selFocusRunId = end.runId;
      _selFocusOffset = end.charOffset;
      _caretRunId = end.runId;
      _caretOffset = end.charOffset;
      _selAnchorX = _pageMargin;
      _selAnchorY = _pageMargin + _fontSize;
      _selFocusX = _pageWidth - _pageMargin;
      _selFocusY = _pageHeight - _pageMargin;
      _selectionRects = _engine!.selectionRectsOnPage(
        _selPage,
        _selAnchorX,
        _selAnchorY,
        _selFocusX,
        _selFocusY,
      );
      notifyListeners();
      return;
    }
    final editor = _textController;
    if (editor == null) return;
    editor.selection = TextSelection(baseOffset: 0, extentOffset: editor.text.length);
    _textFocusNode?.requestFocus();
  }

  void deleteSelection() {
    if (!canCutOrCopy) return;
    if (usesGlyphRendering && _engine != null) {
      _deleteGlyphSelection();
      notifyListeners();
      return;
    }
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
  String get activeParagraphStyle => _activeParagraphStyle;
  double get indentLeft => _indentLeft;
  String? get infoMessage => _infoMessage;
  CaretGeometry? get caretGeometry => _caretGeometry;
  /// Page index (0-based) where the caret/selection is active.
  int get caretPage => _selPage;
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
    if (_engine == null) return Uint8List(0);

    final cached = _pageDisplayLists[page];
    if (cached != null) return cached;

    final bytes = _engine!.fetchPageDisplayList(page) ?? Uint8List(0);
    if (bytes.isNotEmpty) {
      _pageDisplayLists[page] = bytes;
    }
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
    _activateGlyphPage(pageIndex);
    var result = _engine!.hitTestPage(pageIndex, x, y);
    if (result == null) {
      _placeCaretOnEmptyPage(pageIndex, x, y);
      return;
    }
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _caretGeometry = _engine!.caretGeometryAt(pageIndex, x, y);
    // Collapse selection to the caret.
    _selAnchorRunId = result.runId;
    _selAnchorOffset = result.charOffset;
    _selAnchorX = x;
    _selAnchorY = y;
    _selFocusRunId = result.runId;
    _selFocusOffset = result.charOffset;
    _selFocusX = x;
    _selFocusY = y;
    _selectionRects = const [];
    _syncRibbonFromCaret();
    notifyListeners();
  }
  void _activateGlyphPage(int pageIndex) {
    _selPage = pageIndex;
    _engine?.setCurrentPageIndex(pageIndex);
  }

  /// Empty pages have no laid-out lines; anchor insertion at the document tail
  /// but draw the caret where the user clicked.
  void _placeCaretOnEmptyPage(int pageIndex, double x, double y) {
    HitTestResult? tail;
    for (var p = pageIndex; p >= 0; p--) {
      tail ??= _engine!.hitTestPage(
        p,
        _pageWidth - _pageMargin,
        _pageHeight - _pageMargin,
      );
    }
    tail ??= _engine!.hitTestPage(0, _pageMargin, _pageMargin + _fontSize);

    if (tail != null) {
      _caretRunId = tail.runId;
      _caretOffset = tail.charOffset;
      _selAnchorRunId = tail.runId;
      _selAnchorOffset = tail.charOffset;
      _selFocusRunId = tail.runId;
      _selFocusOffset = tail.charOffset;
    }

    final geom = tail != null
        ? _engine!.caretAtPosition(pageIndex, tail.runId, tail.charOffset)
        : null;
    _caretGeometry = geom ??
        CaretGeometry(
          x: x.clamp(_pageMargin, _pageWidth - _pageMargin),
          y: (y - _fontSize).clamp(_pageMargin, _pageHeight - _pageMargin),
          height: _fontSize * _lineHeightFactor,
        );
    _selAnchorX = _caretGeometry!.x;
    _selAnchorY = _caretGeometry!.y;
    _selFocusX = _caretGeometry!.x;
    _selFocusY = _caretGeometry!.y;
    _selectionRects = const [];
    _syncRibbonFromCaret();
    notifyListeners();
  }

  void moveGlyphCaretByArrow(LogicalKeyboardKey key) {
    if (_engine == null) return;
    if (_preferTextRendering) return; // TextField fallback handles arrows.
    if (_caretRunId == null) return;

    switch (key) {
      case LogicalKeyboardKey.arrowLeft:
        _moveGlyphCaretOffset(-1);
        return;
      case LogicalKeyboardKey.arrowRight:
        _moveGlyphCaretOffset(1);
        return;
      case LogicalKeyboardKey.arrowUp:
        _moveGlyphCaretUpDown(-1);
        return;
      case LogicalKeyboardKey.arrowDown:
        _moveGlyphCaretUpDown(1);
        return;
      default:
        return;
    }
  }

  void _moveGlyphCaretOffset(int delta) {
    final runId = _caretRunId;
    if (runId == null) return;

    final before = _engine!.caretAtPosition(_selPage, runId, _caretOffset);
    final candidate = (_caretOffset + delta).clamp(0, 1 << 30);
    if (candidate != _caretOffset) {
      final after = _engine!.caretAtPosition(_selPage, runId, candidate);
      if (after != null && !_sameCaretGeometry(before, after)) {
        _setGlyphCaret(runId, candidate, after);
        return;
      }
    }

    // At a run boundary — nudge horizontally and hit-test the adjacent position.
    if (before == null) return;
    final nudge = delta > 0 ? 2.0 : -2.0;
    final probeX = (before.x + nudge).clamp(_pageMargin, _pageWidth - _pageMargin);
    hitTestAt(_selPage, probeX, before.y);
  }

  bool _sameCaretGeometry(CaretGeometry? a, CaretGeometry b) {
    if (a == null) return false;
    const eps = 0.01;
    return (b.x - a.x).abs() < eps && (b.y - a.y).abs() < eps;
  }

  void _setGlyphCaret(String runId, int offset, CaretGeometry geometry) {
    _caretRunId = runId;
    _caretOffset = offset;
    _caretGeometry = geometry;
    _selAnchorRunId = runId;
    _selAnchorOffset = offset;
    _selFocusRunId = runId;
    _selFocusOffset = offset;
    _selectionRects = const [];
    _syncRibbonFromCaret();
    notifyListeners();
  }

  void _moveGlyphCaretUpDown(int direction) {
    if (_caretGeometry == null) return;
    final stepY = _fontSize * _lineHeightFactor;
    final newY = _caretGeometry!.y + stepY * direction;
    // Keep X stable so we land on the nearest glyph segment for that line.
    hitTestAt(_selPage, _caretGeometry!.x, newY);
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
    _syncRibbonFromCaret();
  }

  void insertGlyphCharacter(String char) {
    if (_engine == null) return;
    // Never insert control characters as glyphs (Enter used to produce tofu □).
    if (char == '\n' || char == '\r' || char.codeUnitAt(0) < 0x20) {
      return;
    }
    final runId = _caretRunId ?? _defaultRunId();
    if (runId == null) return;
    final oldCaretX = _caretGeometry?.x;
    final oldCaretOffset = _caretOffset;
    if (!_engine!.tryInsertText(runId, _caretOffset, char)) {
      return;
    }
    _caretOffset += char.length;
    _selAnchorRunId = runId;
    _selAnchorOffset = _caretOffset;
    _selFocusRunId = runId;
    _selFocusOffset = _caretOffset;
    _selectionRects = const [];
    _refreshFromEngine();
    // Whitespace can advance layout without producing a visible glyph, which
    // can leave caret-x effectively unchanged. Ensure the caret visibly
    // advances immediately after inserting a space.
    if (usesGlyphRendering &&
        oldCaretX != null &&
        oldCaretOffset != _caretOffset &&
        char.trim().isEmpty &&
        _caretGeometry != null &&
        (_caretGeometry!.x - oldCaretX).abs() < 0.5) {
      final approxSpace = _fontSize * _avgCharWidthFactor;
      _caretGeometry = CaretGeometry(
        x: (oldCaretX + approxSpace).clamp(0.0, _pageWidth),
        y: _caretGeometry!.y,
        height: _caretGeometry!.height,
      );
    }
    notifyListeners();
  }

  /// Word Enter: split the current paragraph at the caret.
  void insertGlyphParagraphBreak() {
    if (_engine == null) return;
    final runId = _caretRunId ?? _defaultRunId();
    if (runId == null) return;
    final prevY = _caretGeometry?.y ?? (_pageMargin + _fontSize);
    final prevX = _caretGeometry?.x ?? _pageMargin;
    _engine!.splitParagraphAt(runId, _caretOffset);
    _selectionRects = const [];
    _refreshFromEngine();
    // Place caret on the new paragraph (line below the previous caret).
    final nextY = prevY + _fontSize * _lineHeightFactor;
    hitTestAt(_selPage, prevX.clamp(_pageMargin, _pageWidth - _pageMargin), nextY);
    notifyListeners();
  }

  void deleteGlyphBackward() {
    if (_engine == null) return;
    if (hasGlyphSelection) {
      _deleteGlyphSelection();
      return;
    }
    final runId = _caretRunId ?? _defaultRunId();
    if (runId == null) return;
    if (_caretOffset > 0) {
      _engine!.deleteRange(runId, _caretOffset - 1, _caretOffset);
      _caretOffset = _caretOffset - 1;
      _selAnchorRunId = runId;
      _selAnchorOffset = _caretOffset;
      _selFocusRunId = runId;
      _selFocusOffset = _caretOffset;
      _selectionRects = const [];
      _refreshFromEngine();
      notifyListeners();
      return;
    }

    // At a run boundary — delete the preceding character via cross-run range.
    final geom = _engine!.caretAtPosition(_selPage, runId, 0);
    if (geom == null) return;
    final probeX = (geom.x - 2.0).clamp(_pageMargin, _pageWidth - _pageMargin);
    final prev = _engine!.hitTestPage(_selPage, probeX, geom.y);
    if (prev == null) return;
    if (prev.runId == runId && prev.charOffset == 0) return;

    final startRun = prev.runId;
    final startOff = prev.charOffset > 0 ? prev.charOffset - 1 : 0;
    if (!_engine!.deleteDocRange(startRun, startOff, runId, _caretOffset)) return;

    _caretRunId = startRun;
    _caretOffset = startOff;
    _selAnchorRunId = startRun;
    _selAnchorOffset = startOff;
    _selFocusRunId = startRun;
    _selFocusOffset = startOff;
    _selectionRects = const [];
    _refreshFromEngine();
    _syncCaretGeometry();
    notifyListeners();
  }

  void _deleteGlyphSelection() {
    if (_engine == null || _selAnchorRunId == null || _selFocusRunId == null) return;
    final anchorRun = _selAnchorRunId!;
    final anchorOff = _selAnchorOffset;
    final focusRun = _selFocusRunId!;
    final focusOff = _selFocusOffset;

    if (anchorRun == focusRun) {
      final start = anchorOff < focusOff ? anchorOff : focusOff;
      final end = anchorOff < focusOff ? focusOff : anchorOff;
      if (start >= end) return;
      if (!_engine!.deleteRange(anchorRun, start, end)) return;
      _caretRunId = anchorRun;
      _caretOffset = start;
    } else {
      if (!_engine!.deleteDocRange(anchorRun, anchorOff, focusRun, focusOff)) return;
      _caretRunId = anchorRun;
      _caretOffset = anchorOff;
    }

    _selAnchorRunId = _caretRunId;
    _selAnchorOffset = _caretOffset;
    _selFocusRunId = _caretRunId;
    _selFocusOffset = _caretOffset;
    _selectionRects = const [];
    _refreshFromEngine();
    _syncCaretGeometry();
    notifyListeners();
  }

  String? _defaultRunId() {
    if (_caretRunId != null) return _caretRunId;
    _ensureGlyphCaret();
    return _caretRunId;
  }

  /// Place the caret on the first editable run if none is set yet.
  void ensureGlyphCaret() {
    if (_caretRunId != null) return;
    _ensureGlyphCaret();
    if (_caretRunId != null) notifyListeners();
  }

  void _ensureGlyphCaret() {
    if (_engine == null || _caretRunId != null) return;
    _selPage = 0;
    _engine!.setCurrentPageIndex(0);
    final result = _engine!.hitTestPage(_selPage, _pageMargin, _pageMargin + _fontSize);
    if (result == null) return;
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _selAnchorRunId = result.runId;
    _selAnchorOffset = result.charOffset;
    _selFocusRunId = result.runId;
    _selFocusOffset = result.charOffset;
    _caretGeometry = _engine!.caretGeometryAt(
      _selPage,
      _pageMargin,
      _pageMargin + _fontSize,
    );
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
      _applyCharFormatJson(jsonEncode({'font_family': family}));
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

  void clearFormatting() {
    final range = _formatRange();
    if (range == null || _engine == null || !usesGlyphRendering) return;
    final (startRun, startOff, endRun, endOff) = range;
    if (_engine!.clearFormat(startRun, startOff, endRun, endOff)) {
      _refreshFromEngine();
      _syncRibbonFromCaret();
      _statusText = 'Formatting cleared';
    }
    notifyListeners();
  }

  static const _indentStep = 36.0;

  void increaseIndent() {
    if (usesGlyphRendering && _engine != null) {
      final next = _indentLeft + _indentStep;
      _applyParaFormatJson('{"indent_left":$next}');
      _indentLeft = next;
    }
    notifyListeners();
  }

  void decreaseIndent() {
    if (usesGlyphRendering && _engine != null) {
      final next = (_indentLeft - _indentStep).clamp(0.0, double.infinity);
      _applyParaFormatJson('{"indent_left":$next}');
      _indentLeft = next;
    }
    notifyListeners();
  }

  void insertPageBreak() {
    final caretRun = _caretRunId ?? _defaultRunId();
    if (_engine != null && _engine!.insertPageBreakAt(caretRunId: caretRun)) {
      _refreshFromEngine();
      _statusText = 'Page break inserted';
    } else {
      _statusText = 'Page break inserted (mock)';
    }
    notifyListeners();
  }

  void _syncRibbonFromCaret() {
    if (!usesGlyphRendering || _engine == null) return;
    final runId = hasGlyphSelection ? _selFocusRunId : _caretRunId;
    final resolvedRun = runId ?? _defaultRunId();
    if (resolvedRun == null) return;
    final json = _engine!.fetchCaretFormat(resolvedRun);
    if (json == null || json.isEmpty) return;

    final map = jsonDecode(json) as Map<String, dynamic>;
    final charFmt = map['char_format'] as Map<String, dynamic>? ?? {};
    final paraFmt = map['para_format'] as Map<String, dynamic>? ?? {};

    _bold = charFmt['bold'] == true;
    _italic = charFmt['italic'] == true;
    final underline = charFmt['underline'];
    _underline = underline != null && underline != 'None';
    _strikethrough = charFmt['strikethrough'] == true;
    _subscript = charFmt['subscript'] == true;
    _superscript = charFmt['superscript'] == true;
    _fontFamily = charFmt['font_family'] as String? ?? 'Calibri';
    _fontSize = (charFmt['font_size'] as num?)?.toDouble() ?? 11;
    _alignment = _alignmentFromJson(paraFmt['alignment'] as String?);
    _indentLeft = (paraFmt['indent_left'] as num?)?.toDouble() ?? 0;

    final styleName = map['style_name'] as String?;
    if (styleName != null && styleName.isNotEmpty) {
      _activeParagraphStyle = styleName;
    }
  }

  TextAlign _alignmentFromJson(String? value) {
    return switch (value) {
      'Center' => TextAlign.center,
      'Right' => TextAlign.right,
      'Justify' => TextAlign.justify,
      _ => TextAlign.left,
    };
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
    final caretRun = _caretRunId ?? _defaultRunId();
    if (_engine != null && _engine!.applyHeading1Style(caretRunId: caretRun)) {
      _activeParagraphStyle = 'Heading 1';
      _refreshFromEngine();
      _updateRenderModeAfterEngineOpen();
      _statusText = 'Heading 1 applied';
    } else {
      _statusText = 'Heading 1 applied (mock)';
    }
    notifyListeners();
  }

  void applyNormalStyle() {
    final caretRun = _caretRunId ?? _defaultRunId();
    if (_engine != null && _engine!.applyNormalStyleAt(caretRunId: caretRun)) {
      _activeParagraphStyle = 'Normal';
      _refreshFromEngine();
      _updateRenderModeAfterEngineOpen();
      _statusText = 'Normal style applied';
    } else {
      _statusText = 'Normal style applied (mock)';
    }
    notifyListeners();
  }

  void applyNumberedList() {
    final caretRun = _caretRunId ?? _defaultRunId();
    if (_engine != null && _engine!.applyNumberedListStyle(caretRunId: caretRun)) {
      _refreshFromEngine();
      _updateRenderModeAfterEngineOpen();
      _statusText = 'Numbered list applied';
    } else {
      _statusText = 'Numbered list applied (mock)';
    }
    notifyListeners();
  }

  void applyBulletList() {
    final caretRun = _caretRunId ?? _defaultRunId();
    if (_engine != null && _engine!.applyBulletListStyle(caretRunId: caretRun)) {
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
      _syncRibbonFromCaret();
      _statusText = 'Undo';
    }
    notifyListeners();
  }

  void redo() {
    if (_engine != null && _engine!.redoEdit()) {
      _refreshFromEngine();
      _syncRibbonFromCaret();
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
    _pageDisplayLists.clear();
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
    _syncCaretGeometry();
  }

  void _syncCaretGeometry() {
    if (_engine == null || _caretRunId == null || _preferTextRendering) return;
    final geom = _engine!.caretAtPosition(
      _selPage,
      _caretRunId!,
      _caretOffset,
    );
    if (geom != null) {
      _caretGeometry = geom;
      return;
    }
    // Run may be on another page after relayout — search all pages.
    for (var page = 0; page < _pageCount; page++) {
      final cross = _engine!.caretAtPosition(page, _caretRunId!, _caretOffset);
      if (cross != null) {
        _selPage = page;
        _caretGeometry = cross;
        return;
      }
    }
    // Re-resolve via hit test at the last known caret location.
    if (_caretGeometry != null) {
      hitTestAt(_selPage, _caretGeometry!.x, _caretGeometry!.y);
    } else {
      _ensureGlyphCaret();
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
    if (_engine == null) {
      _preferTextRendering = true;
      _recomputePageCount();
      return;
    }
    // Keep the engine as the editing path even for empty documents.
    _preferTextRendering = false;
  }

  bool _engineHasPaintableDisplayList() {
    if (_displayListBytes.isEmpty) return false;
    final snapshot = DisplayListSnapshot.fromBytes(_displayListBytes);
    return snapshot.hasPaintableGlyphs || snapshot.hasPaintableContent;
  }

  /// Whether the UI is in glyph/engine editing mode (vs TextField fallback).
  bool get usesGlyphRendering => !_preferTextRendering;

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
