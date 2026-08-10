import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F28.S1 production AI providers', () {
    test('U-F28-S1-hybrid-router-local', () {
      final http = MockAiHttpClient();
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      final id = client.route(
        AiTask.grammar,
        const AiDocumentContext(pageCount: 1, totalTokenEstimate: 500),
      );
      expect(id, AiProviderIds.llamaCpp);
    });

    test('U-F28-S1-hybrid-router-cloud', () {
      final http = MockAiHttpClient();
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      final id = client.route(
        AiTask.summarize,
        const AiDocumentContext(pageCount: 200, totalTokenEstimate: 120000),
      );
      expect(id, AiProviderIds.openai);
    });

    test('U-F28-S1-openai-adapter-parses-response', () async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'https://api.openai.com/',
        '{"choices":[{"message":{"content":"cloud summary"}}],"usage":{"total_tokens":42}}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk-test',
      );
      final text = await client.summarize(
        const AiDocumentContext(
          selectionText: 'Long document text…',
          pageCount: 200,
          totalTokenEstimate: 120000,
        ),
      );
      expect(text, 'cloud summary');
      expect(http.calls.first.headers['Authorization'], contains('sk-test'));
    });

    test('U-F28-S1-always-local-mode', () {
      final client = AiClient.productionDesktop(
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      )..setRoutingMode(AiRoutingMode.alwaysLocal);
      expect(
        client.route(
          AiTask.summarize,
          const AiDocumentContext(pageCount: 200, totalTokenEstimate: 120000),
        ),
        AiProviderIds.llamaCpp,
      );
    });

    testWidgets('I-F28-S1-ai-settings-from-review', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('ai_settings')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('ai_settings')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_settings_dialog')), findsOneWidget);
      expect(find.byKey(const Key('ai_routing_automatic')), findsOneWidget);
      expect(find.byKey(const Key('ai_provider_openai')), findsOneWidget);
      expect(find.byKey(const Key('ai_provider_llama_cpp')), findsOneWidget);

      await tester.tap(find.byKey(const Key('ai_routing_always_local')));
      await tester.pumpAndSettle();
      expect(controller.aiClient.routingMode, AiRoutingMode.alwaysLocal);

      await tester.tap(find.byKey(const Key('ai_settings_close')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('ai_settings_dialog')), findsNothing);
    });

    testWidgets('I-F28-S1-switch-routing-without-restart', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'https://api.openai.com/',
        '{"choices":[{"message":{"content":"cloud summary"}}]}',
      );
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"local grammar fix"}}]}',
      );

      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk-test'
        ..geminiApiKey = 'gem-test'
        ..llamaEndpoint = 'http://127.0.0.1:11434';

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('open_ai'),
                onPressed: () => controller.openAiSettings(context),
                child: const Text('AI'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byKey(const Key('open_ai')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('ai_routing_always_cloud')));
      await tester.pumpAndSettle();
      expect(controller.aiClient.routingMode, AiRoutingMode.alwaysCloud);

      final cloud = await controller.aiClient.summarize(
        const AiDocumentContext(
          selectionText: 'doc',
          pageCount: 1,
          totalTokenEstimate: 100,
        ),
      );
      expect(cloud, 'cloud summary');
      expect(controller.aiClient.lastRoutedProviderId, AiProviderIds.openai);

      await tester.tap(find.byKey(const Key('ai_routing_always_local')));
      await tester.pumpAndSettle();
      expect(controller.aiClient.routingMode, AiRoutingMode.alwaysLocal);

      final local = await controller.aiClient.correctGrammar(
        const AiDocumentContext(selectionText: 'This are wrong.'),
      );
      expect(local, 'local grammar fix');
      expect(controller.aiClient.lastRoutedProviderId, AiProviderIds.llamaCpp);
    });

    test('S-F28-S1-router-and-complete-churn', () async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'https://api.openai.com/',
        '{"choices":[{"message":{"content":"cloud summary"}}]}',
      );
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"local grammar fix"}}]}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      const small = AiDocumentContext(pageCount: 1, totalTokenEstimate: 500);
      const large = AiDocumentContext(pageCount: 200, totalTokenEstimate: 120000);
      for (var i = 0; i < 300; i++) {
        if (i.isEven) {
          expect(client.route(AiTask.grammar, small), AiProviderIds.llamaCpp);
          await client.correctGrammar(small);
        } else {
          expect(client.route(AiTask.summarize, large), AiProviderIds.openai);
          await client.summarize(large);
        }
      }
      expect(client.completionCount, 300);
      expect(http.calls.length, greaterThanOrEqualTo(300));
    });
  });
}
