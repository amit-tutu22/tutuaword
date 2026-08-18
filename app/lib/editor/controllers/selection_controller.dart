import 'dart:async';
import 'dart:convert';

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

  /// Selection highlight for any page (cross-page drag paints non-caret pages too).
  List<GlyphSelectionRect> selectionRectsForPage(int pageIndex) {
    if (_engine == null || _selection == null || !hasGlyphSelection) {
      return const [];
    }
    if (pageIndex == caretPage) return _selectionRects;
    return _engine!.selectionRectsForRange(pageIndex, _selection!);
  }

  /// [extend] overrides the live Shift state, which the web DOM key listener
  /// bypasses (it never reaches [HardwareKeyboard]).
  void moveGlyphCaretByArrow(LogicalKeyboardKey key, {bool? extend}) {
    if (_engine == null || _caretRunId == null) return;
    final extending = extend ?? HardwareKeyboard.instance.isShiftPressed;
    if (extending) _ensureSelectionAnchorForExtend();

    switch (key) {
      case LogicalKeyboardKey.arrowLeft:
        _moveGlyphCaretOffset(-1, extendSelection: extending);
      case LogicalKeyboardKey.arrowRight:
        _moveGlyphCaretOffset(1, extendSelection: extending);
      case LogicalKeyboardKey.arrowUp:
        _moveGlyphCaretUpDown(-1, extendSelection: extending);
      case LogicalKeyboardKey.arrowDown:
        _moveGlyphCaretUpDown(1, extendSelection: extending);
      default:
        break;
    }
  }

  /// Home / End — the visual line edge, found by probing the caret's own
  /// baseline at the text-column margins.
  void moveGlyphCaretToLineEdge({required bool toEnd, required bool extend}) {
    if (_engine == null || _caretGeometry == null) return;
    if (extend) _ensureSelectionAnchorForExtend();
    final y = _caretGeometry!.y;
    final x = toEnd ? _host.pageWidth - _marginRight : _marginLeft;
    if (extend) {
      _moveGlyphCaretToHit(caretPage, x, y, extendSelection: true);
    } else {
      hitTestAt(caretPage, x, y);
    }
  }

  /// Ctrl+Home / Ctrl+End.
  void moveGlyphCaretToDocumentEdge({
    required bool toEnd,
    required bool extend,
  }) {
    if (_engine == null) return;
    if (extend) _ensureSelectionAnchorForExtend();

    if (!toEnd) {
      _engine!.setCurrentPageIndex(0);
      final x = _marginLeft;
      final y = _marginTop + _fontSize;
      if (extend) {
        _moveGlyphCaretToHit(0, x, y, extendSelection: true);
      } else {
        hitTestAt(0, x, y);
      }
      return;
    }

    final tail = _engine!.fetchDocumentTailHit(0);
    if (tail == null) return;
    final lastPage = _host.pageCount > 0 ? _host.pageCount - 1 : 0;
    final located = _engine!.caretPageAndGeometry(
      tail.runId,
      tail.charOffset,
      hintPage: lastPage,
    );
    final page = located?.$1 ?? caretPage;
    _engine!.setCurrentPageIndex(page);
    _caretRunId = tail.runId;
    _caretOffset = tail.charOffset;
    if (located != null) _caretGeometry = located.$2;
    final position = DocPosition(runId: tail.runId, offset: tail.charOffset);
    if (extend && _selection != null) {
      _selection = _selection!.copyWith(page: page, focus: position);
      _refreshSelectionRects();
    } else {
      _selection = DocRange(anchor: position, focus: position, page: page);
      _selectionRects = const [];
    }
    onSelectionChanged();
    notifyListeners();
  }

  /// Page Up / Page Down. A page is the scroll unit in this viewport, so one
  /// screen and one page are the same step; the column (x) is preserved.
  void moveGlyphCaretByPage({required int direction, required bool extend}) {
    if (_engine == null || direction == 0) return;
    final nextPage = caretPage + direction;
    if (nextPage < 0 || nextPage >= _host.pageCount) {
      // Word clamps to the document edge on the first and last screen.
      moveGlyphCaretToDocumentEdge(toEnd: direction > 0, extend: extend);
      return;
    }
    if (extend) _ensureSelectionAnchorForExtend();
    final x = _caretGeometry?.x ?? _marginLeft;
    final y = (_caretGeometry?.y ?? (_marginTop + _fontSize))
        .clamp(_marginTop + _fontSize, _host.pageHeight - _marginBottom);
    if (extend) {
      _moveGlyphCaretToHit(nextPage, x, y, extendSelection: true);
    } else {
      hitTestAt(nextPage, x, y);
    }
  }

  /// Ctrl+Left / Ctrl+Right — Word lands on the start of the adjacent word.
  void moveGlyphCaretByWord({required int direction, required bool extend}) {
    final runId = _caretRunId;
    if (_engine == null || runId == null || direction == 0) return;
    final target = direction < 0
        ? _wordStartBefore(runId, _caretOffset)
        : _wordStartAfter(runId, _caretOffset);
    final geometry = target == _caretOffset
        ? null
        : _engine!.caretAtPosition(caretPage, runId, target);
    if (geometry == null) {
      // Already at the run edge: a single-character step is what crosses into
      // the neighbouring run, and page, for us.
      if (extend) _ensureSelectionAnchorForExtend();
      _moveGlyphCaretOffset(direction, extendSelection: extend);
      return;
    }
    if (extend) _ensureSelectionAnchorForExtend();
    _applyGlyphCaretMove(runId, target, geometry, extendSelection: extend);
  }

  /// Ctrl+Up / Ctrl+Down. Word steps to the start of the caret's own paragraph
  /// first and only then to the previous one; down always lands on the next
  /// paragraph's start, and both clamp to the document edges.
  void moveGlyphCaretByParagraph({required int direction, required bool extend}) {
    final runId = _caretRunId;
    if (_engine == null || runId == null || direction == 0) return;
    final nav = _paragraphNav(runId, _caretOffset);
    if (nav == null) {
      // No paragraph information from this engine — a line step is the closest
      // honest move, which is what the caret did before paragraph motion existed.
      if (extend) _ensureSelectionAnchorForExtend();
      _moveGlyphCaretUpDown(direction, extendSelection: extend);
      return;
    }
    final start = nav['start'];
    final atParagraphStart =
        start != null && start.$1 == runId && start.$2 == _caretOffset;
    final target = direction < 0
        ? (atParagraphStart ? nav['prev'] : start)
        : nav['next'];
    if (target == null) {
      moveGlyphCaretToDocumentEdge(toEnd: direction > 0, extend: extend);
      return;
    }
    final located = _engine!.caretPageAndGeometry(
      target.$1,
      target.$2,
      hintPage: caretPage,
    );
    if (located == null) return;
    if (extend) _ensureSelectionAnchorForExtend();
    _engine!.setCurrentPageIndex(located.$1);
    _caretRunId = target.$1;
    _caretOffset = target.$2;
    _caretGeometry = located.$2;
    final position = DocPosition(runId: target.$1, offset: target.$2);
    if (extend && _selection != null) {
      _selection = _selection!.copyWith(page: located.$1, focus: position);
      _refreshSelectionRects();
    } else {
      _selection = DocRange(
        anchor: position,
        focus: position,
        page: located.$1,
      );
      _selectionRects = const [];
    }
    onSelectionChanged();
    notifyListeners();
  }

  /// `start` / `prev` / `next` paragraph starts as (runId, offset) pairs, or null
  /// when the engine cannot report paragraph structure.
  Map<String, (String, int)?>? _paragraphNav(String runId, int offset) {
    final json = _engine!.fetchParagraphNav(runId, offset);
    if (json == null || json.isEmpty) return null;
    try {
      final decoded = jsonDecode(json);
      if (decoded is! Map<String, dynamic>) return null;
      (String, int)? position(String key) {
        final value = decoded[key];
        if (value is! Map) return null;
        final run = value['run'];
        final at = value['offset'];
        if (run is! String || at is! int) return null;
        return (run, at);
      }

      return {
        'start': position('start'),
        'prev': position('prev'),
        'next': position('next'),
      };
    } on FormatException {
      return null;
    }
  }

  /// Range Ctrl+Backspace / Ctrl+Delete should remove, or null when the caret
  /// is already at the run edge and a plain character delete should run instead.
  (int, int)? wordDeleteRange({required int direction}) {
    final runId = _caretRunId;
    if (_engine == null || runId == null || direction == 0) return null;
    if (direction < 0) {
      final start = _wordStartBefore(runId, _caretOffset);
      return start < _caretOffset ? (start, _caretOffset) : null;
    }
    final end = _wordStartAfter(runId, _caretOffset);
    return end > _caretOffset ? (_caretOffset, end) : null;
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
    var reachedHorizontalEdge = false;
    for (final distance in distances) {
      final probeX = (before.x + delta.sign * distance)
          .clamp(_marginLeft, _host.pageWidth - _marginRight);
      if (delta < 0 && probeX <= _marginLeft + 0.5) {
        reachedHorizontalEdge = true;
      }
      if (delta > 0 && probeX >= _host.pageWidth - _marginRight - 0.5) {
        reachedHorizontalEdge = true;
      }
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
    // Only leave the page when the caret is already at the horizontal edge.
    if (reachedHorizontalEdge) {
      _moveGlyphCaretAcrossPage(delta.sign, extendSelection: extendSelection);
    }
  }

  void _moveGlyphCaretAcrossPage(int direction, {required bool extendSelection}) {
    if (direction == 0) return;
    final nextPage = caretPage + direction;
    if (nextPage < 0 || nextPage >= _host.pageCount) return;
    final x = direction > 0
        ? _marginLeft
        : (_host.pageWidth - _marginRight);
    final y = direction > 0
        ? (_marginTop + _fontSize)
        : (_host.pageHeight - _marginBottom - _fontSize);
    if (extendSelection) {
      _ensureSelectionAnchorForExtend();
      _moveGlyphCaretToHit(nextPage, x, y, extendSelection: true);
    } else {
      hitTestAt(nextPage, x, y);
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
    final x = _caretGeometry!.x;
    final newY = _caretGeometry!.y + stepY * direction;
    final contentTop = _marginTop;
    final contentBottom = _host.pageHeight - _marginBottom;

    if (newY > contentBottom + 0.5 && direction > 0) {
      _moveGlyphCaretAcrossPage(1, extendSelection: extendSelection);
      return;
    }
    if (newY < contentTop - 0.5 && direction < 0) {
      _moveGlyphCaretAcrossPage(-1, extendSelection: extendSelection);
      return;
    }

    final beforeRun = _caretRunId;
    final beforeOff = _caretOffset;
    final beforeY = _caretGeometry!.y;
    final clampedY = newY.clamp(contentTop, contentBottom);
    final probingPageEdge = (direction > 0 && newY >= contentBottom - stepY) ||
        (direction < 0 && newY <= contentTop + stepY);
    if (extendSelection) {
      _ensureSelectionAnchorForExtend();
      _moveGlyphCaretToHit(caretPage, x, clampedY, extendSelection: true);
    } else {
      hitTestAt(caretPage, x, clampedY);
    }
    // Stuck on the last/first line — only then advance across the page break.
    // (Engines that ignore probe Y must not treat every Down as a page jump.)
    final stuck = _caretRunId == beforeRun &&
        _caretOffset == beforeOff &&
        ((_caretGeometry?.y ?? beforeY) - beforeY).abs() < 0.5;
    if (stuck && probingPageEdge) {
      _moveGlyphCaretAcrossPage(direction, extendSelection: extendSelection);
    }
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
    if (!hasGlyphSelection) return false;
    final rects = selectionRectsForPage(pageIndex);
    if (rects.isEmpty) return false;
    for (final rect in rects) {
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

  static final _wordCharPattern = RegExp(r'[\p{L}\p{N}_]', unicode: true);

  /// Longest run offset the scanners will probe, so a missing run cannot spin.
  static const _runScanLimit = 1 << 16;

  String? _charAt(String runId, int index) {
    if (index < 0) return null;
    final ch = _engine!.fetchTextRange(runId, index, runId, index + 1);
    if (ch == null || ch.isEmpty) return null;
    return ch;
  }

  bool _isWordCharAt(String runId, int index) {
    final ch = _charAt(runId, index);
    return ch != null && _wordCharPattern.hasMatch(ch);
  }

  /// Start of the word at or before [offset]: skip any gap, then the word.
  int _wordStartBefore(String runId, int offset) {
    var pos = offset;
    while (pos > 0 && !_isWordCharAt(runId, pos - 1)) {
      pos--;
    }
    while (pos > 0 && _isWordCharAt(runId, pos - 1)) {
      pos--;
    }
    return pos;
  }

  /// Start of the next word after [offset]: skip the current word, then the gap.
  int _wordStartAfter(String runId, int offset) {
    var pos = offset;
    final limit = offset + _runScanLimit;
    while (pos < limit && _isWordCharAt(runId, pos)) {
      pos++;
    }
    while (pos < limit && _charAt(runId, pos) != null && !_isWordCharAt(runId, pos)) {
      pos++;
    }
    return pos;
  }

  (int, int) _wordBoundsInRun(String runId, int offset) {
    bool isWordCharAt(int index) => _isWordCharAt(runId, index);

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
