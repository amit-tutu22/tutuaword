import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/references_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F16.S3 Citations and bibliography', () {
    testWidgets('I-F16-S3-insert-citation-from-references-ribbon', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReferencesTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_citation')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('(Smith, 2020)'));
      expect(controller.sessionController.statusText, contains('Citation inserted'));
    });

    testWidgets('I-F16-S3-insert-bibliography-from-references-ribbon', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReferencesTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_citation')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);
      await tester.tap(find.byKey(const Key('insert_bibliography')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Bibliography'));
      expect(engine.text, contains('Example Research'));
      expect(controller.sessionController.statusText, contains('Bibliography inserted'));
    });

    testWidgets('I-F16-S3-insert-bibliography-alone-from-references-ribbon', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReferencesTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_bibliography')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Bibliography'));
      expect(controller.sessionController.statusText, contains('Bibliography inserted'));
    });

    testWidgets('I-F16-S3-insert-citation-direct-api', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return TextButton(
                onPressed: () => controller.insertCitation(context),
                child: const Text('Cite'),
              );
            },
          ),
        ),
      );

      await tester.tap(find.text('Cite'));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('(Smith, 2020)'));
    });
  });

  testWidgets('ReferencesTab wires citation and bibliography buttons', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ReferencesTab(controller: controller),
        ),
      ),
    );

    expect(find.byKey(const Key('insert_citation')), findsOneWidget);
    expect(find.byKey(const Key('insert_bibliography')), findsOneWidget);
  });
}
