/// Spell-check issue with Hunspell-style suggestions (F17.S1 UX).
class SpellIssue {
  const SpellIssue({
    required this.word,
    this.start,
    this.end,
    this.suggestions = const [],
  });

  final String word;
  final int? start;
  final int? end;
  final List<String> suggestions;

  static List<SpellIssue> parseJsonList(String json) {
    if (json.trim().isEmpty) return const [];
    try {
      final decoded = _decode(json);
      if (decoded is! List) return const [];
      return decoded
          .whereType<Map>()
          .map(
            (item) => SpellIssue(
              word: item['word']?.toString() ?? '',
              start: _asInt(item['start']),
              end: _asInt(item['end']),
              suggestions: (item['suggestions'] as List?)
                      ?.map((s) => s.toString())
                      .toList() ??
                  const [],
            ),
          )
          .where((issue) => issue.word.isNotEmpty)
          .toList();
    } catch (_) {
      return json
          .split('\n')
          .where((line) => line.trim().isNotEmpty)
          .map((word) => SpellIssue(word: word))
          .toList();
    }
  }

  static int? _asInt(dynamic value) {
    if (value == null) return null;
    if (value is int) return value;
    return int.tryParse(value.toString());
  }

  static dynamic _decode(String source) {
    return _JsonDecoder(source).decode();
  }
}

class _JsonDecoder {
  _JsonDecoder(this.source);
  final String source;
  int _i = 0;

  dynamic decode() => _parseValue();

  dynamic _parseValue() {
    _skipWs();
    if (_i >= source.length) return null;
    final c = source[_i];
    if (c == '{') return _parseObject();
    if (c == '[') return _parseArray();
    if (c == '"') return _parseString();
    return _parseLiteral();
  }

  Map<String, dynamic> _parseObject() {
    _i++;
    final map = <String, dynamic>{};
    _skipWs();
    if (_peek() == '}') {
      _i++;
      return map;
    }
    while (_i < source.length) {
      _skipWs();
      final key = _parseString();
      _skipWs();
      if (_peek() == ':') _i++;
      map[key] = _parseValue();
      _skipWs();
      if (_peek() == ',') {
        _i++;
        continue;
      }
      if (_peek() == '}') {
        _i++;
        break;
      }
    }
    return map;
  }

  List<dynamic> _parseArray() {
    _i++;
    final list = <dynamic>[];
    _skipWs();
    if (_peek() == ']') {
      _i++;
      return list;
    }
    while (_i < source.length) {
      list.add(_parseValue());
      _skipWs();
      if (_peek() == ',') {
        _i++;
        continue;
      }
      if (_peek() == ']') {
        _i++;
        break;
      }
    }
    return list;
  }

  String _parseString() {
    _i++;
    final buf = StringBuffer();
    while (_i < source.length) {
      final c = source[_i++];
      if (c == '"') break;
      if (c == '\\' && _i < source.length) {
        buf.write(source[_i++]);
      } else {
        buf.write(c);
      }
    }
    return buf.toString();
  }

  dynamic _parseLiteral() {
    final start = _i;
    while (_i < source.length && ',]}'.contains(source[_i]) == false) {
      _i++;
    }
    final raw = source.substring(start, _i).trim();
    if (raw == 'true') return true;
    if (raw == 'false') return false;
    if (raw == 'null') return null;
    return num.tryParse(raw) ?? raw;
  }

  void _skipWs() {
    while (_i < source.length && ' \n\r\t'.contains(source[_i])) {
      _i++;
    }
  }

  String _peek() => _i < source.length ? source[_i] : '';
}
