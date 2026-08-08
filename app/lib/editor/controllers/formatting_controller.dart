import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/controllers/selection_controller.dart';

/// Paragraph line-spacing presets exposed by the Home spacing dialog (F04.S2).
enum LineSpacingMode {
  single,
  oneAndHalf,
  double_,
  exact,
}

/// Ribbon format state — reads ONLY from engine caret-format JSON on caret move.
class FormattingController extends ChangeNotifier {
  FormattingController({
    required EngineHost host,
    required SelectionController selection,
  })  : _host = host,
        _selection = selection;

  final EngineHost _host;
  final SelectionController _selection;

  bool _bold = false;
  bool _italic = false;
  bool _underline = false;
  String _fontFamily = 'Calibri';
  double _fontSize = 11;
  TextAlign _alignment = TextAlign.left;
  bool _strikethrough = false;
  bool _subscript = false;
  bool _superscript = false;
  bool _allCaps = false;
  bool _smallCaps = false;
  bool _hidden = false;
  bool _ligatures = true;
  Color _fontColor = Colors.black;
  Color? _highlightColor;
  String _activeParagraphStyle = 'Normal';
  double _indentLeft = 0;
  bool _inList = false;
  int _listLevel = 0;
  LineSpacingMode _lineSpacing = LineSpacingMode.single;
  double _exactLineSpacingPt = 12;
  double _spaceBefore = 0;
  double _spaceAfter = 0;
  List<Map<String, dynamic>> _tabStops = const [];
  bool _keepTogether = false;
  bool _keepWithNext = false;
  bool _widowOrphanControl = true;
  Color? _paraShading;
  double _borderWidth = 0;
  String _styleInspectorSummary = '';

  bool get bold => _bold;
  bool get italic => _italic;
  bool get underline => _underline;
  String get fontFamily => _fontFamily;
  double get fontSize => _fontSize;
  TextAlign get alignment => _alignment;
  bool get strikethrough => _strikethrough;
  bool get subscript => _subscript;
  bool get superscript => _superscript;
  bool get allCaps => _allCaps;
  bool get smallCaps => _smallCaps;
  bool get hidden => _hidden;
  bool get ligatures => _ligatures;
  Color get fontColor => _fontColor;
  Color? get highlightColor => _highlightColor;
  String get activeParagraphStyle => _activeParagraphStyle;
  double get indentLeft => _indentLeft;
  bool get isInList => _inList;
  int get listLevel => _listLevel;
  LineSpacingMode get lineSpacing => _lineSpacing;
  double get exactLineSpacingPt => _exactLineSpacingPt;
  double get spaceBefore => _spaceBefore;
  double get spaceAfter => _spaceAfter;
  List<Map<String, dynamic>> get tabStops =>
      List<Map<String, dynamic>>.from(_tabStops.map(Map<String, dynamic>.from));
  bool get keepTogether => _keepTogether;
  bool get keepWithNext => _keepWithNext;
  bool get widowOrphanControl => _widowOrphanControl;
  Color? get paraShading => _paraShading;
  double get borderWidth => _borderWidth;
  String get styleInspectorSummary => _styleInspectorSummary;

  void syncFromCaret() {
    final engine = _host.engine;
    if (engine == null) return;
    final runId = _selection.hasGlyphSelection
        ? _selection.selection?.focus.runId
        : _selection.caretRunId;
    final resolvedRun = runId ?? _selection.defaultRunId();
    if (resolvedRun == null) return;
    final json = engine.fetchCaretFormat(resolvedRun);
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
    _allCaps = charFmt['all_caps'] == true;
    _smallCaps = charFmt['small_caps'] == true;
    _hidden = charFmt['hidden'] == true;
    _ligatures = charFmt['ligatures'] != false;
    _fontFamily = charFmt['font_family'] as String? ?? 'Calibri';
    final fontSize = charFmt['font_size'];
    if (fontSize is num) {
      _fontSize = fontSize.toDouble();
    } else if (fontSize is String) {
      final parsed = double.tryParse(fontSize);
      if (parsed != null) _fontSize = parsed.clamp(6, 96);
    }
    _fontColor = _colorFromFormatJson(charFmt['color']) ?? Colors.black;
    _highlightColor = _colorFromFormatJson(charFmt['highlight']);
    _alignment = _alignmentFromJson(paraFmt['alignment'] as String?);
    _indentLeft = (paraFmt['indent_left'] as num?)?.toDouble() ?? 0;
    final numbering = paraFmt['numbering'];
    if (numbering is Map) {
      _inList = true;
      _listLevel = (numbering['level'] as num?)?.toInt() ?? 0;
    } else {
      _inList = false;
      _listLevel = 0;
    }
    _spaceBefore = (paraFmt['space_before'] as num?)?.toDouble() ?? 0;
    _spaceAfter = (paraFmt['space_after'] as num?)?.toDouble() ?? 0;
    final parsedSpacing = _lineSpacingFromJson(paraFmt['line_spacing']);
    _lineSpacing = parsedSpacing.$1;
    if (parsedSpacing.$2 != null) {
      _exactLineSpacingPt = parsedSpacing.$2!;
    }
    _tabStops = _tabStopsFromJson(paraFmt['tab_stops']);
    _keepTogether = paraFmt['keep_together'] == true;
    _keepWithNext = paraFmt['keep_with_next'] == true;
    _widowOrphanControl = paraFmt['widow_orphan_control'] != false;
    _paraShading = _colorFromFormatJson(paraFmt['shading']);
    if (_paraShading?.alpha == 0) _paraShading = null;
    _borderWidth = _borderWidthFromJson(paraFmt['borders']);
    final styleName = map['style_name'] as String?;
    if (styleName != null && styleName.isNotEmpty) {
      _activeParagraphStyle = styleName;
    }
    _styleInspectorSummary = map['inspector_summary'] as String? ?? '';
    notifyListeners();
  }

  Color? _colorFromFormatJson(dynamic value) {
    if (value is! Map) return null;
    final r = value['r'];
    final g = value['g'];
    final b = value['b'];
    if (r is! num || g is! num || b is! num) return null;
    final a = value['a'];
    return Color.fromARGB(
      a is num ? a.round().clamp(0, 255) : 255,
      r.round().clamp(0, 255),
      g.round().clamp(0, 255),
      b.round().clamp(0, 255),
    );
  }

  TextAlign _alignmentFromJson(String? value) => switch (value) {
        'Center' => TextAlign.center,
        'Right' => TextAlign.right,
        'Justify' => TextAlign.justify,
        _ => TextAlign.left,
      };

  /// Parses serde externally-tagged [`LineSpacing`] JSON into UI mode + exact pt.
  (LineSpacingMode, double?) _lineSpacingFromJson(dynamic value) {
    if (value == null || value == 'Single') {
      return (LineSpacingMode.single, null);
    }
    if (value == 'Double') {
      return (LineSpacingMode.double_, null);
    }
    if (value is Map) {
      if (value.containsKey('Exactly')) {
        final pt = (value['Exactly'] as num?)?.toDouble();
        return (LineSpacingMode.exact, pt ?? _exactLineSpacingPt);
      }
      if (value.containsKey('AtLeast')) {
        final pt = (value['AtLeast'] as num?)?.toDouble();
        return (LineSpacingMode.exact, pt ?? _exactLineSpacingPt);
      }
      if (value.containsKey('Multiple')) {
        final m = (value['Multiple'] as num?)?.toDouble() ?? 1.0;
        if ((m - 1.5).abs() < 0.05) return (LineSpacingMode.oneAndHalf, null);
        if ((m - 2.0).abs() < 0.05) return (LineSpacingMode.double_, null);
        if ((m - 1.0).abs() < 0.05) return (LineSpacingMode.single, null);
        return (LineSpacingMode.oneAndHalf, null);
      }
    }
    return (LineSpacingMode.single, null);
  }

  Object _lineSpacingToJson(LineSpacingMode mode, double exactPt) => switch (mode) {
        LineSpacingMode.single => 'Single',
        LineSpacingMode.oneAndHalf => {'Multiple': 1.5},
        LineSpacingMode.double_ => 'Double',
        LineSpacingMode.exact => {'Exactly': exactPt},
      };

  List<Map<String, dynamic>> _tabStopsFromJson(dynamic value) {
    if (value is! List) return const [];
    final stops = <Map<String, dynamic>>[];
    for (final item in value) {
      if (item is! Map) continue;
      final pos = item['position'];
      if (pos is! num) continue;
      final align = item['alignment'];
      stops.add({
        'position': pos.toDouble(),
        'alignment': align is String && align.isNotEmpty ? align : 'Left',
      });
    }
    stops.sort((a, b) =>
        ((a['position'] as num).toDouble()).compareTo((b['position'] as num).toDouble()));
    return stops;
  }

  double _borderWidthFromJson(dynamic value) {
    if (value is! Map) return 0;
    for (final side in ['top', 'left', 'bottom', 'right']) {
      final edge = value[side];
      if (edge is Map) {
        final width = edge['width'];
        if (width is num && width.toDouble() > 0) return width.toDouble();
      }
    }
    return 0;
  }

  Map<String, dynamic>? _colorPatch(Color? color) {
    if (color == null) return null;
    return {
      'r': color.red,
      'g': color.green,
      'b': color.blue,
      'a': color.alpha,
    };
  }

  Map<String, dynamic> _borderSpecJson(double width, Color color) => {
        'width': width,
        'color': _colorPatch(color),
      };

  Future<void> _applyCharFormatJson(String json) async {
    final range = _selection.formatRangeTuple();
    if (range == null || _host.engine == null) return;
    final (startRun, startOff, endRun, endOff) = range;
    final edit = _host.performNativeEdit(
      () => _host.engine!.applyCharFormatJsonAsync(
        startRunId: startRun,
        startOffset: startOff,
        endRunId: endRun,
        endOffset: endOff,
        formatJson: json,
      ),
      dirtyPage: _selection.caretPage,
    );
    await edit;
    syncFromCaret();
  }

  Future<void> _applyParaFormatJson(String json) async {
    final range = _selection.formatRangeTuple();
    if (range == null || _host.engine == null) return;
    final (startRun, startOff, endRun, endOff) = range;
    final edit = _host.performNativeEdit(
      () => _host.engine!.applyParaFormatJsonAsync(
        startRunId: startRun,
        startOffset: startOff,
        endRunId: endRun,
        endOffset: endOff,
        formatJson: json,
      ),
      dirtyPage: _selection.caretPage,
    );
    await edit;
    syncFromCaret();
  }

  void toggleBold() {
    _bold = !_bold;
    unawaited(_applyCharFormatJson('{"bold":$_bold}'));
    notifyListeners();
  }

  void toggleItalic() {
    _italic = !_italic;
    unawaited(_applyCharFormatJson('{"italic":$_italic}'));
    notifyListeners();
  }

  void toggleUnderline() {
    _underline = !_underline;
    unawaited(_applyCharFormatJson(
      _underline ? '{"underline":"Single"}' : '{"underline":"None"}',
    ));
    notifyListeners();
  }

  void setFontFamily(String family) {
    _fontFamily = family;
    unawaited(_applyCharFormatJson(jsonEncode({'font_family': family})));
    notifyListeners();
  }

  void setFontSize(double size) {
    final clamped = size.clamp(6, 96).toDouble();
    _fontSize = clamped;
    unawaited(_applyCharFormatJson('{"font_size":$clamped}'));
    notifyListeners();
  }

  void increaseFontSize() => setFontSize(_fontSize + 1);
  void decreaseFontSize() => setFontSize(_fontSize - 1);

  String _encodeColorPatch({
    Color? color,
    Color? highlight,
    Map<String, dynamic>? themeColor,
  }) {
    final map = <String, dynamic>{};
    if (color != null) {
      map['color'] = {'r': color.red, 'g': color.green, 'b': color.blue, 'a': color.alpha};
      if (themeColor == null) {
        map['theme_color'] = null;
      }
    }
    if (themeColor != null) {
      map['theme_color'] = themeColor;
    }
    if (highlight != null) {
      map['highlight'] = {
        'r': highlight.red,
        'g': highlight.green,
        'b': highlight.blue,
        'a': highlight.alpha,
      };
    }
    return jsonEncode(map);
  }

  void setFontColor(Color color, {String? themeSlot, int? themeVariant}) {
    _fontColor = color;
    final themeColor = themeSlot != null && themeVariant != null
        ? {'slot': themeSlot, 'variant': themeVariant}
        : null;
    unawaited(_applyCharFormatJson(_encodeColorPatch(color: color, themeColor: themeColor)));
    notifyListeners();
  }

  void clearFontColor() {
    _fontColor = Colors.black;
    unawaited(_applyCharFormatJson('{"clear_color":true}'));
    notifyListeners();
  }

  void setHighlight(Color color) {
    _highlightColor = color;
    unawaited(_applyCharFormatJson(_encodeColorPatch(highlight: color)));
    notifyListeners();
  }

  void clearHighlight() {
    _highlightColor = null;
    unawaited(_applyCharFormatJson('{"clear_highlight":true}'));
    notifyListeners();
  }

  void setAlignment(TextAlign align) {
    _alignment = align;
    final name = switch (align) {
      TextAlign.left || TextAlign.start => 'Left',
      TextAlign.center => 'Center',
      TextAlign.right || TextAlign.end => 'Right',
      TextAlign.justify => 'Justify',
    };
    unawaited(_applyParaFormatJson('{"alignment":"$name"}'));
    notifyListeners();
  }

  void toggleStrikethrough() {
    _strikethrough = !_strikethrough;
    unawaited(_applyCharFormatJson('{"strikethrough":$_strikethrough}'));
    notifyListeners();
  }

  void toggleSubscript() {
    _subscript = !_subscript;
    if (_subscript) _superscript = false;
    unawaited(_applyCharFormatJson('{"subscript":$_subscript,"superscript":false}'));
    notifyListeners();
  }

  void toggleSuperscript() {
    _superscript = !_superscript;
    if (_superscript) _subscript = false;
    unawaited(_applyCharFormatJson('{"superscript":$_superscript,"subscript":false}'));
    notifyListeners();
  }

  void toggleAllCaps() {
    _allCaps = !_allCaps;
    if (_allCaps) _smallCaps = false;
    unawaited(_applyCharFormatJson(jsonEncode({'all_caps': _allCaps, 'small_caps': false})));
    notifyListeners();
  }

  void toggleSmallCaps() {
    _smallCaps = !_smallCaps;
    if (_smallCaps) _allCaps = false;
    unawaited(_applyCharFormatJson(jsonEncode({'small_caps': _smallCaps, 'all_caps': false})));
    notifyListeners();
  }

  void toggleHidden() {
    _hidden = !_hidden;
    unawaited(_applyCharFormatJson('{"hidden":$_hidden}'));
    notifyListeners();
  }

  void toggleLigatures() {
    _ligatures = !_ligatures;
    unawaited(_applyCharFormatJson('{"ligatures":$_ligatures}'));
    notifyListeners();
  }

  static const _indentStep = 36.0;

  void increaseIndent() {
    if (_inList) {
      promoteListLevel();
      return;
    }
    final next = _indentLeft + _indentStep;
    unawaited(_applyParaFormatJson('{"indent_left":$next}'));
    _indentLeft = next;
    notifyListeners();
  }

  void decreaseIndent() {
    if (_inList) {
      demoteListLevel();
      return;
    }
    final next = (_indentLeft - _indentStep).clamp(0.0, double.infinity);
    unawaited(_applyParaFormatJson('{"indent_left":$next}'));
    _indentLeft = next;
    notifyListeners();
  }

  void promoteListLevel() {
    if (!_inList) return;
    unawaited(_adjustListLevel(1));
  }

  void demoteListLevel() {
    if (!_inList) return;
    unawaited(_adjustListLevel(-1));
  }

  Future<void> _adjustListLevel(int delta) async {
    if (_host.engine == null) return;
    final edit = _host.performNativeEdit(
      () => _host.engine!.adjustListLevelAsync(
        caretRunId: _selection.defaultRunId(),
        delta: delta,
      ),
      dirtyPage: _selection.caretPage,
    );
    if (await edit) {
      syncFromCaret();
    }
  }

  /// Applies line spacing + space before/after + pagination flags (F04.S2/S4).
  void applySpacing({
    required LineSpacingMode lineSpacing,
    required double exactPoints,
    required double spaceBefore,
    required double spaceAfter,
    required bool keepTogether,
    required bool keepWithNext,
    required bool widowOrphanControl,
  }) {
    _lineSpacing = lineSpacing;
    _exactLineSpacingPt = exactPoints.clamp(1, 240).toDouble();
    _spaceBefore = spaceBefore.clamp(0, 240).toDouble();
    _spaceAfter = spaceAfter.clamp(0, 240).toDouble();
    _keepTogether = keepTogether;
    _keepWithNext = keepWithNext;
    _widowOrphanControl = widowOrphanControl;
    final patch = <String, dynamic>{
      'line_spacing': _lineSpacingToJson(_lineSpacing, _exactLineSpacingPt),
      'space_before': _spaceBefore,
      'space_after': _spaceAfter,
      'keep_together': keepTogether,
      'keep_with_next': keepWithNext,
      'widow_orphan_control': widowOrphanControl,
    };
    unawaited(_applyParaFormatJson(jsonEncode(patch)));
    notifyListeners();
  }

  /// Replaces explicit tab stops (`Some([])` clears; F04.S3).
  void applyTabStops(List<Map<String, dynamic>> stops) {
    final normalized = _tabStopsFromJson(stops);
    _tabStops = normalized;
    unawaited(_applyParaFormatJson(jsonEncode({'tab_stops': normalized})));
    notifyListeners();
  }

  /// Applies paragraph shading and uniform borders (F04.S4).
  void applyBordersAndShading({
    Color? shading,
    double borderWidth = 0,
    Color borderColor = Colors.black,
    bool clearShading = false,
    bool clearBorders = false,
  }) {
    final patch = <String, dynamic>{};
    if (clearShading) {
      patch['shading'] = {'r': 0, 'g': 0, 'b': 0, 'a': 0};
      _paraShading = null;
    } else if (shading != null) {
      patch['shading'] = _colorPatch(shading);
      _paraShading = shading;
    }

    if (clearBorders) {
      patch['borders'] = {
        'top': null,
        'left': null,
        'bottom': null,
        'right': null,
      };
      _borderWidth = 0;
    } else if (borderWidth > 0) {
      final spec = _borderSpecJson(borderWidth, borderColor);
      patch['borders'] = {
        'top': spec,
        'left': spec,
        'bottom': spec,
        'right': spec,
      };
      _borderWidth = borderWidth;
    }

    if (patch.isEmpty) return;
    unawaited(_applyParaFormatJson(jsonEncode(patch)));
    notifyListeners();
  }

  Future<void> clearFormatting() async {
    final range = _selection.formatRangeTuple();
    if (range == null || _host.engine == null) return;
    final (startRun, startOff, endRun, endOff) = range;
    final edit = _host.performNativeEdit(
      () => _host.engine!.clearFormatAsync(startRun, startOff, endRun, endOff),
      dirtyPage: _selection.caretPage,
    );
    if (await edit) syncFromCaret();
  }

  void setActiveParagraphStyle(String style) {
    _activeParagraphStyle = style;
    notifyListeners();
  }
}
