/// Flutter-side AI facade mirroring `tw-ai` HybridRouter (F28.S1 / ADR-0010).
///
/// Product capability methods never take a provider ID. Routing internals may
/// use provider ids for tests and settings status only.

enum AiRoutingMode {
  automatic,
  alwaysLocal,
  alwaysCloud,
}

enum AiTask {
  grammar,
  correctGrammar,
  summarize,
  rewrite,
  translate,
  generate,
  explain,
  spellCheck,
  chat,
}

enum AiRewriteTone {
  neutral,
  formal,
  casual,
  shorten,
  expand,
}

/// Internal provider ids (settings / routing diagnostics only).
abstract final class AiProviderIds {
  static const openai = 'openai';
  static const gemini = 'gemini';
  static const llamaCpp = 'llama_cpp';
  static const rules = 'rules';
}

class AiDocumentContext {
  const AiDocumentContext({
    this.selectionText = '',
    this.pageCount = 1,
    this.totalTokenEstimate = 500,
  });

  final String selectionText;
  final int pageCount;
  final int totalTokenEstimate;
}

class AiProviderStatus {
  const AiProviderStatus({
    required this.id,
    required this.name,
    required this.local,
    required this.available,
  });

  final String id;
  final String name;
  final bool local;
  final bool available;
}

typedef AiHttpPost = Future<String> Function(
  String url,
  Map<String, String> headers,
  String body,
);

/// In-memory HTTP stub for unit/integration tests (no network).
class MockAiHttpClient {
  final Map<String, String> _exact = {};
  final List<(String, String)> _prefixes = [];
  final List<({String url, Map<String, String> headers, String body})> calls =
      [];

  void enqueue(String url, String body) => _exact[url] = body;

  void enqueuePrefix(String prefix, String body) =>
      _prefixes.add((prefix, body));

  Future<String> postJson(
    String url,
    Map<String, String> headers,
    String body,
  ) async {
    calls.add((url: url, headers: Map.of(headers), body: body));
    final exact = _exact[url];
    if (exact != null) return exact;
    for (final entry in _prefixes) {
      if (url.startsWith(entry.$1)) return entry.$2;
    }
    throw StateError('mock AI HTTP: no fixture for $url');
  }
}

/// Production-capable AI client used by the editor UI (F28.S1).
class AiClient {
  AiClient({
    AiHttpPost? httpPost,
    this.openaiApiKey = '',
    this.geminiApiKey = '',
    this.llamaEndpoint = 'http://127.0.0.1:11434',
    this.routingMode = AiRoutingMode.automatic,
    this.defaultCloud = AiProviderIds.openai,
    this.defaultLocal = AiProviderIds.llamaCpp,
  }) : httpPost = httpPost;

  /// Injectable HTTP (tests use [MockAiHttpClient]; production wires a real client later).
  AiHttpPost? httpPost;
  String openaiApiKey;
  String geminiApiKey;
  String llamaEndpoint;
  AiRoutingMode routingMode;
  String defaultCloud;
  String defaultLocal;

  /// Last routed provider id (diagnostics / tests only).
  String? lastRoutedProviderId;

  /// Completions issued (stress / integration).
  int completionCount = 0;

  List<AiProviderStatus> listProviders() => [
        AiProviderStatus(
          id: AiProviderIds.openai,
          name: 'OpenAI',
          local: false,
          available: openaiApiKey.trim().isNotEmpty,
        ),
        AiProviderStatus(
          id: AiProviderIds.gemini,
          name: 'Google Gemini',
          local: false,
          available: geminiApiKey.trim().isNotEmpty,
        ),
        AiProviderStatus(
          id: AiProviderIds.llamaCpp,
          name: 'llama.cpp (local)',
          local: true,
          available: llamaEndpoint.trim().isNotEmpty,
        ),
      ];

  void setRoutingMode(AiRoutingMode mode) => routingMode = mode;

  /// Select a provider for [task]. Returns an internal id for diagnostics.
  String route(AiTask task, AiDocumentContext ctx) {
    if (task == AiTask.spellCheck) {
      lastRoutedProviderId = AiProviderIds.rules;
      return AiProviderIds.rules;
    }
    final id = switch (routingMode) {
      AiRoutingMode.alwaysLocal => _resolveLocal(),
      AiRoutingMode.alwaysCloud => _resolveCloud(),
      AiRoutingMode.automatic => _prefersLocal(task, ctx)
          ? (_tryLocal() ?? _resolveCloud())
          : (_tryCloud() ?? _resolveLocal()),
    };
    lastRoutedProviderId = id;
    return id;
  }

  bool _prefersLocal(AiTask task, AiDocumentContext ctx) {
    switch (task) {
      case AiTask.grammar:
      case AiTask.correctGrammar:
      case AiTask.rewrite:
        return true;
      case AiTask.translate:
      case AiTask.explain:
      case AiTask.chat:
        return ctx.totalTokenEstimate < 4000;
      case AiTask.summarize:
        return ctx.pageCount <= 5 && ctx.totalTokenEstimate < 8000;
      case AiTask.generate:
        return ctx.totalTokenEstimate < 2000;
      case AiTask.spellCheck:
        return true;
    }
  }

  String? _tryLocal() {
    for (final id in [
      defaultLocal,
      AiProviderIds.llamaCpp,
    ]) {
      if (_isAvailable(id)) return id;
    }
    return null;
  }

  String? _tryCloud() {
    for (final id in [
      defaultCloud,
      AiProviderIds.openai,
      AiProviderIds.gemini,
    ]) {
      if (_isAvailable(id)) return id;
    }
    return null;
  }

  String _resolveLocal() =>
      _tryLocal() ?? (throw StateError('no local provider available'));

  String _resolveCloud() =>
      _tryCloud() ?? (throw StateError('no cloud provider available'));

  bool _isAvailable(String id) {
    return listProviders().any((p) => p.id == id && p.available);
  }

  Future<String> summarize(AiDocumentContext ctx) =>
      _complete(AiTask.summarize, ctx);

  Future<String> correctGrammar(AiDocumentContext ctx) =>
      _complete(AiTask.correctGrammar, ctx);

  Future<String> rewrite(AiDocumentContext ctx, AiRewriteTone tone) =>
      _complete(AiTask.rewrite, ctx, extra: ' Tone: ${tone.name}.');

  Future<String> translate(AiDocumentContext ctx, String lang) =>
      _complete(AiTask.translate, ctx, extra: ' Target language: $lang.');

  /// Document chat completion (F28.S3); [ctx.selectionText] holds the RAG prompt.
  Future<String> chatComplete(AiDocumentContext ctx) =>
      _complete(AiTask.chat, ctx);

  /// Content generation (F28.S4).
  Future<String> generate(AiDocumentContext ctx, String instruction) =>
      _complete(AiTask.generate, ctx, extra: ' Instruction: $instruction');

  Future<String> _complete(
    AiTask task,
    AiDocumentContext ctx, {
    String extra = '',
  }) async {
    final providerId = route(task, ctx);
    if (providerId == AiProviderIds.rules) {
      throw StateError('rules engine has no LLM completion');
    }
    final prompt = '${ctx.selectionText}$extra';
    final text = await _invokeProvider(providerId, prompt);
    completionCount++;
    return text;
  }

  Future<String> _invokeProvider(String providerId, String prompt) async {
    final post = httpPost;
    if (post == null) {
      throw StateError('AI HTTP transport not configured');
    }
    switch (providerId) {
      case AiProviderIds.openai:
        return _openaiComplete(post, prompt);
      case AiProviderIds.gemini:
        return _geminiComplete(post, prompt);
      case AiProviderIds.llamaCpp:
        return _llamaComplete(post, prompt);
      default:
        throw StateError('unknown provider $providerId');
    }
  }

  Future<String> _openaiComplete(AiHttpPost post, String prompt) async {
    if (openaiApiKey.trim().isEmpty) {
      throw StateError('OpenAI API key not configured');
    }
    final body =
        '{"model":"gpt-4o-mini","messages":[{"role":"user","content":${_jsonString(prompt)}}]}';
    final raw = await post(
      'https://api.openai.com/v1/chat/completions',
      {
        'Authorization': 'Bearer $openaiApiKey',
        'Content-Type': 'application/json',
      },
      body,
    );
    return _parseOpenAiChat(raw);
  }

  Future<String> _geminiComplete(AiHttpPost post, String prompt) async {
    if (geminiApiKey.trim().isEmpty) {
      throw StateError('Gemini API key not configured');
    }
    final url =
        'https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=$geminiApiKey';
    final body =
        '{"contents":[{"parts":[{"text":${_jsonString(prompt)}}]}]}';
    final raw = await post(url, {'Content-Type': 'application/json'}, body);
    return _parseGemini(raw);
  }

  Future<String> _llamaComplete(AiHttpPost post, String prompt) async {
    final base = llamaEndpoint.replaceAll(RegExp(r'/+$'), '');
    final body =
        '{"model":"llama3.2","messages":[{"role":"user","content":${_jsonString(prompt)}}]}';
    final raw = await post(
      '$base/v1/chat/completions',
      {'Content-Type': 'application/json'},
      body,
    );
    return _parseOpenAiChat(raw);
  }

  static String _jsonString(String value) {
    final escaped = value
        .replaceAll(r'\', r'\\')
        .replaceAll('"', r'\"')
        .replaceAll('\n', r'\n');
    return '"$escaped"';
  }

  static String _parseOpenAiChat(String raw) {
    final contentMatch =
        RegExp(r'"content"\s*:\s*"((?:\\.|[^"\\])*)"').firstMatch(raw);
    if (contentMatch == null) {
      throw StateError('OpenAI response missing content');
    }
    return contentMatch.group(1)!
        .replaceAll(r'\"', '"')
        .replaceAll(r'\n', '\n');
  }

  static String _parseGemini(String raw) {
    final textMatch =
        RegExp(r'"text"\s*:\s*"((?:\\.|[^"\\])*)"').firstMatch(raw);
    if (textMatch == null) {
      throw StateError('Gemini response missing text');
    }
    return textMatch.group(1)!
        .replaceAll(r'\"', '"')
        .replaceAll(r'\n', '\n');
  }

  /// Factory with production defaults and injectable HTTP (tests use mock).
  static AiClient productionDesktop({
    AiHttpPost? httpPost,
    String? openaiApiKey,
    String? geminiApiKey,
    String? llamaEndpoint,
  }) {
    return AiClient(
      httpPost: httpPost,
      openaiApiKey: openaiApiKey ?? '',
      geminiApiKey: geminiApiKey ?? '',
      llamaEndpoint: llamaEndpoint ?? 'http://127.0.0.1:11434',
    );
  }
}

