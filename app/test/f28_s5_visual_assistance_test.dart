import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ai_visual.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F28.S5 visual assistance', () {
    test('U-F28-S5-parse-table-suggestion', () {
      final s = parseVisualSuggestion(
        '{"kind":"table","rows":4,"cols":2,"title":"Compare","rationale":"side by side"}',
      );
      expect(s.title, 'Compare');
      expect(s.kind.type, AiVisualKindType.table);
      expect(s.kind.rows, 4);
      expect(s.kind.cols, 2);
    });

    test('U-F28-S5-parse-diagram-and-timeline', () {
      final d = parseVisualSuggestion(
        '{"kind":"diagram","diagram":"hierarchy","title":"Org","rationale":"structure"}',
      );
      expect(d.kind.type, AiVisualKindType.diagram);
      expect(d.kind.diagramType, 1);

      final t = parseVisualSuggestion(
        '{"kind":"timeline","stages":["Plan","Build","Ship"],"title":"Roadmap","rationale":"phases"}',
      );
      expect(t.kind.type, AiVisualKindType.timeline);
      expect(t.kind.stages, ['Plan', 'Build', 'Ship']);
    });

    testWidgets('I-F28-S5-suggest-and-insert-table', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        r'{"choices":[{"message":{"content":"{\"kind\":\"table\",\"rows\":2,\"cols\":3,\"title\":\"Grid\",\"rationale\":\"compare\"}"}}]}',
      );

      final engine = MockDocumentEngine(initialText: 'Compare options');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk-test'
        ..geminiApiKey = 'gem-test'
        ..llamaEndpoint = 'http://127.0.0.1:11434';

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('ai_visual')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('ai_visual')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_visual_dialog')), findsOneWidget);
      await tester.enterText(
        find.byKey(const Key('ai_visual_topic')),
        'comparison matrix',
      );
      await tester.tap(find.byKey(const Key('ai_visual_suggest')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_visual_title')), findsOneWidget);
      expect(find.textContaining('Grid'), findsOneWidget);
      expect(find.textContaining('2×3'), findsOneWidget);

      await tester.tap(find.byKey(const Key('ai_visual_insert')));
      await tester.pumpAndSettle();

      expect(engine.tableRows, 2);
      expect(engine.tableCols, 3);
      expect(
        controller.sessionController.statusText.toLowerCase(),
        contains('table'),
      );
    });

    test('I-F28-S5-insert-diagram-and-timeline-api', () async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.applyVisualSuggestion(
        const AiVisualSuggestion(
          kind: AiVisualKind.diagram(diagramType: 2),
          title: 'Cycle',
          rationale: 'loop',
        ),
      );
      expect(engine.lastDiagramType, 2);

      await controller.applyVisualSuggestion(
        const AiVisualSuggestion(
          kind: AiVisualKind.timeline(stages: ['A', 'B', 'C']),
          title: 'TL',
        ),
      );
      expect(engine.tableRows, 1);
      expect(engine.tableCols, 3);
    });

    test('S-F28-S5-suggest-insert-churn', () async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        r'{"choices":[{"message":{"content":"{\"kind\":\"timeline\",\"stages\":[\"A\",\"B\"],\"title\":\"T\",\"rationale\":\"r\"}"}}]}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk-test'
        ..geminiApiKey = 'gem-test';

      for (var i = 0; i < 120; i++) {
        final suggestion = await suggestVisual(client: client, topic: 'topic-$i');
        expect(suggestion.kind.type, AiVisualKindType.timeline);
        await controller.applyVisualSuggestion(suggestion);
        expect(engine.tableRows, 1);
        expect(engine.tableCols, 2);
      }
    });
  });
}
