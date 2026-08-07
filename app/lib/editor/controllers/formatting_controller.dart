import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/controllers/selection_controller.dart';

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
    final styleName = map['style_name'] as String?;
    if (styleName != null && styleName.isNotEmpty) {
      _activeParagraphStyle = styleName;
    }
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

  String _encodeColorPatch({Color? color, Color? highlight}) {
    final map = <String, dynamic>{};
    if (color != null) {
      map['color'] = {'r': color.red, 'g': color.green, 'b': color.blue, 'a': color.alpha};
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

  void setFontColor(Color color) {
    _fontColor = color;
    unawaited(_applyCharFormatJson(_encodeColorPatch(color: color)));
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
    final next = _indentLeft + _indentStep;
    unawaited(_applyParaFormatJson('{"indent_left":$next}'));
    _indentLeft = next;
    notifyListeners();
  }

  void decreaseIndent() {
    final next = (_indentLeft - _indentStep).clamp(0.0, double.infinity);
    unawaited(_applyParaFormatJson('{"indent_left":$next}'));
    _indentLeft = next;
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
