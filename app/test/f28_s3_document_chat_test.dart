import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/ai_chat.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ai_chat_dialog.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

const _sampleDoc =
    'Budget overview: Q3 spending is under control.\n'
    'Risk assessment: supply chain delays remain the top risk.\n'
    'Next steps: schedule a vendor review meeting next week.';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F28.S3 document chat / RAG', () {
    test('U-F28-S3-chunk-document-by-paragraph', () {
      final chunks = chunkDocumentText(_sampleDoc);
      expect(chunks.length, 3);
      expect(chunks[0].text, contains('Budget'));
      expect(chunks[1].paragraphId, 'para-1');
      expect(chunks[1].text, contains('Risk'));
    });

    test('U-F28-S3-rag-retrieve-by-keywords', () {
      final index = AiDocumentRagIndex.fromText(_sampleDoc);
      final hits = index.retrieve('What is the top risk?', 2);
      expect(hits, isNotEmpty);
      expect(hits.first.text.toLowerCase(), contains('risk'));
    });

    test('U-F28-S3-chat-routes-local-for-small-context', () {
      final client = AiClient.productionDesktop(
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      final id = client.route(
        AiTask.chat,
        const AiDocumentContext(pageCount: 1, totalTokenEstimate: 400),
      );
      expect(id, AiProviderIds.llamaCpp);
    });

    testWidgets('I-F28-S3-chat-from-review-with-citations', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"Supply chain delays are the top risk. [[cite:para-1]]"}}]}',
      );

      final engine = MockDocumentEngine(initialText: _sampleDoc);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk-test'
        ..geminiApiKey = 'gem-test'
        ..llamaEndpoint = 'http://127.0.0.1:11434';

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('ai_chat')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('ai_chat')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_chat_dialog')), findsOneWidget);
      expect(find.byKey(const Key('ai_chat_chunk_count')), findsOneWidget);

      await tester.enterText(
        find.byKey(const Key('ai_chat_input')),
        'What is the top risk?',
      );
      await tester.tap(find.byKey(const Key('ai_chat_send')));
      await tester.pumpAndSettle();

      expect(
        find.textContaining('Supply chain delays are the top risk'),
        findsOneWidget,
      );
      expect(find.byKey(const Key('ai_chat_cite_para-1')), findsOneWidget);

      await tester.tap(find.byKey(const Key('ai_chat_cite_para-1')));
      await tester.pumpAndSettle();
      expect(controller.sessionController.statusText, contains('para-1'));
    });

    testWidgets('I-F28-S3-chat-fallback-retrieved-citations', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"Budget looks healthy."}}]}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk',
        geminiApiKey: 'g',
      );
      final session = AiDocumentChatSession.fromText(_sampleDoc);
      final reply = await session.ask(client, 'budget spending overview');
      expect(reply.role, AiChatRole.assistant);
      expect(reply.references, isNotEmpty);
      expect(reply.references.first.excerpt, contains('Budget'));

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('open_chat'),
                onPressed: () => AiChatDialog.show(
                  context,
                  client: client,
                  documentText: _sampleDoc,
                ),
                child: const Text('Chat'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byKey(const Key('open_chat')));
      await tester.pumpAndSettle();
      expect(find.textContaining('paragraphs indexed'), findsOneWidget);
    });

    test('S-F28-S3-chunk-retrieve-chat-churn', () async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"ok"}}]}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      final session = AiDocumentChatSession.fromText(_sampleDoc);
      for (var i = 0; i < 200; i++) {
        final q = i.isEven ? 'budget spending overview' : 'risk assessment delays';
        expect(session.index.retrieve(q, 2), isNotEmpty);
        final reply = await session.ask(client, q);
        expect(reply.content, isNotEmpty);
      }
      expect(session.history.length, greaterThanOrEqualTo(400));
    });
  });
}
