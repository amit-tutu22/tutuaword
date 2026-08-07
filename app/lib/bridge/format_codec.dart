import 'dart:convert';

/// Dynamic caret/selection format state parsed from engine JSON (R2.5).
///
/// Unknown fields are preserved — new Rust `CharFormat` fields require no Dart typedefs.
class CharFormatState {
  CharFormatState(this.raw);

  final Map<String, dynamic> raw;

  bool? get bold => raw['bold'] as bool?;
  bool? get italic => raw['italic'] as bool?;
  bool? get strikethrough => raw['strikethrough'] as bool?;
  bool? get subscript => raw['subscript'] as bool?;
  bool? get superscript => raw['superscript'] as bool?;
  bool? get allCaps => raw['all_caps'] as bool?;
  bool? get smallCaps => raw['small_caps'] as bool?;
  bool? get hidden => raw['hidden'] as bool?;
  bool? get ligatures => raw['ligatures'] as bool?;
  String? get fontFamily => raw['font_family'] as String?;
  double? get fontSize => FormatCodec.numericField(raw, 'font_size');
  double? get characterSpacing => FormatCodec.numericField(raw, 'character_spacing');
  dynamic get color => raw['color'];
  dynamic get highlight => raw['highlight'];
  dynamic get underline => raw['underline'];

  dynamic operator [](String key) => raw[key];
}

class ParaFormatState {
  ParaFormatState(this.raw);

  final Map<String, dynamic> raw;

  String? get alignment => raw['alignment'] as String?;
  double? get indentLeft => FormatCodec.numericField(raw, 'indent_left');
}

class CaretFormatState {
  CaretFormatState({
    required this.charFormat,
    required this.paraFormat,
    this.styleName,
  });

  final CharFormatState charFormat;
  final ParaFormatState paraFormat;
  final String? styleName;
}

/// Open-map helpers for char/para format patches (no per-field FFI typedefs).
class FormatCodec {
  FormatCodec._();

  static Map<String, dynamic> charFormatPatch([Map<String, dynamic>? fields]) {
    return Map<String, dynamic>.from(fields ?? const {});
  }

  static Map<String, dynamic> charFormatPatchFromEntries(Map<String, dynamic> entries) {
    return Map<String, dynamic>.from(entries);
  }

  static Map<String, dynamic> paraFormatPatch([Map<String, dynamic>? fields]) {
    return Map<String, dynamic>.from(fields ?? const {});
  }

  static CaretFormatState fromCaretFormatJson(String json) {
    final map = jsonDecode(json) as Map<String, dynamic>;
    final charFmt = map['char_format'] as Map<String, dynamic>? ?? const {};
    final paraFmt = map['para_format'] as Map<String, dynamic>? ?? const {};
    return CaretFormatState(
      charFormat: CharFormatState(charFmt),
      paraFormat: ParaFormatState(paraFmt),
      styleName: map['style_name'] as String?,
    );
  }

  static double? numericField(Map<String, dynamic> map, String key) {
    final value = map[key];
    if (value is num) return value.toDouble();
    if (value is String) return double.tryParse(value);
    return null;
  }
}
