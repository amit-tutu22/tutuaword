import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'ollama_host.dart';

OllamaHost createOllamaHost() => IoOllamaHost();

/// Desktop/mobile IO host: probe endpoint, start `ollama serve` when needed.
class IoOllamaHost implements OllamaHost {
  @override
  Future<bool> isHealthy(String endpoint) async {
    final uri = _tagsUri(endpoint);
    final client = HttpClient();
    try {
      final request = await client.getUrl(uri).timeout(const Duration(seconds: 2));
      final response =
          await request.close().timeout(const Duration(seconds: 2));
      await response.drain<void>();
      return response.statusCode >= 200 && response.statusCode < 500;
    } catch (_) {
      return false;
    } finally {
      client.close(force: true);
    }
  }

  @override
  Future<OllamaEnsureResult> ensureRunning(String endpoint) async {
    if (await isHealthy(endpoint)) {
      return const OllamaEnsureResult(
        status: OllamaEnsureStatus.alreadyRunning,
        message: 'Local AI server is running.',
      );
    }

    if (!_isLocalEndpoint(endpoint)) {
      return OllamaEnsureResult(
        status: OllamaEnsureStatus.unavailable,
        message:
            'Endpoint $endpoint is unreachable. Start that server manually, '
            'or use http://127.0.0.1:11434 for auto-start.',
      );
    }

    final launchError = await _launchOllama();
    if (launchError != null) {
      return OllamaEnsureResult(
        status: OllamaEnsureStatus.unavailable,
        message: launchError,
      );
    }

    final ready = await _waitHealthy(endpoint, const Duration(seconds: 45));
    if (ready) {
      return const OllamaEnsureResult(
        status: OllamaEnsureStatus.started,
        message:
            'Started Ollama locally. If prompts fail, run: ollama pull llama3.2',
      );
    }
    return const OllamaEnsureResult(
      status: OllamaEnsureStatus.unavailable,
      message:
          'Ollama did not become ready. Open the Ollama app (or run '
          '`ollama serve` in Terminal), then try again.',
    );
  }

  /// Returns null on success, or a user-facing error string.
  Future<String?> _launchOllama() async {
    if (Platform.isMacOS) {
      final openedApp = await _tryOpenMacOllamaApp();
      if (openedApp) return null;
    }

    final binary = await _resolveOllamaBinary();
    if (binary == null) {
      return 'Ollama is not installed. Install from https://ollama.com '
          '(or brew install ollama), open the Ollama app once, then try again.';
    }

    try {
      await Process.start(
        binary,
        const ['serve'],
        mode: ProcessStartMode.detached,
      );
      return null;
    } on ProcessException catch (e) {
      if (_isPermissionDenied(e)) {
        return 'macOS blocked starting Ollama from the app. Open the Ollama '
            'app from Applications (or run `ollama serve` in Terminal), then '
            'try Generate again.';
      }
      return 'Failed to start Ollama ($e).';
    } catch (e) {
      final text = e.toString();
      if (_isPermissionDeniedText(text)) {
        return 'macOS blocked starting Ollama from the app. Open the Ollama '
            'app from Applications (or run `ollama serve` in Terminal), then '
            'try Generate again.';
      }
      return 'Failed to start Ollama ($e).';
    }
  }

  /// Launch Services path — works under App Sandbox when Ollama.app is installed.
  Future<bool> _tryOpenMacOllamaApp() async {
    final candidates = <String>[
      '/Applications/Ollama.app',
      if (Platform.environment['HOME'] != null)
        '${Platform.environment['HOME']}/Applications/Ollama.app',
    ];
    for (final app in candidates) {
      if (!await Directory(app).exists()) continue;
      try {
        final result = await Process.run('/usr/bin/open', ['-a', app]);
        if (result.exitCode == 0) return true;
      } catch (_) {}
    }
    try {
      final result = await Process.run('/usr/bin/open', ['-a', 'Ollama']);
      return result.exitCode == 0;
    } catch (_) {
      return false;
    }
  }

  bool _isPermissionDenied(ProcessException e) {
    return _isPermissionDeniedText(e.message) ||
        _isPermissionDeniedText(e.toString());
  }

  bool _isPermissionDeniedText(String text) {
    final lower = text.toLowerCase();
    return lower.contains('operation not permitted') ||
        lower.contains('permission denied');
  }

  Future<bool> _waitHealthy(String endpoint, Duration timeout) async {
    final deadline = DateTime.now().add(timeout);
    while (DateTime.now().isBefore(deadline)) {
      if (await isHealthy(endpoint)) return true;
      await Future<void>.delayed(const Duration(milliseconds: 500));
    }
    return false;
  }

  Uri _tagsUri(String endpoint) {
    final base = endpoint.replaceAll(RegExp(r'/+$'), '');
    return Uri.parse('$base/api/tags');
  }

  bool _isLocalEndpoint(String endpoint) {
    final uri = Uri.tryParse(endpoint);
    if (uri == null) return false;
    final host = uri.host.toLowerCase();
    return host == '127.0.0.1' || host == 'localhost' || host == '::1';
  }

  Future<String?> _resolveOllamaBinary() async {
    final pathEnv = Platform.environment['PATH'] ?? '';
    final sep = Platform.isWindows ? ';' : ':';
    final dirs = <String>[
      if (Platform.environment['HOME'] != null)
        '${Platform.environment['HOME']}/.local/bin',
      '/opt/homebrew/bin',
      '/usr/local/bin',
      ...pathEnv.split(sep),
    ];
    final names =
        Platform.isWindows ? const ['ollama.exe', 'ollama'] : const ['ollama'];
    for (final dir in dirs) {
      if (dir.isEmpty) continue;
      for (final name in names) {
        final candidate = Platform.isWindows ? '$dir\\$name' : '$dir/$name';
        if (await File(candidate).exists()) return candidate;
      }
    }
    try {
      final which = await Process.run(
        Platform.isWindows ? 'where' : 'which',
        ['ollama'],
      );
      if (which.exitCode == 0) {
        final line = (which.stdout as String)
            .split(RegExp(r'\r?\n'))
            .map((s) => s.trim())
            .firstWhere((s) => s.isNotEmpty, orElse: () => '');
        if (line.isNotEmpty) return line;
      }
    } catch (_) {}
    return null;
  }
}

/// Decode helper kept for future model listing UI.
List<String> decodeOllamaModelNames(String body) {
  try {
    final map = jsonDecode(body) as Map<String, dynamic>;
    final models = map['models'] as List<dynamic>? ?? const [];
    return models
        .map((m) => (m as Map<String, dynamic>)['name']?.toString() ?? '')
        .where((n) => n.isNotEmpty)
        .toList();
  } catch (_) {
    return const [];
  }
}
