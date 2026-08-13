/// Result of ensuring a local Ollama / llama.cpp server is reachable.
enum OllamaEnsureStatus {
  alreadyRunning,
  started,
  unavailable,
  unsupported,
}

class OllamaEnsureResult {
  const OllamaEnsureResult({
    required this.status,
    this.message = '',
  });

  final OllamaEnsureStatus status;
  final String message;

  bool get ok =>
      status == OllamaEnsureStatus.alreadyRunning ||
      status == OllamaEnsureStatus.started;
}

/// Starts / probes a local OpenAI-compatible server (Ollama default :11434).
abstract class OllamaHost {
  Future<bool> isHealthy(String endpoint);

  /// If [endpoint] is not healthy, attempt to start `ollama serve` (desktop).
  Future<OllamaEnsureResult> ensureRunning(String endpoint);
}

/// Test / web double that never starts a process.
class FakeOllamaHost implements OllamaHost {
  FakeOllamaHost({
    this.healthy = true,
    this.ensureResult = const OllamaEnsureResult(
      status: OllamaEnsureStatus.alreadyRunning,
      message: 'fake',
    ),
  });

  bool healthy;
  OllamaEnsureResult ensureResult;
  int ensureCalls = 0;

  @override
  Future<bool> isHealthy(String endpoint) async => healthy;

  @override
  Future<OllamaEnsureResult> ensureRunning(String endpoint) async {
    ensureCalls++;
    return ensureResult;
  }
}
