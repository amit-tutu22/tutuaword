import 'package:tutuaword/editor/equation_omml.dart';

/// LaTeX → OMML subset converter (F14.S4). Mirrors Rust `tw_model::latex_omml`.
class LatexOmml {
  LatexOmml._();

  static String? lastError;

  /// Returns inline or display OMML, or null with [lastError] set.
  static String? convert(String latex, {bool display = false}) {
    lastError = null;
    final trimmed = latex.trim();
    if (trimmed.isEmpty) {
      lastError = 'Empty LaTeX input';
      return null;
    }
    try {
      final parser = _Parser(trimmed);
      final content = parser.parseSequence();
      parser.skipWs();
      if (!parser.atEnd) {
        lastError = 'Unexpected trailing input';
        return null;
      }
      final inner = display
          ? EquationOmml.displayOmathPara(EquationOmml.inlineOmath(content))
          : EquationOmml.inlineOmath(content);
      return inner;
    } on _LatexParseException catch (e) {
      lastError = e.message;
      return null;
    }
  }
}

class _LatexParseException implements Exception {
  _LatexParseException(this.message);
  final String message;
}

class _Parser {
  _Parser(this.input);

  final String input;
  int _pos = 0;

  bool get atEnd => _pos >= input.length;

  String? _peek() => atEnd ? null : input[_pos];

  String _bump() {
    final ch = input[_pos];
    _pos += 1;
    return ch;
  }

  void skipWs() {
    while (_peek() != null && _peek()!.trim().isEmpty) {
      _bump();
    }
  }

  bool get _atGroupEnd => atEnd || _peek() == '}';

  String parseSequence() {
    final parts = <String>[];
    skipWs();
    while (!atEnd && !_atGroupEnd) {
      final atom = parseAtom();
      parts.add(applyScripts(atom));
      skipWs();
    }
    if (parts.isEmpty) {
      throw _LatexParseException('Expected math content');
    }
    return parts.join();
  }

  String parseAtom() {
    skipWs();
    final ch = _peek();
    if (ch == null) throw _LatexParseException('Unexpected end of LaTeX');
    if (ch == '{') {
      _bump();
      final inner = parseSequence();
      if (_bump() != '}') throw _LatexParseException('Expected }');
      return inner;
    }
    if (ch == r'\') return parseCommand();
    if (ch == '^' || ch == '_' || ch == '}') {
      throw _LatexParseException('Expected math atom');
    }
    return EquationOmml.run(_bump());
  }

  String parseCommand() {
    _bump(); // \
    final name = parseCommandName();
    switch (name) {
      case 'frac':
        skipWs();
        final num = parseAtom();
        skipWs();
        final den = parseAtom();
        return '<m:f><m:num>$num</m:num><m:den>$den</m:den></m:f>';
      case 'sqrt':
        skipWs();
        var degree = '<m:deg/>';
        if (_peek() == '[') {
          _bump();
          final buf = StringBuffer();
          while (_peek() != null && _peek() != ']') {
            buf.write(_bump());
          }
          if (_bump() != ']') throw _LatexParseException('Expected ]');
          degree = EquationOmml.run(buf.toString());
        }
        skipWs();
        final body = parseAtom();
        return '<m:rad>$degree<m:e>$body</m:e></m:rad>';
      case 'text':
        skipWs();
        if (_bump() != '{') throw _LatexParseException('Expected {');
        final buf = StringBuffer();
        while (_peek() != null && _peek() != '}') {
          buf.write(_bump());
        }
        if (_bump() != '}') throw _LatexParseException('Expected }');
        return EquationOmml.run(buf.toString());
      case 'left':
      case 'right':
        skipWs();
        if (_peek() != null && '()[]|.'.contains(_peek()!)) {
          final ch = _bump();
          if (ch == '.') return '';
          return EquationOmml.run(ch);
        }
        return '';
      default:
        final sym = _commandSymbol(name);
        if (sym == null) {
          throw _LatexParseException('Unknown command: \\$name');
        }
        return EquationOmml.run(sym);
    }
  }

  String parseCommandName() {
    final buf = StringBuffer();
    while (_peek() != null) {
      final ch = _peek()!;
      if (!RegExp(r'[a-zA-Z]').hasMatch(ch)) break;
      buf.write(_bump());
    }
    if (buf.isEmpty) throw _LatexParseException('Expected command name');
    return buf.toString();
  }

  String applyScripts(String base) {
    String? sup;
    String? sub;
    while (true) {
      skipWs();
      final ch = _peek();
      if (ch == '^') {
        _bump();
        sup = parseAtom();
      } else if (ch == '_') {
        _bump();
        sub = parseAtom();
      } else {
        break;
      }
    }
    if (sup == null && sub == null) return base;
    if (sup != null && sub == null) {
      return '<m:sSup><m:e>$base</m:e><m:sup>$sup</m:sup></m:sSup>';
    }
    if (sup == null && sub != null) {
      return '<m:sSub><m:e>$base</m:e><m:sub>$sub</m:sub></m:sSub>';
    }
    return '<m:sSubSup><m:e>$base</m:e><m:sub>$sub</m:sub><m:sup>$sup</m:sup></m:sSubSup>';
  }

  static String? _commandSymbol(String name) {
    return switch (name) {
      'alpha' => 'α',
      'beta' => 'β',
      'gamma' => 'γ',
      'pi' => 'π',
      'Sigma' || 'sum' => 'Σ',
      'infty' => '∞',
      'pm' => '±',
      'times' => '×',
      'div' => '÷',
      'leq' || 'le' => '≤',
      'geq' || 'ge' => '≥',
      'neq' || 'ne' => '≠',
      'int' => '∫',
      'partial' => '∂',
      'theta' => 'θ',
      'Delta' => 'Δ',
      'lambda' => 'λ',
      'mu' => 'μ',
      'cdot' => '·',
      _ => null,
    };
  }
}
