import 'ollama_host.dart';

OllamaHost createOllamaHost() => FakeOllamaHost(
      healthy: false,
      ensureResult: const OllamaEnsureResult(
        status: OllamaEnsureStatus.unsupported,
        message: 'Local Ollama is not available on this platform.',
      ),
    );
