import 'dart:async';
import 'dart:convert';
import 'dart:io';

/// Maximum AI response body size (16 MiB).
const _maxAiResponseBytes = 16 * 1024 * 1024;

/// Default AI HTTP POST for desktop / VM targets (`dart:io`).
Future<String> defaultAiHttpPost(
  String url,
  Map<String, String> headers,
  String body,
) async {
  final client = HttpClient();
  client.connectionTimeout = const Duration(seconds: 15);
  try {
    final request = await client
        .postUrl(Uri.parse(url))
        .timeout(const Duration(seconds: 30));
    headers.forEach(request.headers.set);
    request.add(utf8.encode(body));
    final response = await request.close().timeout(const Duration(seconds: 120));
    final buffer = StringBuffer();
    var received = 0;
    await for (final chunk in response.transform(utf8.decoder)) {
      received += chunk.length;
      if (received > _maxAiResponseBytes) {
        throw StateError('AI response too large');
      }
      buffer.write(chunk);
    }
    final text = buffer.toString();
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
  } on TimeoutException {
    throw StateError('AI request timed out ($url)');
  } finally {
    client.close(force: true);
  }
}

/// Reads an environment variable on IO platforms.
String? aiEnv(String key) => Platform.environment[key];
