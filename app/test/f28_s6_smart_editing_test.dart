import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ai_smart_edit.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

const _messyDoc = 'Introduction\n'
    'This is a long body paragraph that explains the background in detail.\n'
    '\n'
    '\n'
    'Next Steps:\n'
    'Ship the MVP and gather feedback from users.\n'
    'A  B';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F28.S6 smart editing', () {
    test('U-F28-S6-heuristics-suggest-headings-and-notes', () {
      final plan = analyzeDocumentHeuristics(_messyDoc);
      expect(
        plan.headings.any((h) => h.previewText.contains('Introduction')),
        isTrue,
      );
      expect(plan.headings.any((h) => h.styleName.startsWith('Heading')), isTrue);
      expect(plan.insertToc, isTrue);
      expect(
        plan.autoFormatNotes.any(
          (n) => n.contains('blank') || n.contains('spacing'),
        ),
        isTrue,
      );
    });

    test('U-F28-S6-parse-ai-plan-overrides-indices', () {
      final plan = parseSmartEditPlan(
        '{"headings":[{"index":0,"style":"Heading 1"},{"index":4,"style":"Heading 2"}],"insert_toc":true}',
        _messyDoc,
      );
      expect(plan.headings, hasLength(2));
      expect(plan.headings[0].styleName, 'Heading 1');
      expect(plan.headings[1].paragraphIndex, 4);
      expect(plan.insertToc, isTrue);
    });

    testWidgets('I-F28-S6-smart-edit-from-review', (tester) async {
      final engine = MockDocumentEngine(initialText: _messyDoc);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('ai_smart_edit')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('ai_smart_edit')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_smart_edit_dialog')), findsOneWidget);
      expect(find.byKey(const Key('ai_smart_edit_heading_count')), findsOneWidget);
      expect(find.byKey(const Key('ai_smart_edit_notes')), findsOneWidget);

      await tester.tap(find.byKey(const Key('ai_smart_edit_accept')));
      await tester.pumpAndSettle();

      expect(controller.smartEditAppliedStyles, isNotEmpty);
      expect(controller.documentText, contains('Table of Contents'));
      expect(
        controller.sessionController.statusText.toLowerCase(),
        contains('smart edit'),
      );
    });

    test('I-F28-S6-suggest-smart-edit-via-ai', () async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        r'{"choices":[{"message":{"content":"{\"headings\":[{\"index\":0,\"style\":\"Heading 1\"}],\"insert_toc\":false}"}}]}',
      );
      final client = AiClient.productionDesktop(
        httpPost: http.postJson,
        openaiApiKey: 'sk-test',
        geminiApiKey: 'gem-test',
      );
      final plan = await suggestSmartEdit(
        client: client,
        documentText: _messyDoc,
      );
      expect(plan.headings, hasLength(1));
      expect(plan.headings.first.styleName, 'Heading 1');
      expect(plan.insertToc, isFalse);

      final engine = MockDocumentEngine(initialText: _messyDoc);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      await controller.applySmartEditPlan(plan);
      expect(controller.smartEditAppliedStyles, ['Heading 1']);
      expect(controller.documentText, isNot(contains('Table of Contents')));
    });

    test('S-F28-S6-analyze-apply-churn', () async {
      for (var i = 0; i < 150; i++) {
        final plan = analyzeDocumentHeuristics(_messyDoc)..insertToc = false;
        final engine = MockDocumentEngine(initialText: _messyDoc);
        final controller = createTestEditorController(engine: engine);
        await controller.applySmartEditPlan(plan);
        expect(controller.smartEditAppliedStyles, isNotEmpty, reason: 'iter $i');
        controller.dispose();
      }
    });
  });
}
