import 'package:tutuaword/editor/latex_omml.dart';

/// Minimal OMML builder for the equation editor (F14.S3).
class EquationOmml {
  EquationOmml._();

  static const ommlNs =
      'http://schemas.openxmlformats.org/officeDocument/2006/math';

  /// Symbol palette entries shown in the insert dialog.
  static const List<EquationPaletteEntry> palette = [
    EquationPaletteEntry(label: 'α', insert: 'α'),
    EquationPaletteEntry(label: 'β', insert: 'β'),
    EquationPaletteEntry(label: 'γ', insert: 'γ'),
    EquationPaletteEntry(label: 'π', insert: 'π'),
    EquationPaletteEntry(label: 'Σ', insert: 'Σ'),
    EquationPaletteEntry(label: '∞', insert: '∞'),
    EquationPaletteEntry(label: '±', insert: '±'),
    EquationPaletteEntry(label: '×', insert: '×'),
    EquationPaletteEntry(label: '÷', insert: '÷'),
    EquationPaletteEntry(label: '√', insert: '√'),
    EquationPaletteEntry(label: '∫', insert: '∫'),
    EquationPaletteEntry(label: '≤', insert: '≤'),
    EquationPaletteEntry(label: '≥', insert: '≥'),
    EquationPaletteEntry(label: '≠', insert: '≠'),
  ];

  static String escapeText(String text) {
    return text
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;');
  }

  static String run(String text) {
    return '<m:r><m:t xml:space="preserve">${escapeText(text)}</m:t></m:r>';
  }

  static String inlineOmath(String content) {
    return '<m:oMath xmlns:m="$ommlNs">$content</m:oMath>';
  }

  static String displayOmathPara(String innerOmath) {
    final inner = innerOmath.contains('<m:oMath')
        ? innerOmath
        : inlineOmath(run(innerOmath));
    return '<m:oMathPara xmlns:m="$ommlNs">$inner</m:oMathPara>';
  }

  /// Build OMML from [model] for inline or display insertion.
  static String build(EquationModel model) {
    final content = _buildContent(model);
    if (model.display) {
      return displayOmathPara(inlineOmath(content));
    }
    return inlineOmath(content);
  }

  static String _buildContent(EquationModel model) {
    switch (model.kind) {
      case EquationKind.plain:
        return run(model.text.trim());
      case EquationKind.fraction:
        return '<m:f>'
            '<m:num>${run(model.numerator.trim())}</m:num>'
            '<m:den>${run(model.denominator.trim())}</m:den>'
            '</m:f>';
      case EquationKind.superscript:
        return '<m:sSup>'
            '<m:e>${run(model.base.trim())}</m:e>'
            '<m:sup>${run(model.superscript.trim())}</m:sup>'
            '</m:sSup>';
      case EquationKind.subscript:
        return '<m:sSub>'
            '<m:e>${run(model.base.trim())}</m:e>'
            '<m:sub>${run(model.subscript.trim())}</m:sub>'
            '</m:sSub>';
      case EquationKind.squareRoot:
        return '<m:rad>'
            '<m:deg/>'
            '<m:e>${run(model.text.trim())}</m:e>'
            '</m:rad>';
      case EquationKind.latex:
        throw StateError('LaTeX kind uses LatexOmml.convert in toOmml()');
    }
  }

  /// Best-effort preview text for the dialog title bar.
  static String previewText(EquationModel model) {
    switch (model.kind) {
      case EquationKind.plain:
        return model.text.trim();
      case EquationKind.fraction:
        return '${model.numerator.trim()}/${model.denominator.trim()}';
      case EquationKind.superscript:
        return '${model.base.trim()}^${model.superscript.trim()}';
      case EquationKind.subscript:
        return '${model.base.trim()}_${model.subscript.trim()}';
      case EquationKind.squareRoot:
        return '√${model.text.trim()}';
      case EquationKind.latex:
        return model.latex.trim();
    }
  }

  /// Parse an existing inline OMML fragment into an editable model.
  static EquationModel? fromOmml(String xml) {
    if (xml.contains('<m:f>')) {
      return EquationModel.fraction(
        numerator: _firstText(xml, '<m:num>') ?? '',
        denominator: _firstText(xml, '<m:den>') ?? '',
        display: xml.contains('<m:oMathPara'),
      );
    }
    if (xml.contains('<m:sSup>')) {
      return EquationModel.superscript(
        base: _firstText(xml, '<m:e>') ?? '',
        superscript: _textAfterTag(xml, '<m:sup>') ?? '',
        display: xml.contains('<m:oMathPara'),
      );
    }
    if (xml.contains('<m:sSub>')) {
      return EquationModel.subscript(
        base: _firstText(xml, '<m:e>') ?? '',
        subscript: _textAfterTag(xml, '<m:sub>') ?? '',
        display: xml.contains('<m:oMathPara'),
      );
    }
    if (xml.contains('<m:rad>')) {
      return EquationModel.squareRoot(
        text: _textAfterTag(xml, '<m:e>') ?? _allText(xml),
        display: xml.contains('<m:oMathPara'),
      );
    }
    final text = _allText(xml);
    if (text.isEmpty) return null;
    return EquationModel.plain(
      text: text,
      display: xml.contains('<m:oMathPara'),
    );
  }

  static String? _firstText(String xml, String openTag) {
    final start = xml.indexOf(openTag);
    if (start < 0) return null;
    final slice = xml.substring(start);
    return _textInRun(slice);
  }

  static String? _textAfterTag(String xml, String openTag) {
    final start = xml.indexOf(openTag);
    if (start < 0) return null;
    return _textInRun(xml.substring(start));
  }

  static String? _textInRun(String slice) {
    final tStart = slice.indexOf('<m:t');
    if (tStart < 0) return null;
    final gt = slice.indexOf('>', tStart);
    if (gt < 0) return null;
    final close = slice.indexOf('</m:t>', gt + 1);
    if (close < 0) return null;
    return _decodeEntities(slice.substring(gt + 1, close));
  }

  static String _allText(String xml) {
    final buffer = StringBuffer();
    var rest = xml;
    while (true) {
      final tStart = rest.indexOf('<m:t');
      if (tStart < 0) break;
      final gt = rest.indexOf('>', tStart);
      if (gt < 0) break;
      final close = rest.indexOf('</m:t>', gt + 1);
      if (close < 0) break;
      buffer.write(_decodeEntities(rest.substring(gt + 1, close)));
      rest = rest.substring(close + 6);
    }
    return buffer.toString();
  }

  static String _decodeEntities(String text) {
    return text
        .replaceAll('&lt;', '<')
        .replaceAll('&gt;', '>')
        .replaceAll('&amp;', '&')
        .replaceAll('&quot;', '"')
        .replaceAll('&apos;', "'");
  }

  static String? latexCommandForSymbol(String symbol) {
    return switch (symbol) {
      'α' => r'\alpha',
      'β' => r'\beta',
      'γ' => r'\gamma',
      'π' => r'\pi',
      'Σ' => r'\Sigma',
      '∞' => r'\infty',
      '±' => r'\pm',
      '×' => r'\times',
      '÷' => r'\div',
      '√' => r'\sqrt{}',
      '∫' => r'\int',
      '≤' => r'\leq',
      '≥' => r'\geq',
      '≠' => r'\neq',
      _ => null,
    };
  }
}

class EquationPaletteEntry {
  const EquationPaletteEntry({required this.label, required this.insert});

  final String label;
  final String insert;
}

enum EquationKind { plain, fraction, superscript, subscript, squareRoot, latex }

class EquationModel {
  const EquationModel._({
    required this.kind,
    this.text = '',
    this.numerator = '',
    this.denominator = '',
    this.base = '',
    this.superscript = '',
    this.subscript = '',
    this.latex = '',
    this.display = false,
  });

  final EquationKind kind;
  final String text;
  final String numerator;
  final String denominator;
  final String base;
  final String superscript;
  final String subscript;
  final String latex;
  final bool display;

  factory EquationModel.plain({String text = 'x', bool display = false}) {
    return EquationModel._(kind: EquationKind.plain, text: text, display: display);
  }

  factory EquationModel.fraction({
    String numerator = 'a',
    String denominator = 'b',
    bool display = true,
  }) {
    return EquationModel._(
      kind: EquationKind.fraction,
      numerator: numerator,
      denominator: denominator,
      display: display,
    );
  }

  factory EquationModel.superscript({
    String base = 'x',
    String superscript = '2',
    bool display = false,
  }) {
    return EquationModel._(
      kind: EquationKind.superscript,
      base: base,
      superscript: superscript,
      display: display,
    );
  }

  factory EquationModel.subscript({
    String base = 'x',
    String subscript = 'i',
    bool display = false,
  }) {
    return EquationModel._(
      kind: EquationKind.subscript,
      base: base,
      subscript: subscript,
      display: display,
    );
  }

  factory EquationModel.squareRoot({
    String text = 'x',
    bool display = false,
  }) {
    return EquationModel._(
      kind: EquationKind.squareRoot,
      text: text,
      display: display,
    );
  }

  factory EquationModel.latex({
    String source = r'\frac{a}{b}',
    bool display = false,
  }) {
    return EquationModel._(
      kind: EquationKind.latex,
      latex: source,
      display: display,
    );
  }

  EquationModel copyWith({
    EquationKind? kind,
    String? text,
    String? numerator,
    String? denominator,
    String? base,
    String? superscript,
    String? subscript,
    String? latex,
    bool? display,
  }) {
    return EquationModel._(
      kind: kind ?? this.kind,
      text: text ?? this.text,
      numerator: numerator ?? this.numerator,
      denominator: denominator ?? this.denominator,
      base: base ?? this.base,
      superscript: superscript ?? this.superscript,
      subscript: subscript ?? this.subscript,
      latex: latex ?? this.latex,
      display: display ?? this.display,
    );
  }

  /// Rebuild with all fields explicitly set (dialog state flush).
  EquationModel withFields({
    required EquationKind kind,
    required String text,
    required String numerator,
    required String denominator,
    required String base,
    required String superscript,
    required String subscript,
    required String latex,
    required bool display,
  }) {
    return EquationModel._(
      kind: kind,
      text: text,
      numerator: numerator,
      denominator: denominator,
      base: base,
      superscript: superscript,
      subscript: subscript,
      latex: latex,
      display: display,
    );
  }

  String toOmml() {
    if (kind == EquationKind.latex) {
      final omml = LatexOmml.convert(latex, display: display);
      if (omml == null) {
        throw StateError(LatexOmml.lastError ?? 'Invalid LaTeX');
      }
      return omml;
    }
    return EquationOmml.build(this);
  }

  bool get isValid {
    switch (kind) {
      case EquationKind.plain:
      case EquationKind.squareRoot:
        return text.trim().isNotEmpty;
      case EquationKind.fraction:
        return numerator.trim().isNotEmpty && denominator.trim().isNotEmpty;
      case EquationKind.superscript:
        return base.trim().isNotEmpty && superscript.trim().isNotEmpty;
      case EquationKind.subscript:
        return base.trim().isNotEmpty && subscript.trim().isNotEmpty;
      case EquationKind.latex:
        return latex.trim().isNotEmpty && LatexOmml.convert(latex, display: display) != null;
    }
  }
}
