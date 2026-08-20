import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/thesaurus.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Review Translate / Thesaurus / Language', () {
    test('U-thesaurus-local-lookup', () {
      expect(lookupThesaurus('happy'), contains('glad'));
      expect(thesaurusLemma('  Happy! '), 'happy');
      expect(parseThesaurusAiResponse('glad, joyful, cheerful', exclude: 'happy'),
          containsAll(['glad', 'joyful', 'cheerful']));
    });

    testWidgets('I-review-language-sets-proofing-language', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('review_language')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('review_language')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('language_dialog')), findsOneWidget);
      await tester.tap(find.byKey(const Key('language_option_es')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('language_ok')));
      await tester.pumpAndSettle();

      expect(controller.proofingLanguageId, 'es');
      expect(controller.proofingLanguage.translateName, 'Spanish');
    });

    testWidgets('I-review-thesaurus-uses-word-at-caret', (tester) async {
      final engine = MockDocumentEngine(initialText: 'I am happy today');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      // Caret inside "happy" with no selection — Word expands to the word.
      controller.selectionController.setCaret(engine.defaultRunId, 7);
      expect(controller.hasGlyphSelection, isFalse);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('review_thesaurus')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('review_thesaurus')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('thesaurus_dialog')), findsOneWidget);
      expect(find.textContaining('happy'), findsWidgets);
    });

    testWidgets('I-review-thesaurus-replaces-word', (tester) async {
      final engine = MockDocumentEngine(initialText: 'I am happy today');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      // Select "happy"
      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 5),
          focus: DocPosition(runId: engine.defaultRunId, offset: 10),
        ),
      );
      expect(controller.selectedText, 'happy');

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('review_thesaurus')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('review_thesaurus')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('thesaurus_dialog')), findsOneWidget);
      await tester.tap(find.byKey(const Key('thesaurus_item_glad')));
      await tester.pumpAndSettle();

      expect(controller.documentText, 'I am glad today');
    });

    testWidgets('I-review-translate-applies-ai-result', (tester) async {
      final http = MockAiHttpClient();
      http.enqueuePrefix(
        'http://127.0.0.1:11434/',
        '{"choices":[{"message":{"content":"Hola mundo"}}]}',
      );

      final engine = MockDocumentEngine(initialText: 'Hello world');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.aiClient
        ..httpPost = http.postJson
        ..openaiApiKey = 'sk-test'
        ..llamaEndpoint = 'http://127.0.0.1:11434';

      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 0),
          focus: DocPosition(
            runId: engine.defaultRunId,
            offset: 'Hello world'.length,
          ),
        ),
      );

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('review_translate')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('review_translate')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_translate_language_picker')), findsOneWidget);
      await tester.tap(find.byKey(const Key('ai_translate_language_ok')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('ai_translate_dialog')), findsOneWidget);
      expect(find.text('Hola mundo'), findsOneWidget);
      await tester.tap(find.byKey(const Key('ai_translate_accept')));
      await tester.pumpAndSettle();

      expect(controller.documentText, 'Hola mundo');
    });
  });
}
