import 'ollama_host.dart';

OllamaHost createOllamaHost() => FakeOllamaHost(
      healthy: false,
      ensureResult: const OllamaEnsureResult(
        status: OllamaEnsureStatus.unsupported,
        message:
            'Browser builds cannot start Ollama. Use cloud keys in AI Settings, '
            'or run the desktop app with Ollama installed.',
      ),
    );
