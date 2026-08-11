import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ai_rewrite.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F28.S2 writing assistant rewrite', () {
    test('U-F28-S2-replace-commands-delete-then-insert', () {
      final cmds = replaceRunRangeCommands(
        runId: 'run-1',
        start: 0,
        end: 5,
        text: 'Hello',
      );
      expect(cmds, hasLength(2));
      expect(cmds[0].isDelete, isTrue);
      expect(cmds[0].start, 0);
      expect(cmds[0].end, 5);
      expect(cmds[1].isInsert, isTrue);
      expect(cmds[1].offset, 0);
      expect(cmds[1].text, 'Hello');
    });

    test('U-F28-S2-rewrite-routes-local', () {
      final client = AiClient.productionDesktop(
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      final id = client.route(
        AiTask.rewrite,
        const AiDocumentContext(pageCount: 1, totalTokenEstimate: 80),
      );
      expect(id, AiProviderIds.llamaCpp);
    });

    test('U-F28-S2-apply-suggestion-replaces-text', () async {
      final engine = MockDocumentEngine(initialText: 'hello world');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      final ok = await controller.applyAiTextSuggestion(
        runId: engine.defaultRunId,
        start: 0,
        end: 5,
        text: 'HELLO',
      );
      expect(ok, isTrue);
      expect(controller.documentText, 'HELLO world');
    });

    testWidgets('I-F28-S2-rewrite-selection', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"Rewritten prose."}}]}',
      );

      final engine = MockDocumentEngine(initialText: 'original draft');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk-test'
        ..geminiApiKey = 'gem-test'
        ..llamaEndpoint = 'http://127.0.0.1:11434';

      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 0),
          focus: DocPosition(
            runId: engine.defaultRunId,
            offset: 'original draft'.length,
          ),
        ),
      );
      expect(controller.selectedText, 'original draft');

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('ai_rewrite')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('ai_rewrite')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_rewrite_dialog')), findsOneWidget);
      expect(find.byKey(const Key('ai_rewrite_original')), findsOneWidget);
      expect(find.text('Rewritten prose.'), findsOneWidget);

      await tester.tap(find.byKey(const Key('ai_rewrite_accept')));
      await tester.pumpAndSettle();

      expect(controller.documentText, 'Rewritten prose.');

      await controller.undo();
      await tester.pumpAndSettle();
      expect(controller.documentText, 'original draft');
    });

    testWidgets('I-F28-S2-rewrite-discard-keeps-text', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"Should not apply"}}]}',
      );
      final engine = MockDocumentEngine(initialText: 'keep me');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk'
        ..geminiApiKey = 'g'
        ..llamaEndpoint = 'http://127.0.0.1:11434';

      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 0),
          focus: DocPosition(runId: engine.defaultRunId, offset: 7),
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('do_rewrite'),
                onPressed: () => controller.rewriteSelection(context),
                child: const Text('Rewrite'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byKey(const Key('do_rewrite')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('ai_rewrite_discard')));
      await tester.pumpAndSettle();
      expect(controller.documentText, 'keep me');
    });

    test('S-F28-S2-rewrite-apply-undo-churn', () async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"Rewritten prose."}}]}',
      );
      final engine = MockDocumentEngine(initialText: 'seed');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk-test'
        ..geminiApiKey = 'gem-test'
        ..llamaEndpoint = 'http://127.0.0.1:11434';

      for (var i = 0; i < 200; i++) {
        final before = controller.documentText;
        final suggestion = await controller.aiClient.rewrite(
          AiDocumentContext(
            selectionText: before,
            pageCount: 1,
            totalTokenEstimate: before.length,
          ),
          AiRewriteTone.neutral,
        );
        expect(suggestion, 'Rewritten prose.');
        final ok = await controller.applyAiTextSuggestion(
          runId: engine.defaultRunId,
          start: 0,
          end: before.length,
          text: suggestion,
        );
        expect(ok, isTrue);
        expect(controller.documentText, 'Rewritten prose.');
        await controller.undo();
        await controller.ensureLayoutReady();
        expect(controller.documentText, before, reason: 'iter $i');
      }
    });
  });
}
