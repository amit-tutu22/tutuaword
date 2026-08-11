/// One message in a comment thread.
class CommentMessageView {
  const CommentMessageView({required this.author, required this.body});

  final String author;
  final String body;

  factory CommentMessageView.fromJson(Map<String, dynamic> json) {
    final bodyBlocks = json['body'] as List? ?? const [];
    final text = bodyBlocks
        .whereType<Map>()
        .map((block) {
          final runs = block['runs'] as List? ?? const [];
          return runs
              .whereType<Map>()
              .map((run) => run['text']?.toString() ?? '')
              .join();
        })
        .join('\n');
    return CommentMessageView(
      author: json['author']?.toString() ?? 'Author',
      body: text.isEmpty ? '(empty)' : text,
    );
  }
}

/// Comment thread for the Review comments pane.
class CommentThreadView {
  const CommentThreadView({
    required this.commentId,
    required this.messages,
    required this.resolved,
  });

  final int commentId;
  final List<CommentMessageView> messages;
  final bool resolved;

  factory CommentThreadView.fromJson(Map<String, dynamic> json) {
    return CommentThreadView(
      commentId: json['comment_id'] as int? ?? 0,
      resolved: json['resolved'] as bool? ?? false,
      messages: (json['messages'] as List? ?? const [])
          .whereType<Map>()
          .map(
            (item) => CommentMessageView.fromJson(
              Map<String, dynamic>.from(item),
            ),
          )
          .toList(),
    );
  }

  String get preview =>
      messages.isEmpty ? '(no text)' : messages.first.body;
}

class CommentThreadList {
  const CommentThreadList(this.threads);

  final List<CommentThreadView> threads;

  factory CommentThreadList.parse(String json) {
    if (json.trim().isEmpty) return const CommentThreadList([]);
    try {
      final decoded = _decode(json);
      if (decoded is! List) return const CommentThreadList([]);
      return CommentThreadList(
        decoded
            .whereType<Map>()
            .map((item) => CommentThreadView.fromJson(
                  Map<String, dynamic>.from(item),
                ))
            .where((t) => t.commentId >= 0)
            .toList(),
      );
    } catch (_) {
      return const CommentThreadList([]);
    }
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
