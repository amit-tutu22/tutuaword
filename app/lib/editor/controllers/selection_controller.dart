import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/doc_range.dart';

typedef SelectionChangedCallback = void Function();

/// Caret, [DocRange] selection, hit-test, drag, and selection rects.
class SelectionController extends ChangeNotifier {
  SelectionController({
    required EngineHost host,
    required this.onSelectionChanged,
    this.lineHeightFactor = 1.4,
    this.avgCharWidthFactor = 0.52,
  }) : _host = host;

  final EngineHost _host;
  final SelectionChangedCallback onSelectionChanged;
  final double lineHeightFactor;
  final double avgCharWidthFactor;

  String? _caretRunId;
  int _caretOffset = 0;
  CaretGeometry? _caretGeometry;
  DocRange? _selection;
  List<GlyphSelectionRect> _selectionRects = const [];
  bool _glyphDragActive = false;

  String? get caretRunId => _caretRunId;
  int get caretOffset => _caretOffset;
  CaretGeometry? get caretGeometry => _caretGeometry;
  DocRange? get selection => _selection;
  int get caretPage => _selection?.page ?? 0;
  List<GlyphSelectionRect> get selectionRects => _selectionRects;
  bool get isGlyphDragActive => _glyphDragActive;

  bool get hasGlyphSelection =>
      _selection != null && _selection!.isValid && !_selection!.isCollapsed;

  double get _fontSize => 11.0;

  DocumentEngine? get _engine => _host.engine;

  double get _marginLeft => _host.marginLeft;
  double get _marginTop => _host.marginTop;
  double get _marginRight => _host.marginRight;
  double get _marginBottom => _host.marginBottom;

  void reset() {
    _caretRunId = null;
    _caretOffset = 0;
    _caretGeometry = null;
    _selection = null;
    _selectionRects = const [];
    _glyphDragActive = false;
  }

  void setCaret(String runId, int offset, {CaretGeometry? geometry, int? page}) {
    _caretRunId = runId;
    _caretOffset = offset;
    if (geometry != null) _caretGeometry = geometry;
    _selection = DocRange(
      anchor: DocPosition(runId: runId, offset: offset),
      focus: DocPosition(runId: runId, offset: offset),
      page: page ?? caretPage,
    );
    _selectionRects = const [];
  }

  void collapseToCaret() {
    final runId = _caretRunId;
    if (runId == null) return;
    _selection = DocRange(
      anchor: DocPosition(runId: runId, offset: _caretOffset),
      focus: DocPosition(runId: runId, offset: _caretOffset),
      page: caretPage,
    );
    _selectionRects = const [];
  }

  void selectDocRange(DocRange range) {
    if (_engine == null) return;
    _caretRunId = range.focus.runId;
    _caretOffset = range.focus.offset;
    _selection = range;
    _caretGeometry = _engine!.caretAtPosition(range.page, range.focus.runId, range.focus.offset) ??
        _caretGeometry;
    _refreshSelectionRects();
    onSelectionChanged();
    notifyListeners();
  }

  String? defaultRunId() {
    if (_caretRunId != null) return _caretRunId;
    ensureGlyphCaret();
    return _caretRunId;
  }

  void ensureGlyphCaret() {
    if (_caretRunId != null) return;
    _ensureGlyphCaret();
    if (_caretRunId != null) notifyListeners();
  }

  void _ensureGlyphCaret() {
    if (_engine == null || _caretRunId != null) return;
    _engine!.setCurrentPageIndex(0);
    final result = _engine!.hitTestPage(0, _marginLeft, _marginTop + _fontSize);
    if (result == null) return;
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _selection = DocRange(
      anchor: DocPosition(runId: result.runId, offset: result.charOffset),
      focus: DocPosition(runId: result.runId, offset: result.charOffset),
      page: 0,
    );
    _caretGeometry = _engine!.caretGeometryAt(0, _marginLeft, _marginTop + _fontSize);
  }

  (String, int, String, int)? formatRangeTuple() {
    final runId = defaultRunId();
    if (runId == null) return null;
    if (hasGlyphSelection && _selection != null) {
      final (start, end) = _selection!.normalized();
      return (start.runId, start.offset, end.runId, end.offset);
    }
    return (runId, _caretOffset, runId, _caretOffset);
  }

  String selectedText() {
    if (_engine == null || !hasGlyphSelection || _selection == null) return '';
    final (start, end) = _selection!.normalized();
    return _engine!.fetchTextRange(
          start.runId,
          start.offset,
          end.runId,
          end.offset,
        ) ??
        '';
  }

  void hitTestAt(int pageIndex, double x, double y) {
    if (_engine == null) return;
    _activateGlyphPage(pageIndex);
    final result = _engine!.hitTestPage(pageIndex, x, y);
    if (result == null) {
      if (_engine!.isPageStale(pageIndex)) return;
      _placeCaretOnEmptyPage(pageIndex, x, y);
      return;
    }
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _caretGeometry = _engine!.caretGeometryAt(pageIndex, x, y);
    _selection = DocRange(
      anchor: DocPosition(runId: result.runId, offset: result.charOffset),
      focus: DocPosition(runId: result.runId, offset: result.charOffset),
      page: pageIndex,
    );
    _selectionRects = const [];
    onSelectionChanged();
    notifyListeners();
  }

  void _activateGlyphPage(int pageIndex) {
    _selection = _selection?.copyWith(page: pageIndex) ??
        DocRange(
          anchor: DocPosition(runId: _caretRunId ?? '', offset: _caretOffset),
          focus: DocPosition(runId: _caretRunId ?? '', offset: _caretOffset),
          page: pageIndex,
        );
    _engine?.setCurrentPageIndex(pageIndex);
  }

  void _placeCaretOnEmptyPage(int pageIndex, double x, double y) {
    HitTestResult? tail;
    for (var p = pageIndex; p >= 0; p--) {
      // A page awaiting reflow reports no hit; treating it as empty would
      // anchor the caret to whatever page happens to be laid out already.
      if (_engine!.isPageStale(p)) continue;
      tail ??= _engine!.hitTestPage(
        p,
        _host.pageWidth - _marginRight,
        _host.pageHeight - _marginBottom,
      );
    }
    tail ??= _engine!.hitTestPage(0, _marginLeft, _marginTop + _fontSize);

    if (tail != null) {
      _caretRunId = tail.runId;
      _caretOffset = tail.charOffset;
      _selection = DocRange(
        anchor: DocPosition(runId: tail.runId, offset: tail.charOffset),
        focus: DocPosition(runId: tail.runId, offset: tail.charOffset),
        page: pageIndex,
      );
    }

    final geom = tail != null
        ? _engine!.caretAtPosition(pageIndex, tail.runId, tail.charOffset)
        : null;
    _caretGeometry = geom ??
        CaretGeometry(
          x: x.clamp(_marginLeft, _host.pageWidth - _marginRight),
          y: (y - _fontSize).clamp(_marginTop, _host.pageHeight - _marginBottom),
          height: _fontSize * lineHeightFactor,
        );
    _selectionRects = const [];
    onSelectionChanged();
    notifyListeners();
  }

  void _refreshSelectionRects() {
    if (_engine == null || _selection == null) {
      _selectionRects = const [];
      return;
    }
    _selectionRects = _engine!.selectionRectsForRange(caretPage, _selection!);
  }

  void moveGlyphCaretByArrow(LogicalKeyboardKey key) {
    if (_engine == null || _caretRunId == null) return;
    final extend = HardwareKeyboard.instance.isShiftPressed;
    if (extend) _ensureSelectionAnchorForExtend();

    switch (key) {
      case LogicalKeyboardKey.arrowLeft:
        _moveGlyphCaretOffset(-1, extendSelection: extend);
      case LogicalKeyboardKey.arrowRight:
        _moveGlyphCaretOffset(1, extendSelection: extend);
      case LogicalKeyboardKey.arrowUp:
        _moveGlyphCaretUpDown(-1, extendSelection: extend);
      case LogicalKeyboardKey.arrowDown:
        _moveGlyphCaretUpDown(1, extendSelection: extend);
      default:
        break;
    }
  }

  void _ensureSelectionAnchorForExtend() {
    if (_caretRunId == null) return;
    if (_selection != null && hasGlyphSelection) return;
    _selection = DocRange(
      anchor: DocPosition(runId: _caretRunId!, offset: _caretOffset),
      focus: DocPosition(runId: _caretRunId!, offset: _caretOffset),
      page: caretPage,
    );
  }

  void _moveGlyphCaretOffset(int delta, {bool extendSelection = false}) {
    final runId = _caretRunId;
    if (runId == null) return;
    final before = _engine!.caretAtPosition(caretPage, runId, _caretOffset);
    final candidate = (_caretOffset + delta).clamp(0, 1 << 30);
    if (candidate != _caretOffset) {
      final after = _engine!.caretAtPosition(caretPage, runId, candidate);
      if (after != null && !_sameCaretGeometry(before, after)) {
        _applyGlyphCaretMove(runId, candidate, after, extendSelection: extendSelection);
        return;
      }
    }
    if (before == null) return;
    // Empty table cells share a baseline and have zero advance; a 2px nudge stays
    // inside the same cell. Probe farther so left/right can cross into neighbors.
    const distances = <double>[2.0, 24.0, 60.0, 110.0, 180.0];
    for (final distance in distances) {
      final probeX = (before.x + delta.sign * distance)
          .clamp(_marginLeft, _host.pageWidth - _marginRight);
      final hit = _engine!.hitTestPage(caretPage, probeX, before.y);
      if (hit == null) continue;
      if (hit.runId == runId && hit.charOffset == _caretOffset) continue;
      if (extendSelection) {
        _moveGlyphCaretToHit(caretPage, probeX, before.y, extendSelection: true);
      } else {
        hitTestAt(caretPage, probeX, before.y);
      }
      return;
    }
  }

  bool _sameCaretGeometry(CaretGeometry? a, CaretGeometry b) {
    if (a == null) return false;
    const eps = 0.01;
    return (b.x - a.x).abs() < eps && (b.y - a.y).abs() < eps;
  }

  void _applyGlyphCaretMove(
    String runId,
    int offset,
    CaretGeometry geometry, {
    required bool extendSelection,
  }) {
    _caretRunId = runId;
    _caretOffset = offset;
    _caretGeometry = geometry;
    if (extendSelection && _selection != null) {
      _selection = _selection!.copyWith(
        focus: DocPosition(runId: runId, offset: offset),
      );
      _refreshSelectionRects();
    } else {
      _selection = DocRange(
        anchor: DocPosition(runId: runId, offset: offset),
        focus: DocPosition(runId: runId, offset: offset),
        page: caretPage,
      );
      _selectionRects = const [];
    }
    onSelectionChanged();
    notifyListeners();
  }

  void _moveGlyphCaretToHit(
    int pageIndex,
    double x,
    double y, {
    required bool extendSelection,
  }) {
    if (_engine == null) return;
    final result = _engine!.hitTestPage(pageIndex, x, y);
    if (result == null) return;
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _caretGeometry = _engine!.caretGeometryAt(pageIndex, x, y);
    if (extendSelection && _selection != null) {
      _selection = _selection!.copyWith(
        page: pageIndex,
        focus: DocPosition(runId: result.runId, offset: result.charOffset),
      );
      _refreshSelectionRects();
    } else {
      _selection = DocRange(
        anchor: DocPosition(runId: result.runId, offset: result.charOffset),
        focus: DocPosition(runId: result.runId, offset: result.charOffset),
        page: pageIndex,
      );
      _selectionRects = const [];
    }
    onSelectionChanged();
    notifyListeners();
  }

  void _moveGlyphCaretUpDown(int direction, {bool extendSelection = false}) {
    if (_caretGeometry == null) return;
    final stepY = _fontSize * lineHeightFactor;
    final newY = _caretGeometry!.y + stepY * direction;
    if (extendSelection) {
      _ensureSelectionAnchorForExtend();
      _moveGlyphCaretToHit(caretPage, _caretGeometry!.x, newY, extendSelection: true);
      return;
    }
    hitTestAt(caretPage, _caretGeometry!.x, newY);
  }

  void beginGlyphSelection(int pageIndex, double x, double y) =>
      hitTestAt(pageIndex, x, y);

  void updateGlyphSelection(int pageIndex, double x, double y) {
    if (_engine == null || _selection == null) return;
    final result = _engine!.hitTestPage(pageIndex, x, y);
    if (result == null) return;
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _caretGeometry = _engine!.caretGeometryAt(pageIndex, x, y);
    _selection = _selection!.copyWith(
      page: pageIndex,
      focus: DocPosition(runId: result.runId, offset: result.charOffset),
    );
    _refreshSelectionRects();
    notifyListeners();
  }

  void endGlyphSelection(int pageIndex, double x, double y) {
    updateGlyphSelection(pageIndex, x, y);
    onSelectionChanged();
  }

  bool isPointInGlyphSelection(int pageIndex, Offset point) {
    if (!hasGlyphSelection || pageIndex != caretPage || _selectionRects.isEmpty) {
      return false;
    }
    for (final rect in _selectionRects) {
      final bounds = Rect.fromLTWH(rect.x, rect.y, rect.width, rect.height);
      if (bounds.inflate(2).contains(point)) return true;
    }
    return false;
  }

  void beginGlyphDrag(int pageIndex) {
    if (!hasGlyphSelection) return;
    _glyphDragActive = true;
    _activateGlyphPage(pageIndex);
  }

  void updateGlyphDragDropCaret(int pageIndex, double x, double y) {
    if (!_glyphDragActive || _engine == null) return;
    _activateGlyphPage(pageIndex);
    final result = _engine!.hitTestPage(pageIndex, x, y);
    if (result == null) {
      if (_engine!.isPageStale(pageIndex)) return;
      _placeCaretOnEmptyPage(pageIndex, x, y);
      return;
    }
    _caretRunId = result.runId;
    _caretOffset = result.charOffset;
    _caretGeometry = _engine!.caretGeometryAt(pageIndex, x, y);
    notifyListeners();
  }

  void completeGlyphDrag(int pageIndex, double x, double y) {
    if (!_glyphDragActive) return;
    _glyphDragActive = false;
  }

  void cancelGlyphDrag() => _glyphDragActive = false;

  Future<void> selectAll() async {
    if (_engine == null) return;
    ensureGlyphCaret();
    await _host.ensureLayoutReady();
    _host.refreshFromEngine(full: true);
    final start = _engine!.hitTestPage(0, _marginLeft, _marginTop + _fontSize);
    final end = _engine!.fetchDocumentTailHit(0);
    if (start == null || end == null) return;

    _engine!.setCurrentPageIndex(0);
    var focusOffset = end.charOffset;
    if (_caretRunId == end.runId && _caretOffset > focusOffset) {
      focusOffset = _caretOffset;
    }
    _caretRunId = end.runId;
    _caretOffset = focusOffset;
    _selection = DocRange(
      anchor: DocPosition(runId: start.runId, offset: start.charOffset),
      focus: DocPosition(runId: end.runId, offset: focusOffset),
      page: 0,
    );
    _caretGeometry = _engine!.caretAtPosition(0, end.runId, focusOffset) ??
        CaretGeometry(
          x: _host.pageWidth - _marginRight,
          y: _host.pageHeight - _marginBottom,
          height: _fontSize * lineHeightFactor,
        );
    _refreshSelectionRects();
    onSelectionChanged();
    notifyListeners();
  }

  void selectGlyphWordAt(int pageIndex, double x, double y) {
    if (_engine == null) return;
    hitTestAt(pageIndex, x, y);
    final runId = _caretRunId;
    if (runId == null) return;
    final bounds = _wordBoundsInRun(runId, _caretOffset);
    if (bounds.$1 >= bounds.$2) return;

    _caretRunId = runId;
    _caretOffset = bounds.$2;
    _selection = DocRange(
      anchor: DocPosition(runId: runId, offset: bounds.$1),
      focus: DocPosition(runId: runId, offset: bounds.$2),
      page: pageIndex,
    );
    _caretGeometry = _engine!.caretAtPosition(pageIndex, runId, bounds.$2) ?? _caretGeometry;
    _refreshSelectionRects();
    onSelectionChanged();
    notifyListeners();
  }

  (int, int) _wordBoundsInRun(String runId, int offset) {
    final wordChar = RegExp(r'[\p{L}\p{N}_]', unicode: true);
    bool isWordCharAt(int index) {
      if (index < 0) return false;
      final ch = _engine!.fetchTextRange(runId, index, runId, index + 1);
      return ch != null && ch.isNotEmpty && wordChar.hasMatch(ch);
    }

    var pos = offset;
    if (!isWordCharAt(pos) && !isWordCharAt(pos - 1)) {
      while (pos < 1 << 16 && !isWordCharAt(pos)) {
        pos++;
        final probe = _engine!.fetchTextRange(runId, pos, runId, pos + 1);
        if (probe == null || probe.isEmpty) break;
      }
      if (!isWordCharAt(pos)) {
        pos = offset;
        while (pos > 0 && !isWordCharAt(pos - 1)) pos--;
        if (pos > 0 && isWordCharAt(pos - 1)) {
          var end = pos;
          while (pos > 0 && isWordCharAt(pos - 1)) pos--;
          return (pos, end);
        }
        return (offset, offset);
      }
    } else if (pos > 0 && !isWordCharAt(pos) && isWordCharAt(pos - 1)) {
      pos--;
    }

    var start = pos;
    while (start > 0 && isWordCharAt(start - 1)) start--;
    var end = pos;
    while (isWordCharAt(end)) end++;
    return (start, end);
  }

  Future<bool> deleteGlyphSelection() async {
    if (_engine == null || _selection == null || !hasGlyphSelection) return false;
    final (start, end) = _selection!.normalized();
    Future<bool> edit;
    if (start.runId == end.runId) {
      final lo = start.offset < end.offset ? start.offset : end.offset;
      final hi = start.offset < end.offset ? end.offset : start.offset;
      if (lo >= hi) return false;
      edit = _host.performNativeEdit(
        () => _engine!.deleteRangeAsync(start.runId, lo, hi),
        dirtyPage: caretPage,
      );
      _caretRunId = start.runId;
      _caretOffset = lo;
    } else {
      edit = _host.performNativeEdit(
        () => _engine!.deleteDocRangeAsync(
          start.runId,
          start.offset,
          end.runId,
          end.offset,
        ),
        dirtyPage: caretPage,
      );
      _caretRunId = start.runId;
      _caretOffset = start.offset;
    }
    if (!await edit) return false;
    collapseToCaret();
    syncCaretGeometry();
    notifyListeners();
    return true;
  }

  void syncCaretGeometry() {
    if (_engine == null || _caretRunId == null) return;
    final geom = _engine!.caretAtPosition(caretPage, _caretRunId!, _caretOffset);
    if (geom != null) {
      _caretGeometry = geom;
      return;
    }
    for (var page = 0; page < _host.pageCount; page++) {
      final cross = _engine!.caretAtPosition(page, _caretRunId!, _caretOffset);
      if (cross != null) {
        _selection = _selection?.copyWith(page: page);
        _caretGeometry = cross;
        return;
      }
    }
    if (_caretGeometry != null) {
      hitTestAt(caretPage, _caretGeometry!.x, _caretGeometry!.y);
    } else {
      _ensureGlyphCaret();
    }
  }

  void afterInsert(String runId, int newOffset, {CaretGeometry? geometry}) {
    _caretRunId = runId;
    _caretOffset = newOffset;
    if (geometry != null) _caretGeometry = geometry;
    _selection = DocRange(
      anchor: DocPosition(runId: runId, offset: newOffset),
      focus: DocPosition(runId: runId, offset: newOffset),
      page: caretPage,
    );
    _selectionRects = const [];
  }
}
