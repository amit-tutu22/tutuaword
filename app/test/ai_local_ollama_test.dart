import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ollama_host.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Ollama tags health status', () {
    test('only 2xx counts as healthy', () {
      expect(isOllamaTagsHealthyStatus(200), isTrue);
      expect(isOllamaTagsHealthyStatus(204), isTrue);
      expect(isOllamaTagsHealthyStatus(404), isFalse);
      expect(isOllamaTagsHealthyStatus(401), isFalse);
      expect(isOllamaTagsHealthyStatus(500), isFalse);
      expect(isOllamaTagsHealthyStatus(0), isFalse);
    });
  });

  group('AI local Ollama auto-start', () {
    test('Always Local ensures Ollama host', () async {
      final host = FakeOllamaHost(
        ensureResult: const OllamaEnsureResult(
          status: OllamaEnsureStatus.started,
          message: 'Started Ollama locally.',
        ),
      );
      final client = AiClient.productionDesktop(
        ollamaHost: host,
        openaiApiKey: 'sk-test',
      );

      final result = await client.applyRoutingMode(AiRoutingMode.alwaysLocal);

      expect(client.routingMode, AiRoutingMode.alwaysLocal);
      expect(host.ensureCalls, 1);
      expect(result?.ok, isTrue);
      expect(client.lastOllamaEnsure?.status, OllamaEnsureStatus.started);
    });

    test('Always Cloud skips Ollama start', () async {
      final host = FakeOllamaHost();
      final client = AiClient.productionDesktop(
        ollamaHost: host,
        openaiApiKey: 'sk-test',
      )..setRoutingMode(AiRoutingMode.alwaysLocal);

      final result = await client.applyRoutingMode(AiRoutingMode.alwaysCloud);

      expect(client.routingMode, AiRoutingMode.alwaysCloud);
      expect(host.ensureCalls, 0);
      expect(result, isNull);
    });

    test('local completion probes Ollama before HTTP', () async {
      final host = FakeOllamaHost(
        ensureResult: const OllamaEnsureResult(
          status: OllamaEnsureStatus.alreadyRunning,
        ),
      );
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"local ok"}}]}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        ollamaHost: host,
      )..setRoutingMode(AiRoutingMode.alwaysLocal);

      final text = await client.correctGrammar(
        const AiDocumentContext(selectionText: 'teh'),
      );

      expect(text, 'local ok');
      expect(host.ensureCalls, greaterThanOrEqualTo(1));
    });

    test('local completion surfaces ensure failure', () async {
      final host = FakeOllamaHost(
        ensureResult: const OllamaEnsureResult(
          status: OllamaEnsureStatus.unavailable,
          message: 'Ollama is not installed.',
        ),
      );
      final client = AiClient.productionDesktop(
        httpPost: MockAiHttpClient().postJson,
        ollamaHost: host,
      )..setRoutingMode(AiRoutingMode.alwaysLocal);

      await expectLater(
        client.correctGrammar(const AiDocumentContext(selectionText: 'teh')),
        throwsA(
          isA<StateError>().having(
            (e) => e.message,
            'message',
            contains('Ollama is not installed'),
          ),
        ),
      );
    });
  });
}
