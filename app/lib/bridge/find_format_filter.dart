import 'dart:convert';

/// Format constraints for find (F18.S4).
class FindFormatFilter {
  const FindFormatFilter({
    this.bold,
    this.italic,
    this.styleName,
  });

  final bool? bold;
  final bool? italic;
  final String? styleName;

  bool get isActive =>
      bold != null || italic != null || (styleName != null && styleName!.isNotEmpty);

  Map<String, dynamic> toJson() => {
        if (bold != null) 'bold': bold,
        if (italic != null) 'italic': italic,
        if (styleName != null && styleName!.isNotEmpty) 'style_name': styleName,
      };

  String encode() => jsonEncode(toJson());

  static const none = FindFormatFilter();
}
