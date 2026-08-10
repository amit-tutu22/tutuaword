import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ai_generate.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F28.S4 content generation', () {
    test('U-F28-S4-document-from-markdown-applies-headings', () {
      const md = '# Title\n## Section\nBody line\n- bullet';
      final paras = paragraphsFromMarkdown(md);
      expect(paras, hasLength(4));
      expect(paras[0].styleName, 'Heading 1');
      expect(paras[0].text, 'Title');
      expect(paras[1].styleName, 'Heading 2');
      expect(paras[2].styleName, isNull);
      expect(paras[3].text, '• bullet');
    });

    test('U-F28-S4-content-kind-prompt', () {
      final p = AiContentKind.outline.promptInstruction('launch');
      expect(p.toLowerCase(), contains('outline'));
      expect(p, contains('launch'));
    });

    test('U-F28-S4-generate-routes-local-for-small-context', () {
      final client = AiClient.productionDesktop(
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      expect(
        client.route(
          AiTask.generate,
          const AiDocumentContext(pageCount: 1, totalTokenEstimate: 100),
        ),
        AiProviderIds.llamaCpp,
      );
    });

    testWidgets('I-F28-S4-generate-outline-new-document', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        r'{"choices":[{"message":{"content":"# Project Plan\n## Goals\n- Ship MVP\n## Timeline\nQ3 kickoff"}}]}',
      );

      final engine = MockDocumentEngine(initialText: 'old draft');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk-test'
        ..geminiApiKey = 'gem-test'
        ..llamaEndpoint = 'http://127.0.0.1:11434';

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('ai_generate')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('ai_generate')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_generate_dialog')), findsOneWidget);
      await tester.tap(find.byKey(const Key('ai_generate_kind_outline')));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('ai_generate_topic')),
        'Project Plan',
      );
      await tester.tap(find.byKey(const Key('ai_generate_run')));
      await tester.pumpAndSettle();

      expect(find.textContaining('Project Plan'), findsWidgets);

      await tester.tap(find.byKey(const Key('ai_generate_open_new')));
      await tester.pumpAndSettle();

      expect(controller.documentText, contains('Project Plan'));
      expect(controller.documentText, contains('Goals'));
      expect(controller.documentText, isNot(contains('old draft')));
      expect(
        controller.sessionController.statusText.toLowerCase(),
        contains('outline'),
      );
    });

    testWidgets('I-F28-S4-generate-minutes-api', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        r'{"choices":[{"message":{"content":"# Sprint Sync\n## Attendees\nAlice\n## Action items\n- Draft"}}]}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk',
        geminiApiKey: 'g',
      );
      final generated = await generateContent(
        client: client,
        kind: AiContentKind.minutes,
        topic: 'Sprint Sync',
      );
      expect(generated.kind, AiContentKind.minutes);
      expect(generated.plainText, contains('Attendees'));
      expect(generated.paragraphs.first.styleName, 'Heading 1');

      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      await controller.openGeneratedDocument(generated);
      await controller.ensureLayoutReady();
      expect(controller.documentText, contains('Sprint Sync'));
    });

    test('S-F28-S4-generate-churn', () async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        r'{"choices":[{"message":{"content":"# T\n## S\nbody"}}]}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      const kinds = AiContentKind.values;
      for (var i = 0; i < 150; i++) {
        final generated = await generateContent(
          client: client,
          kind: kinds[i % kinds.length],
          topic: 'topic-$i',
        );
        expect(generated.paragraphCount, greaterThanOrEqualTo(1));
        expect(generated.markdown, isNotEmpty);
      }
    });
  });
}
