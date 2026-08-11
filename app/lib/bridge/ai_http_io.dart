import 'dart:convert';
import 'dart:io';

/// Default AI HTTP POST for desktop / VM targets (`dart:io`).
Future<String> defaultAiHttpPost(
  String url,
  Map<String, String> headers,
  String body,
) async {
  final client = HttpClient();
  try {
    final request = await client.postUrl(Uri.parse(url));
    headers.forEach(request.headers.set);
    request.add(utf8.encode(body));
    final response = await request.close();
    final text = await response.transform(utf8.decoder).join();
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw StateError('AI HTTP ${response.statusCode}: $text');
    }
    return text;
  } on SocketException catch (e) {
    throw StateError(
      'AI endpoint unreachable ($url). '
      'Start a local llama.cpp / Ollama server, or configure a cloud API key '
      'in Review → AI Settings. (${e.message})',
    );
  } finally {
    client.close(force: true);
  }
}

/// Reads an environment variable on IO platforms.
String? aiEnv(String key) => Platform.environment[key];
