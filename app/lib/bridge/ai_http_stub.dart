/// Fallback when neither `dart:io` nor `dart:html` is available.
Future<String> defaultAiHttpPost(
  String url,
  Map<String, String> headers,
  String body,
) async {
  throw StateError('AI HTTP transport not configured for this platform');
}

String? aiEnv(String key) => null;
