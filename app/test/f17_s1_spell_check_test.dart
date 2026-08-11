import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F17.S1 Spell check upgrade', () {
    testWidgets('I-F17-S1-spell-check-menu', (tester) async {
      final engine = MockDocumentEngine(
        initialText: 'Teh document has a mispelling.',
      );
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.tap(find.byKey(const Key('spell_check')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(controller.spellMisspellings, contains('Teh'));
      expect(controller.spellMisspellings, contains('mispelling'));
      expect(controller.sessionController.statusText, contains('2 issue'));
    });

    testWidgets('I-F17-S1-spell-check-direct-api', (tester) async {
      final engine = MockDocumentEngine(
        initialText: 'Teh document has a mispelling.',
      );
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return TextButton(
                onPressed: () => controller.spellCheckDocument(),
                child: const Text('Spell'),
              );
            },
          ),
        ),
      );

      await tester.tap(find.text('Spell'));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(controller.spellMisspellings, isNotEmpty);
      expect(controller.spellMisspellings, contains('Teh'));
    });

    testWidgets('I-F17-S1-spell-check-clean-document', (tester) async {
      final engine = MockDocumentEngine(
        initialText: 'The document editor works well.',
      );
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.spellCheckDocument();

      expect(controller.spellMisspellings, isEmpty);
      expect(controller.sessionController.statusText, contains('No spelling issues found'));
    });
  });

  testWidgets('ReviewTab wires spell check button', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ReviewTab(controller: controller),
        ),
      ),
    );

    expect(find.byKey(const Key('spell_check')), findsOneWidget);
  });
}
