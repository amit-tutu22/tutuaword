import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/references_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F16.S1 Insert footnote', () {
    testWidgets('I-F16-S1-insert-footnote-from-references-ribbon', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReferencesTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_footnote')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('¹'));
      expect(controller.sessionController.statusText, contains('Footnote inserted'));
    });

    testWidgets('I-F16-S1-insert-footnote-direct-api', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return TextButton(
                onPressed: () => controller.insertFootnote(context),
                child: const Text('Footnote'),
              );
            },
          ),
        ),
      );

      await tester.tap(find.text('Footnote'));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, '¹');
    });
  });

  testWidgets('ReferencesTab wires insert footnote button', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ReferencesTab(controller: controller),
        ),
      ),
    );

    expect(find.byKey(const Key('insert_footnote')), findsOneWidget);
  });
}
