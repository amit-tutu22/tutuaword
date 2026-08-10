import 'dart:html' as html;

/// Default AI HTTP POST for web (`dart:html`).
Future<String> defaultAiHttpPost(
  String url,
  Map<String, String> headers,
  String body,
) async {
  try {
    final response = await html.HttpRequest.request(
      url,
      method: 'POST',
      sendData: body,
      requestHeaders: headers,
    );
    final status = response.status ?? 0;
    final text = response.responseText ?? '';
    if (status < 200 || status >= 300) {
      throw StateError('AI HTTP $status: $text');
    }
    return text;
  } on html.ProgressEvent catch (_) {
    throw StateError(
      'AI endpoint unreachable ($url). '
      'Configure a cloud API key in Review → AI Settings, or run a local '
      'llama.cpp / Ollama server.',
    );
  }
}

/// Web builds do not expose process environment secrets.
String? aiEnv(String key) => null;
